//! RAID 4 and RAID 6, aka single and dual parity blocks
//!
//! RAID 4 and RAID 6, short for Redundant Arrays of Independent Disks,
//! are schemes for uses redundant disks to store parity information
//! capable of reconstructing any one (for RAID 4) or two (for RAID 6)
//! failed disks.
//!
//! These schemes be applied to anything organized by blocks, and "disk
//! failure" can be determined by attaching a CRC or other checksum to each
//! block.
//!
//! RAID 4 uses a simple parity calculated by xoring all data blocks
//! together, which is technically addition over gf256:
//!
//!     p = d0 + d1 + d2 + ...
//!
//! We can find/repair any single block by reversing this formula:
//!
//!     d1 = p - (d0 + d2 + ...)
//!
//! RAID 6 adds an additional parity calculated by summing the data
//! blocks multiplied at the byte-level by a primitive element, g, in
//! gf256 raise to a unique power for each block:
//!
//!     q = d0*g^0 + d1*g^1 + d2*g^2 + ...
//!
//! One property that g has as a primitive element is that g^i forms a
//! cycle, eventually iterating through every non-zero element of gf256.
//! As a side-effect, RAID 6 is limited to 255 blocks of data.
//!
//! We now have two formulas that let us solve for any data block,
//! allowing recover if both a data block and a parity block die:
//!
//!          q - (d0*g^0 + d2*g^2 + ...)
//!     d1 = ---------------------------
//!                    g^1
//!
//! With these two formulas we can also solve for any two data blocks:
//!
//!     d0     + d1     = p - (d2 + d3 + ...)
//!     d0*g^0 + d1*g^1 = q - (d2*g^2 + d3*g^3 + ...)
//!
//!          (p - (d2 + d3 + ...))*g^1 + (q - (d2*g^2 + d3*g^3 + ...))
//!     d0 = ---------------------------------------------------------
//!                              g^0 + g^1
//!
//!     d1 = p - (d2 + d3 + ...) - d0
//!
//! Which ends up covering all cases for 2 block failures.
//!

use std::io;
use std::slice;
use std::cell::RefCell;
use std::io::Write;
use std::io::Seek;
use std::convert::TryFrom;
use std::cmp::min;
use std::cmp::max;
use rand;
use ::gf256::*;


//// Raid4 ////

#[derive(Clone)]
pub struct Raid4<'a, B>
where
    B: io::Read + io::Write + io::Seek
{
    // This looks really complicated, but it's just a way to share mutable
    // references between objects safely
    //
    blocks: &'a RefCell<dyn AsMut<[B]>>,
    i: usize,
}

impl<'a, B> Raid4<'a, B>
where
    B: io::Read + io::Write + io::Seek
{
    /// Format blocks with RAID4, aka single block of parity
    ///
    /// Requires N+1 blocks, with the last block being used for parity
    ///
    pub fn format(blocks: &mut [B]) -> Result<(), io::Error> {
        assert!(blocks.len() >= 1+1);

        for i in 0..blocks.len() {
            blocks[i].rewind()?;
        }

        loop {
            let mut p = 0;
            let mut done = true;

            for i in 0..blocks.len()-1 {
                // read from blocks
                let mut x = 0;
                match blocks[i].read(slice::from_mut(&mut x)) {
                    Ok(0)    => { continue; }
                    Ok(..)   => { done = false; }
                    Err(err) => { Err(err)?; }
                }

                // this could be gf256(a) + gf256(b), but that's just xor anyways
                p ^= x;
            }

            if done {
                break;
            }

            // write parity to last block
            blocks[blocks.len()-1].write_all(slice::from_ref(&p))?;
        }

        Ok(())
    }

    /// Repair up to one block of failure
    pub fn repair(
        blocks: &mut [B],
        bad_blocks: &[usize]
    ) -> Result<(), io::Error> {
        assert!(blocks.len() >= 1+1);

        for i in 0..blocks.len() {
            blocks[i].rewind()?;
        }

        if bad_blocks.len() == 0 {
            // no repair needed
            Ok(())
        } else if bad_blocks.len() == 1 {
            // repair using p
            let bad_block = bad_blocks[0];
            loop {
                let mut p = 0;
                let mut done = true;

                for i in (0..blocks.len()).filter(|i| *i != bad_block) {
                    // read from blocks
                    let mut x = 0;
                    match blocks[i].read(slice::from_mut(&mut x)) {
                        Ok(0)    => { continue; }
                        Ok(..)   => { done = false; }
                        Err(err) => { Err(err)?; }
                    }

                    p ^= x;
                }

                if done {
                    break;
                }

                // write repaired block
                blocks[bad_block].write_all(slice::from_ref(&p))?;
            }

            Ok(())
        } else {
            // can't repair
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Too many errors to correct ({} > 1)",
                    bad_blocks.len()
                )
            ))
        }
    }

    /// Create a set of Raid4 blocks, each mapping to an underlying data
    /// block. Writes to a Raid4 will also update the parity block.
    ///
    /// Note this returns N-1 blocks.
    ///
    pub fn mount(blocks: &'a RefCell<dyn AsMut<[B]>>) -> Result<Vec<Self>, io::Error> {
        assert!(blocks.borrow_mut().as_mut().len() >= 1+1);

        {
            let mut blocks = blocks.borrow_mut();
            let blocks = blocks.as_mut();
            for i in 0..blocks.len() {
                blocks[i].rewind()?;
            }
        }

        Ok(
            (0..blocks.borrow_mut().as_mut().len()-1)
                .map(|i| {
                    Self { 
                        blocks: blocks,
                        i: i
                    }
                })
                .collect::<Vec<_>>()
        )
    }
}

