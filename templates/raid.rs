//! Template for LFSR structs
//!
//! See examples/lfsr.rs for a more detailed explanation of
//! where these implementations come from

use __crate::internal::cfg_if::cfg_if;
use __crate::traits::TryFrom;
use core::cell::RefCell;
use core::slice;
use core::cmp::min;
use core::cmp::max;

extern crate alloc;
use alloc::vec::Vec;

extern crate std;
use std::io;
use std::format;


// TODO should __raid actually take an AsRef<RefCell<dyn AsMut<[B]>>>?
// This would allow Rc, etc

/// TODO doc
#[derive(Clone)]
pub struct __raid<'a, B>
where
    B: io::Read + io::Write + io::Seek
{
    // This looks really complicated, but it's just a way to share mutable
    // references between objects safely
    //
    blocks: &'a RefCell<dyn AsMut<[B]>>,
    i: usize,

    #[cfg(__if(__parity >= 2))]
    coeff: __gf,
}

impl<'a, B> __raid<'a, B>
where
    B: io::Read + io::Write + io::Seek
{
// TODO can we select comments here?
//
//    /// Format blocks with RAID4, aka single block of parity
//    ///
//    /// Requires N+1 blocks, with the last block being used for parity
//    ///
    /// Format blocks with RAID4, aka two blocks of parity
    ///
    /// Requires N+2 blocks, with the last two blocks being used for parity
    ///
    pub fn format(blocks: &mut [B]) -> Result<(), io::Error> {
        assert!(blocks.len() >= 1+__parity);
        #[cfg(__if(__parity >= 2))]
        assert!(blocks.len() <= 255+__parity);

        cfg_if! {
            if #[cfg(__if(__parity == 0))] {
                Ok(())
            } else if #[cfg(__if(__parity == 1))] {
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
            } else if #[cfg(__if(__parity == 2))] {
                for i in 0..blocks.len() {
                    blocks[i].rewind()?;
                }

                loop {
                    let mut p = __gf(0);
                    let mut q = __gf(0);
                    let mut done = true;

                    for i in 0..blocks.len()-2 {
                        // read from blocks
                        let mut x = 0;
                        match blocks[i].read(slice::from_mut(&mut x)) {
                            Ok(0)    => { continue; }
                            Ok(..)   => { done = false; }
                            Err(err) => { Err(err)?; }
                        }

                        p += __gf(x);
                        q += __gf(x) * __gf::GENERATOR.pow(u8::try_from(i).unwrap());
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
        }
    }

// TODO
//    /// Repair up to one block of failure
    /// Repair up to two block of failure
    pub fn repair(
        blocks: &mut [B],
        bad_blocks: &[usize]
    ) -> Result<(), io::Error> {
        assert!(blocks.len() >= 1+__parity);
        #[cfg(__if(__parity >= 2))]
        assert!(blocks.len() <= 255+__parity);

        cfg_if! {
            if #[cfg(__if(__parity == 0))] {
                if bad_blocks.len() == 0 {
                    // no repair needed
                    Ok(())
                } else {
                    // can't repair
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Too many errors to correct ({} > {})",
                            bad_blocks.len(),
                            __parity
                        )
                    ))
                }
            } else if #[cfg(__if(__parity == 1))] {
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
                        format!("Too many errors to correct ({} > {})",
                            bad_blocks.len(),
                            __parity
                        )
                    ))
                }
            } else if #[cfg(__if(__parity == 2))] {
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
                            let mut q = __gf(0);
                            let mut done = true;

                            for i in 0..blocks.len()-2 {
                                // read from blocks
                                let mut x = 0;
                                match blocks[i].read(slice::from_mut(&mut x)) {
                                    Ok(0)    => { continue; }
                                    Ok(..)   => { done = false; }
                                    Err(err) => { Err(err)?; }
                                }

                                q += __gf(x) * __gf::GENERATOR.pow(u8::try_from(i).unwrap());
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
                            let mut p = __gf(0);
                            let mut done = true;

                            for i in (0..blocks.len()-1).filter(|i| *i != bad_block) {
                                // read from blocks
                                let mut x = 0;
                                match blocks[i].read(slice::from_mut(&mut x)) {
                                    Ok(0)    => { continue; }
                                    Ok(..)   => { done = false; }
                                    Err(err) => { Err(err)?; }
                                }

                                p -= __gf(x);
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
                            let mut p = __gf(0);
                            let mut q = __gf(0);
                            let mut done = true;

                            for i in (0..blocks.len()-1).filter(|i| *i != bad_block1) {
                                // read from blocks
                                let mut x = 0;
                                match blocks[i].read(slice::from_mut(&mut x)) {
                                    Ok(0)    => { continue; }
                                    Ok(..)   => { done = false; }
                                    Err(err) => { Err(err)?; }
                                }

                                p -= __gf(x);
                                if i < blocks.len()-2 {
                                    q += __gf(x) * __gf::GENERATOR.pow(u8::try_from(i).unwrap());
                                }
                            }

                            if done {
                                break;
                            }

                            // q depends on final value
                            if bad_block1 < blocks.len()-2 {
                                q += p * __gf::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
                            }

                            // write repaired blocks
                            blocks[bad_block1].write_all(slice::from_ref(&u8::from(p)))?;
                            blocks[bad_block2].write_all(slice::from_ref(&u8::from(q)))?;
                        }

                        Ok(())
                    } else if bad_block2 == blocks.len()-2 {
                        // repair data block using q, and then regenerate p
                        loop {

                            let mut p = __gf(0);
                            let mut q;
                            let mut done = true;

                            let mut x = 0;
                            match blocks[blocks.len()-1].read(slice::from_mut(&mut x)) {
                                Ok(0)    => { break; }
                                Ok(..)   => { done = false; }
                                Err(err) => { Err(err)?; }
                            }
                            q = __gf(x);

                            for i in (0..blocks.len()-2).filter(|i| *i != bad_block1) {
                                // read from blocks
                                let mut x = 0;
                                match blocks[i].read(slice::from_mut(&mut x)) {
                                    Ok(0)    => { continue; }
                                    Ok(..)   => { done = false; }
                                    Err(err) => { Err(err)?; }
                                }

                                p += __gf(x);
                                q -= __gf(x) * __gf::GENERATOR.pow(u8::try_from(i).unwrap());
                            }

                            if done {
                                break;
                            }

                            // find final data and p
                            q /= __gf::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
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
                            p = __gf(x);

                            let mut x = 0;
                            match blocks[blocks.len()-1].read(slice::from_mut(&mut x)) {
                                Ok(0)    => { break; }
                                Ok(..)   => { done = false; }
                                Err(err) => { Err(err)?; }
                            }
                            q = __gf(x);

                            for i in (0..blocks.len()-2).filter(|i| *i != bad_block1 && *i != bad_block2) {
                                // read from blocks
                                let mut x = 0;
                                match blocks[i].read(slice::from_mut(&mut x)) {
                                    Ok(0)    => { continue; }
                                    Ok(..)   => { done = false; }
                                    Err(err) => { Err(err)?; }
                                }

                                p += __gf(x);
                                q += __gf(x) * __gf::GENERATOR.pow(u8::try_from(i).unwrap());
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
                            let g1 = __gf::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
                            let g2 = __gf::GENERATOR.pow(u8::try_from(bad_block2).unwrap());
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
                        format!("Too many errors to correct ({} > {})",
                            bad_blocks.len(),
                            __parity
                        )
                    ))
                }
            }
        }
    }