impl<B> io::Read for Raid4<'_, B>
where
    B: io::Read + io::Write + io::Seek
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        // just pass to our block
        self.blocks.borrow_mut().as_mut()[self.i].read(buf)
    }
}

impl<B> io::Write for Raid4<'_, B>
where
    B: io::Read + io::Write + io::Seek
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let mut blocks = self.blocks.borrow_mut();
        let blocks = blocks.as_mut();
        let p_i = blocks.len()-1;

        // make sure parity block is at the right position, since we
        // share this it can always be moved
        let pos = blocks[self.i].stream_position()?;
        blocks[p_i].seek(io::SeekFrom::Start(pos))?;

        for b in buf {
            // read old values so we can calculate new values without
            // recomputing the entire parity
            //
            // note if we're eof, the values are treated as zero
            //
            let mut old_b = 0;
            let diff = blocks[self.i].read(slice::from_mut(&mut old_b))?;
            blocks[self.i].seek(io::SeekFrom::Current(-(diff as i64)))?;

            let mut old_p = 0;
            let diff = blocks[p_i].read(slice::from_mut(&mut old_p))?;
            blocks[p_i].seek(io::SeekFrom::Current(-(diff as i64)))?;

            // calculate new parity
            let p = old_p ^ old_b ^ *b;

            // write updated value and parity
            blocks[self.i].write_all(slice::from_ref(&b))?;
            blocks[p_i].write_all(slice::from_ref(&p))?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        // flush both our block and the parity block
        let mut blocks = self.blocks.borrow_mut();
        let blocks = blocks.as_mut();
        let p_i = blocks.len()-1;
        blocks[self.i].flush()?;
        blocks[p_i].flush()?;
        Ok(())
    }
}

impl<B> io::Seek for Raid4<'_, B>
where
    B: io::Read + io::Write + io::Seek
{
    fn seek(&mut self, from_: io::SeekFrom) -> Result<u64, io::Error> {
        // seek our current block, we'll lazily seek the parity block
        // since it is shared across raid blocks
        self.blocks.borrow_mut().as_mut()[self.i].seek(from_)
    }
}


//// Raid6 ////

#[derive(Clone)]
pub struct Raid6<'a, B>
where
    B: io::Read + io::Write + io::Seek
{
    // This looks really complicated, but it's just a way to share mutable
    // references between objects safely
    //
    blocks: &'a RefCell<dyn AsMut<[B]>>,
    i: usize,
    coeff: gf256,
}

impl<'a, B> Raid6<'a, B>
where
    B: io::Read + io::Write + io::Seek
{
    /// Format blocks with RAID4, aka two blocks of parity
    ///
    /// Requires N+2 blocks, with the last two blocks being used for parity
    ///
    pub fn format(blocks: &mut [B]) -> Result<(), io::Error> {
        assert!(blocks.len() >= 1+2);
        assert!(blocks.len() <= 255+2);

        for i in 0..blocks.len() {
            blocks[i].rewind()?;
        }

        loop {
            let mut p = gf256(0);
            let mut q = gf256(0);
            let mut done = true;

            for i in 0..blocks.len()-2 {
                // read from blocks
                let mut x = 0;
                match blocks[i].read(slice::from_mut(&mut x)) {
                    Ok(0)    => { continue; }
                    Ok(..)   => { done = false; }
                    Err(err) => { Err(err)?; }
                }

                p += gf256(x);
                q += gf256(x) * gf256::GENERATOR.pow(u8::try_from(i).unwrap());
            }

            if done {
                break;
            }

            // write parity to last two blocks
            blocks[blocks.len()-2].write_all(slice::from_ref(&u8::from(p)))?;
            blocks[blocks.len()-1].write_all(slice::from_ref(&u8::from(q)))?;
        }

        Ok(())
    }

    /// Repair up to two block of failure
    pub fn repair(
        blocks: &mut [B],
        bad_blocks: &[usize]
    ) -> Result<(), io::Error> {
        assert!(blocks.len() >= 1+2);
        assert!(blocks.len() <= 255+2);

        for i in 0..blocks.len() {
            blocks[i].rewind()?;
        }

        if bad_blocks.len() == 0 {
            // no repair needed
            Ok(())
        } else if bad_blocks.len() == 1 {
            let bad_block = bad_blocks[0];
            if bad_block == blocks.len()-1 {
                // regenerate q
                loop {
                    let mut q = gf256(0);
                    let mut done = true;

                    for i in 0..blocks.len()-2 {
                        // read from blocks
                        let mut x = 0;
                        match blocks[i].read(slice::from_mut(&mut x)) {
                            Ok(0)    => { continue; }
                            Ok(..)   => { done = false; }
                            Err(err) => { Err(err)?; }
                        }

                        q += gf256(x) * gf256::GENERATOR.pow(u8::try_from(i).unwrap());
                    }

                    if done {
                        break;
                    }

                    // write repaired block
                    blocks[bad_block].write_all(slice::from_ref(&u8::from(q)))?;
                }

                Ok(())
            } else {
                // repair using p
                let bad_block = bad_blocks[0];
                loop {
                    let mut p = gf256(0);
                    let mut done = true;

                    for i in (0..blocks.len()-1).filter(|i| *i != bad_block) {
                        // read from blocks
                        let mut x = 0;
                        match blocks[i].read(slice::from_mut(&mut x)) {
                            Ok(0)    => { continue; }
                            Ok(..)   => { done = false; }
                            Err(err) => { Err(err)?; }
                        }

                        p -= gf256(x);
                    }

                    if done {
                        break;
                    }

                    // write repaired block
                    blocks[bad_block].write_all(slice::from_ref(&u8::from(p)))?;
                }

                Ok(())
            }
        } else if bad_blocks.len() == 2 {
            let bad_block1 = min(bad_blocks[0], bad_blocks[1]);
            let bad_block2 = max(bad_blocks[0], bad_blocks[1]);

            if bad_block2 == blocks.len()-1 {
                // repair data block using p, and then regenerate q
                loop {
                    let mut p = gf256(0);
                    let mut q = gf256(0);
                    let mut done = true;

                    for i in (0..blocks.len()-1).filter(|i| *i != bad_block1) {
                        // read from blocks
                        let mut x = 0;
                        match blocks[i].read(slice::from_mut(&mut x)) {
                            Ok(0)    => { continue; }
                            Ok(..)   => { done = false; }
                            Err(err) => { Err(err)?; }
                        }

                        p -= gf256(x);
                        if i < blocks.len()-2 {
                            q += gf256(x) * gf256::GENERATOR.pow(u8::try_from(i).unwrap());
                        }
                    }

                    if done {
                        break;
                    }

                    // q depends on final value
                    if bad_block1 < blocks.len()-2 {
                        q += p * gf256::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
                    }

                    // write repaired blocks
                    blocks[bad_block1].write_all(slice::from_ref(&u8::from(p)))?;
                    blocks[bad_block2].write_all(slice::from_ref(&u8::from(q)))?;
                }

                Ok(())
            } else if bad_block2 == blocks.len()-2 {
                // repair data block using q, and then regenerate p
                loop {

                    let mut p = gf256(0);
                    let mut q;
                    let mut done = true;

                    let mut x = 0;
                    match blocks[blocks.len()-1].read(slice::from_mut(&mut x)) {
                        Ok(0)    => { break; }
                        Ok(..)   => { done = false; }
                        Err(err) => { Err(err)?; }
                    }
                    q = gf256(x);

                    for i in (0..blocks.len()-2).filter(|i| *i != bad_block1) {
                        // read from blocks
                        let mut x = 0;
                        match blocks[i].read(slice::from_mut(&mut x)) {
                            Ok(0)    => { continue; }
                            Ok(..)   => { done = false; }
                            Err(err) => { Err(err)?; }
                        }

                        p += gf256(x);
                        q -= gf256(x) * gf256::GENERATOR.pow(u8::try_from(i).unwrap());
                    }

                    if done {
                        break;
                    }

                    // find final data and p
                    q /= gf256::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
                    p += q;

                    // write repaired blocks
                    blocks[bad_block1].write_all(slice::from_ref(&u8::from(q)))?;
                    blocks[bad_block2].write_all(slice::from_ref(&u8::from(p)))?;
                }

                Ok(())
            } else {
                // repair data blocks using p and q
                loop {
                    let mut p;
                    let mut q;
                    let mut done = true;

                    let mut x = 0;
                    match blocks[blocks.len()-2].read(slice::from_mut(&mut x)) {
                        Ok(0)    => { break; }
                        Ok(..)   => { done = false; }
                        Err(err) => { Err(err)?; }
                    }
                    p = gf256(x);

                    let mut x = 0;
                    match blocks[blocks.len()-1].read(slice::from_mut(&mut x)) {
                        Ok(0)    => { break; }
                        Ok(..)   => { done = false; }
                        Err(err) => { Err(err)?; }
                    }
                    q = gf256(x);

                    for i in (0..blocks.len()-2).filter(|i| *i != bad_block1 && *i != bad_block2) {
                        // read from blocks
                        let mut x = 0;
                        match blocks[i].read(slice::from_mut(&mut x)) {
                            Ok(0)    => { continue; }
                            Ok(..)   => { done = false; }
                            Err(err) => { Err(err)?; }
                        }

                        p += gf256(x);
                        q += gf256(x) * gf256::GENERATOR.pow(u8::try_from(i).unwrap());
                    }

                    if done {
                        break;
                    }

                    // find final data points
                    //
                    // d_x +     d_y     = p-sum(d_i)
                    // d_x*g^x + d_y*g^y = q-sum(d_i*g^i)
                    //
                    //       (p-sum(d_i))*g^y + (q-sum(d_i*g^i))
                    // d_x = -----------------------------------
                    //                   g^x + g^y
                    //
                    let g1 = gf256::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
                    let g2 = gf256::GENERATOR.pow(u8::try_from(bad_block2).unwrap());
                    let d1 = (p*g2 + q) / (g1+g2);
                    let d2 = p + d1;

                    // write repaired blocks
                    blocks[bad_block1].write_all(slice::from_ref(&u8::from(d1)))?;
                    blocks[bad_block2].write_all(slice::from_ref(&u8::from(d2)))?;
                }

                Ok(())
            }
        } else {
            // can't repair
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Too many errors to correct ({} > 2)",
                    bad_blocks.len()
                )
            ))
        }
    }

    /// Create a set of Raid6 blocks, each mapping to an underlying data
    /// block. Writes to a Raid6 will also update the parity blocks.
    ///
    /// Note this returns N-2 blocks.
    ///
    pub fn mount(blocks: &'a RefCell<dyn AsMut<[B]>>) -> Result<Vec<Self>, io::Error> {
        assert!(blocks.borrow_mut().as_mut().len() >= 1+2);
        assert!(blocks.borrow_mut().as_mut().len() <= 255+2);

        {
            let mut blocks = blocks.borrow_mut();
            let blocks = blocks.as_mut();
            for i in 0..blocks.len() {
                blocks[i].rewind()?;
            }
        }

        Ok(
            (0..blocks.borrow_mut().as_mut().len()-1)
                .map(|i| {
                    Self { 
                        blocks: blocks,
                        i: i,
                        coeff: gf256::GENERATOR.pow(u8::try_from(i).unwrap()),
                    }
                })
                .collect::<Vec<_>>()
        )
    }
}