// TODO
//    /// Create a set of Raid4 blocks, each mapping to an underlying data
//    /// block. Writes to a Raid4 will also update the parity block.
//    ///
//    /// Note this returns N-1 blocks.
//    ///
    /// Create a set of Raid6 blocks, each mapping to an underlying data
    /// block. Writes to a Raid6 will also update the parity blocks.
    ///
    /// Note this returns N-2 blocks.
    ///
    pub fn mount(blocks: &'a RefCell<dyn AsMut<[B]>>) -> Result<Vec<Self>, io::Error> {
        assert!(blocks.borrow_mut().as_mut().len() >= 1+__parity);
        #[cfg(__if(__parity >= 2))]
        assert!(blocks.borrow_mut().as_mut().len() <= 255+__parity);

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
                        #[cfg(__if(__parity >= 2))]
                        coeff: __gf::GENERATOR.pow(u8::try_from(i).unwrap()),
                    }
                })
                .collect::<Vec<_>>()
        )
    }
}

impl<B> io::Read for __raid<'_, B>
where
    B: io::Read + io::Write + io::Seek
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        // just pass to our block
        self.blocks.borrow_mut().as_mut()[self.i].read(buf)
    }
}

impl<B> io::Write for __raid<'_, B>
where
    B: io::Read + io::Write + io::Seek
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        cfg_if! {
            if #[cfg(__if(__parity == 0))] {
                let mut blocks = self.blocks.borrow_mut();
                let blocks = blocks.as_mut();
                blocks[self.i].write(buf)
            } else if #[cfg(__if(__parity == 1))] {
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
            } else if #[cfg(__if(__parity == 2))] {
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
                    let p = __gf(old_p) + (__gf(*b) - __gf(old_b));
                    let q = __gf(old_q) + self.coeff*(__gf(*b) - __gf(old_b));

                    // write updated value and parity
                    blocks[self.i].write_all(slice::from_ref(&b))?;
                    blocks[p_i].write_all(slice::from_ref(&u8::from(p)))?;
                    blocks[q_i].write_all(slice::from_ref(&u8::from(q)))?;
                }

                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        cfg_if! {
            if #[cfg(__if(__parity == 0))] {
                // flush our block
                let mut blocks = self.blocks.borrow_mut();
                let blocks = blocks.as_mut();
                blocks[self.i].flush()
            } else if #[cfg(__if(__parity == 1))] {
                // flush both our block and the parity block
                let mut blocks = self.blocks.borrow_mut();
                let blocks = blocks.as_mut();
                let p_i = blocks.len()-1;
                blocks[self.i].flush()?;
                blocks[p_i].flush()?;
                Ok(())
            } else if #[cfg(__if(__parity == 2))] {
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
    }
}

impl<B> io::Seek for __raid<'_, B>
where
    B: io::Read + io::Write + io::Seek
{
    fn seek(&mut self, from_: io::SeekFrom) -> Result<u64, io::Error> {
        // seek our current block, we'll lazily seek the parity blocks
        // since they are shared across raid blocks
        self.blocks.borrow_mut().as_mut()[self.i].seek(from_)
    }
}