impl<B> io::Read for Raid6<'_, B>
where
    B: io::Read + io::Write + io::Seek
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        // just pass to our block
        self.blocks.borrow_mut().as_mut()[self.i].read(buf)
    }
}

impl<B> io::Write for Raid6<'_, B>
where
    B: io::Read + io::Write + io::Seek
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let mut blocks = self.blocks.borrow_mut();
        let blocks = blocks.as_mut();
        let p_i = blocks.len()-2;
        let q_i = blocks.len()-1;

        // make sure parity blocks are at the right position, since we
        // share these they can always be moved
        let pos = blocks[self.i].stream_position()?;
        blocks[p_i].seek(io::SeekFrom::Start(pos))?;
        blocks[q_i].seek(io::SeekFrom::Start(pos))?;

        for b in buf {
            // read old values so we can calculate new values without
            // recomputing the entire parity
            //
            // note if we're eof, the values are treated as zero
            //
            let mut old_b = 0;
            let diff = blocks[self.i].read(slice::from_mut(&mut old_b))?;
            blocks[self.i].seek(io::SeekFrom::Current(-(diff as i64)))?;

            let mut old_p = 0;
            let diff = blocks[p_i].read(slice::from_mut(&mut old_p))?;
            blocks[p_i].seek(io::SeekFrom::Current(-(diff as i64)))?;

            let mut old_q = 0;
            let diff = blocks[q_i].read(slice::from_mut(&mut old_q))?;
            blocks[q_i].seek(io::SeekFrom::Current(-(diff as i64)))?;

            // calculate new parity
            let p = gf256(old_p) + (gf256(*b) - gf256(old_b));
            let q = gf256(old_q) + self.coeff*(gf256(*b) - gf256(old_b));

            // write updated value and parity
            blocks[self.i].write_all(slice::from_ref(&b))?;
            blocks[p_i].write_all(slice::from_ref(&u8::from(p)))?;
            blocks[q_i].write_all(slice::from_ref(&u8::from(q)))?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        // flush both our block and the parity blocks
        let mut blocks = self.blocks.borrow_mut();
        let blocks = blocks.as_mut();
        let p_i = blocks.len()-2;
        let q_i = blocks.len()-1;
        blocks[self.i].flush()?;
        blocks[p_i].flush()?;
        blocks[q_i].flush()?;
        Ok(())
    }
}

impl<B> io::Seek for Raid6<'_, B>
where
    B: io::Read + io::Write + io::Seek
{
    fn seek(&mut self, from_: io::SeekFrom) -> Result<u64, io::Error> {
        // seek our current block, we'll lazily seek the parity blocks
        // since they are shared across raid blocks
        self.blocks.borrow_mut().as_mut()[self.i].seek(from_)
    }
}



pub fn main() {
    fn hex(xs: &[u8]) -> String {
        xs.iter()
            .map(|x| format!("{:02x}", x))
            .collect()
    }


    // test Raid4

    let mut blocks = [
        io::Cursor::new(Vec::from(b"Hell".as_ref())),
        io::Cursor::new(Vec::from(b"o Wo".as_ref())),
        io::Cursor::new(Vec::from(b"rld?".as_ref())),
        io::Cursor::new(Vec::from(b"    ".as_ref())),
    ];

    println!();
    println!("testing raid4({:?})",
        blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .take(3)
            .collect::<String>()
    );

    Raid4::format(&mut blocks).unwrap();
    println!("{:<7} => {:<19} {}",
        "format",
        format!("{:?}", blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>()),
        blocks.iter()
            .map(|b| hex(b.get_ref()))
            .collect::<String>()
    );

    let blocks_ref = RefCell::new(blocks);
    let mut raid_blocks = Raid4::mount(&blocks_ref).unwrap();
    raid_blocks[2].seek(io::SeekFrom::Start(3)).unwrap();
    raid_blocks[2].write_all(b"!").unwrap();
    let mut blocks = blocks_ref.into_inner();
    println!("{:<7} => {:<19} {}",
        "update",
        format!("{:?}", blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>()),
        blocks.iter()
            .map(|b| hex(b.get_ref()))
            .collect::<String>()
    );

    let mut rng = rand::thread_rng();
    let bad_blocks = rand::seq::index::sample(&mut rng, blocks.len(), 1).into_vec();
    for bad_block in bad_blocks.iter() {
        blocks[*bad_block].get_mut().fill(b'x');
    }
    println!("{:<7} => {:<19} {}",
        "corrupt",
        format!("{:?}", blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>()),
        blocks.iter()
            .map(|b| hex(b.get_ref()))
            .collect::<String>()
    );

    Raid4::repair(&mut blocks, &bad_blocks).unwrap();
    println!("{:<7} => {:<19} {}",
        "repair",
        format!("{:?}", blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>()),
        blocks.iter()
            .map(|b| hex(b.get_ref()))
            .collect::<String>()
    );
    assert_eq!(
        blocks.iter()
            .take(3)
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>(),
        "Hello World!"
    );


    // test Raid6

    let mut blocks = [
        io::Cursor::new(Vec::from(b"Hell".as_ref())),
        io::Cursor::new(Vec::from(b"o Wo".as_ref())),
        io::Cursor::new(Vec::from(b"rld?".as_ref())),
        io::Cursor::new(Vec::from(b"    ".as_ref())),
        io::Cursor::new(Vec::from(b"    ".as_ref())),
    ];

    println!();
    println!("testing raid6({:?})",
        blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .take(3)
            .collect::<String>()
    );

    Raid6::format(&mut blocks).unwrap();
    println!("{:<7} => {:<23} {}",
        "format",
        format!("{:?}", blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>()),
        blocks.iter()
            .map(|b| hex(b.get_ref()))
            .collect::<String>()
    );

    let blocks_ref = RefCell::new(blocks);
    let mut raid_blocks = Raid6::mount(&blocks_ref).unwrap();
    raid_blocks[2].seek(io::SeekFrom::Start(3)).unwrap();
    raid_blocks[2].write_all(b"!").unwrap();
    let mut blocks = blocks_ref.into_inner();
    println!("{:<7} => {:<23} {}",
        "update",
        format!("{:?}", blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>()),
        blocks.iter()
            .map(|b| hex(b.get_ref()))
            .collect::<String>()
    );

    let mut rng = rand::thread_rng();
    let bad_blocks = rand::seq::index::sample(&mut rng, blocks.len(), 1).into_vec();
    for bad_block in bad_blocks.iter() {
        blocks[*bad_block].get_mut().fill(b'x');
    }
    println!("{:<7} => {:<23} {}",
        "corrupt",
        format!("{:?}", blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>()),
        blocks.iter()
            .map(|b| hex(b.get_ref()))
            .collect::<String>()
    );

    Raid6::repair(&mut blocks, &bad_blocks).unwrap();
    println!("{:<7} => {:<23} {}",
        "repair",
        format!("{:?}", blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>()),
        blocks.iter()
            .map(|b| hex(b.get_ref()))
            .collect::<String>()
    );
    assert_eq!(
        blocks.iter()
            .take(3)
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>(),
        "Hello World!"
    );

    let mut rng = rand::thread_rng();
    let bad_blocks = rand::seq::index::sample(&mut rng, blocks.len(), 2).into_vec();
    for bad_block in bad_blocks.iter() {
        blocks[*bad_block].get_mut().fill(b'x');
    }
    println!("{:<7} => {:<23} {}",
        "corrupt",
        format!("{:?}", blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>()),
        blocks.iter()
            .map(|b| hex(b.get_ref()))
            .collect::<String>()
    );

    Raid6::repair(&mut blocks, &bad_blocks).unwrap();
    println!("{:<7} => {:<23} {}",
        "repair",
        format!("{:?}", blocks.iter()
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>()),
        blocks.iter()
            .map(|b| hex(b.get_ref()))
            .collect::<String>()
    );
    assert_eq!(
        blocks.iter()
            .take(3)
            .map(|b| String::from_utf8_lossy(b.get_ref()))
            .collect::<String>(),
        "Hello World!"
    );

    println!();
}
