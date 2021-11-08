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

use std::convert::TryFrom;
use std::cmp::min;
use std::cmp::max;
use std::fmt;
use rand;
use ::gf256::*;


/// Error codes for RAID arrays
#[derive(Debug, Clone)]
pub enum RaidError {
    /// RAID-parity can fail to decode if there are more bad-blocks
    /// than there are parity blocks
    ///
    TooManyBadBlocks,
}

impl fmt::Display for RaidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaidError::TooManyBadBlocks => write!(f, "Too many bad-blocks to repair"),
        }
    }
}

/// Convert mut slice of u8s to gf256s
fn gf256_slice_mut(slice: &mut [u8]) -> &mut [gf256] {
    // I couldn't find a safe way to do this cheaply+safely
    unsafe {
        std::slice::from_raw_parts_mut(
            slice.as_mut_ptr() as *mut gf256,
            slice.len()
        )
    }
}


//// RAID4 ////

/// Format blocks with RAID4, aka single block of parity
pub fn raid4_format<B: AsRef<[u8]>>(blocks: &[B], p: &mut [u8]) {
    let len = p.len();
    assert!(blocks.iter().all(|b| b.as_ref().len() == len));

    for i in 0..len {
        p[i] = 0;
    }

    for b in blocks {
        for i in 0..len {
            // this could be gf256(a) + gf256(b), but that's just xor anyways
            p[i] ^= b.as_ref()[i];
        }
    }
}

/// Repair up to one block of failure
pub fn raid4_repair<B: AsMut<[u8]>>(
    blocks: &mut [B],
    p: &mut [u8],
    bad_blocks: &[usize]
) -> Result<(), RaidError> {
    let len = p.len();

    if bad_blocks.len() == 0 {
        // no repair needed
        Ok(())
    } else if bad_blocks.len() == 1 {
        let bad_block = bad_blocks[0];
        assert!(bad_block < blocks.len()+1);

        if bad_block < blocks.len() {
            // repair using p
            let (before, after) = blocks.split_at_mut(bad_block);
            let (d, after) = after.split_first_mut().unwrap();
            let d = d.as_mut();

            for i in 0..len {
                d[i] = p[i];
            }

            for b in before.iter_mut().chain(after.iter_mut()) {
                for i in 0..len {
                    d[i] ^= b.as_mut()[i];
                }
            }
            Ok(())
        } else if bad_block == blocks.len()+0 {
            // regenerate p
            for i in 0..len {
                p[i] = 0;
            }

            for b in blocks.iter_mut() {
                for i in 0..len {
                    p[i] ^= b.as_mut()[i];
                }
            }
            Ok(())
        } else {
            unreachable!()
        }
    } else {
        // can't repair
        Err(RaidError::TooManyBadBlocks)
    }
}

/// Add a block to a RAID4 array
///
/// Note the block index must be unique in the array! This does not
/// update other block indices.
///
pub fn raid4_add(_j: usize, new: &[u8], p: &mut [u8]) {
    let len = p.len();

    for i in 0..len {
        // calculate new parity
        p[i] ^= new[i];
    }
}

/// Add a block from a RAID4 array
///
/// Note the block index must already exit in the array, otherwise the
/// array will become corrupted. This does not update other block indices.
///
pub fn raid4_remove(_j: usize, old: &[u8], p: &mut [u8]) {
    let len = p.len();

    for i in 0..len {
        // calculate new parity
        p[i] ^= old[i];
    }
}

/// Update a block in a RAID4 array
///
/// This is functionally equivalent to remove(i)+add(i), but more efficient.
///
pub fn raid4_update(_j: usize, old: &[u8], new: &[u8], p: &mut [u8]) {
    let len = p.len();

    for i in 0..len {
        // calculate new parity
        p[i] ^= old[i] ^ new[i];
    }
}


//// RAID6 ////

/// Format blocks with RAID6, aka two blocks of parity
pub fn raid6_format<B: AsRef<[u8]>>(blocks: &[B], p: &mut [u8], q: &mut [u8]) {
    let len = p.len();
    assert!(q.len() == len);
    assert!(blocks.iter().all(|b| b.as_ref().len() == len));
    assert!(blocks.len() <= 255);
    let p = gf256_slice_mut(p);
    let q = gf256_slice_mut(q);

    for i in 0..len {
        p[i] = gf256(0);
        q[i] = gf256(0);
    }

    for (j, b) in blocks.iter().enumerate() {
        let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
        for i in 0..len {
            p[i] += gf256(b.as_ref()[i]);
            q[i] += gf256(b.as_ref()[i]) * g;
        }
    }
}

/// Repair up to two blocks of failure
pub fn raid6_repair<B: AsMut<[u8]>>(
    blocks: &mut [B],
    p: &mut [u8],
    q: &mut [u8],
    bad_blocks: &[usize]
) -> Result<(), RaidError> {
    let len = p.len();
    let p = gf256_slice_mut(p);
    let q = gf256_slice_mut(q);

    if bad_blocks.len() == 0 {
        // no repair needed
        Ok(())
    } else if bad_blocks.len() == 1 {
        let bad_block = bad_blocks[0];
        assert!(bad_block < blocks.len()+2);

        if bad_block < blocks.len() {
            // repair using p
            let (before, after) = blocks.split_at_mut(bad_block);
            let (d, after)  = after.split_first_mut().unwrap();
            let d = gf256_slice_mut(d.as_mut());

            for i in 0..len {
                d[i] = p[i];
            }

            for b in before.iter_mut().chain(after.iter_mut()) {
                for i in 0..len {
                    d[i] -= gf256(b.as_mut()[i]);
                }
            }
            Ok(())
        } else if bad_block == blocks.len()+0 {
            // regenerate p
            for i in 0..len {
                p[i] = gf256(0);
            }

            for b in blocks.iter_mut() {
                for i in 0..len {
                    p[i] += gf256(b.as_mut()[i]);
                }
            }
            Ok(())
        } else if bad_block == blocks.len()+1 {
            // regenerate q
            for i in 0..len {
                q[i] = gf256(0);
            }

            for (j, b) in blocks.iter_mut().enumerate() {
                let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
                for i in 0..len {
                    q[i] += gf256(b.as_mut()[i]) * g;
                }
            }
            Ok(())
        } else {
            unreachable!()
        }
    } else if bad_blocks.len() == 2 {
        let bad_block1 = min(bad_blocks[0], bad_blocks[1]);
        let bad_block2 = max(bad_blocks[0], bad_blocks[1]);
        assert!(bad_block1 < blocks.len()+2);
        assert!(bad_block2 < blocks.len()+2);

        if bad_block1 < blocks.len() && bad_block2 < blocks.len() {
            // repair d1 and d2 using p and q
            let (before, between) = blocks.split_at_mut(bad_block1);
            let (d1, between) = between.split_first_mut().unwrap();
            let (between, after) = between.split_at_mut(bad_block2-(bad_block1+1));
            let (d2, after) = after.split_first_mut().unwrap();
            let d1 = gf256_slice_mut(d1.as_mut());
            let d2 = gf256_slice_mut(d2.as_mut());

            // find intermediate values
            //
            // p-sum(d_i)
            // q-sum(d_i*g^i)
            //
            for i in 0..len {
                d1[i] = p[i];
                d2[i] = q[i];
            }

            for (j, b) in before.iter_mut().enumerate()
                .chain((bad_block1+1..).zip(between.iter_mut()))
                .chain((bad_block2+1..).zip(after.iter_mut()))
            {
                let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
                for i in 0..len {
                    d1[i] -= gf256(b.as_mut()[i]);
                    d2[i] -= gf256(b.as_mut()[i]) * g;
                }
            }

            // find final d1/d2
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
            for i in 0..len {
                let p = d1[i];
                let q = d2[i];
                d1[i] = (p*g2 + q) / (g1+g2);
                d2[i] = p - d1[i];
            }
            Ok(())
        } else if bad_block1 < blocks.len() && bad_block2 == blocks.len()+0 {
            // repair d using q, and then regenerate p
            let (before, after) = blocks.split_at_mut(bad_block1);
            let (d, after) = after.split_first_mut().unwrap();
            let d = gf256_slice_mut(d.as_mut());

            for i in 0..len {
                d[i] = q[i];
                p[i] = gf256(0);
            }

            for (j, b) in before.iter_mut().enumerate()
                .chain((bad_block1+1..).zip(after.iter_mut()))
            {
                let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
                for i in 0..len {
                    d[i] -= gf256(b.as_mut()[i]) * g;
                    p[i] += gf256(b.as_mut()[i]);
                }
            }

            // update p and d based on final value of d
            let g = gf256::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
            for i in 0..len {
                d[i] /= g;
                p[i] += d[i];
            }
            Ok(())
        } else if bad_block1 < blocks.len() && bad_block2 == blocks.len()+1 {
            // repair d using p, and then regenerate q
            let (before, after) = blocks.split_at_mut(bad_block1);
            let (d, after) = after.split_first_mut().unwrap();
            let d = gf256_slice_mut(d.as_mut());

            for i in 0..len {
                d[i] = p[i];
                q[i] = gf256(0);
            }

            for (j, b) in before.iter_mut().enumerate()
                .chain((bad_block1+1..).zip(after.iter_mut()))
            {
                let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
                for i in 0..len {
                    d[i] -= gf256(b.as_mut()[i]);
                    q[i] += gf256(b.as_mut()[i]) * g;
                }
            }

            // update q based on final value of d
            let g = gf256::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
            for i in 0..len {
                q[i] += d[i] * g;
            }
            Ok(())
        } else if bad_block1 == blocks.len()+0 && bad_block2 == blocks.len()+1 {
            // regenerate p and q
            for i in 0..len {
                p[i] = gf256(0);
                q[i] = gf256(0);
            }

            for (j, b) in blocks.iter_mut().enumerate() {
                let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
                for i in 0..len {
                    p[i] += gf256(b.as_mut()[i]);
                    q[i] += gf256(b.as_mut()[i]) * g;
                }
            }
            Ok(())
        } else {
            unreachable!()
        }
    } else {
        // can't repair
        Err(RaidError::TooManyBadBlocks)
    }
}

/// Add a block to a RAID6 array
///
/// Note the block index must be unique in the array! This does not
/// update other block indices.
///
pub fn raid6_add(j: usize, new: &[u8], p: &mut [u8], q: &mut [u8]) {
    let len = p.len();
    let p = gf256_slice_mut(p);
    let q = gf256_slice_mut(q);

    let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
    for i in 0..len {
        // calculate new parity
        p[i] += gf256(new[i]);
        q[i] += gf256(new[i]) * g;
    }
}

/// Add a block from a RAID6 array
///
/// Note the block index must already exit in the array, otherwise the
/// array will become corrupted. This does not update other block indices.
///
pub fn raid6_remove(j: usize, old: &[u8], p: &mut [u8], q: &mut [u8]) {
    let len = p.len();
    let p = gf256_slice_mut(p);
    let q = gf256_slice_mut(q);

    let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
    for i in 0..len {
        // calculate new parity
        p[i] -= gf256(old[i]);
        q[i] -= gf256(old[i]) * g;
    }
}

/// Update a block in a RAID6 array
///
/// This is functionally equivalent to remove(i)+add(i), but more efficient.
///
pub fn raid6_update(j: usize, old: &[u8], new: &[u8], p: &mut [u8], q: &mut [u8]) {
    let len = p.len();
    let p = gf256_slice_mut(p);
    let q = gf256_slice_mut(q);

    let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
    for i in 0..len {
        // calculate new parity
        p[i] += gf256(new[i]) - gf256(old[i]);
        q[i] += (gf256(new[i]) - gf256(old[i])) * g;
    }
}


pub fn main() {
    fn hex(xs: &[u8]) -> String {
        xs.iter()
            .map(|x| format!("{:02x}", x))
            .collect()
    }

    fn ascii(xs: &[u8]) -> String {
        xs.iter()
            .map(|x| {
                if *x < b' ' || *x > b'~' {
                    '.'
                } else {
                    char::from(*x)
                }
            })
            .collect::<String>()
    }


    // testing RAID4

    let mut blocks = [
        Vec::from(b"Hell".as_ref()),
        Vec::from(b"o Wo".as_ref()),
        Vec::from(b"rld?".as_ref()),
    ];
    let mut parity = Vec::from(b"    ".as_ref());

    println!();
    println!("testing raid4({:?})",
        blocks.iter()
            .map(|b| ascii(b))
            .collect::<String>()
    );

    raid4_format(&blocks, &mut parity);
    println!("{:<7} => {}  {}",
        "format",
        blocks.iter()
            .chain([&parity])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity])
            .map(|b| hex(b))
            .collect::<String>()
    );

    let old = blocks[2][3];
    blocks[2][3] = b'!';
    raid4_update(2, &[old], &[b'!'], &mut parity[3..4]);
    println!("{:<7} => {}  {}",
        "update",
        blocks.iter()
            .chain([&parity])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity])
            .map(|b| hex(b))
            .collect::<String>()
    );

    let mut rng = rand::thread_rng();
    let bad_blocks = rand::seq::index::sample(&mut rng, blocks.len()+1, 1).into_vec();
    for bad_block in bad_blocks.iter() {
        if *bad_block < blocks.len() {
            blocks[*bad_block].fill(b'x');
        } else if *bad_block == blocks.len()+0 {
            parity.fill(b'x');
        } else {
            unreachable!();
        }
    }
    println!("{:<7} => {}  {}",
        "corrupt",
        blocks.iter()
            .chain([&parity])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity])
            .map(|b| hex(b))
            .collect::<String>()
    );

    raid4_repair(&mut blocks, &mut parity, &bad_blocks).unwrap();
    println!("{:<7} => {}  {}",
        "repair",
        blocks.iter()
            .chain([&parity])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity])
            .map(|b| hex(b))
            .collect::<String>()
    );
    assert_eq!(
        blocks.iter()
            .map(|b| ascii(b))
            .collect::<String>(),
        "Hello World!"
    );


    // testing RAID6

    let mut blocks = [
        Vec::from(b"Hell".as_ref()),
        Vec::from(b"o Wo".as_ref()),
        Vec::from(b"rld?".as_ref()),
    ];
    let mut parity1 = Vec::from(b"    ".as_ref());
    let mut parity2 = Vec::from(b"    ".as_ref());

    println!();
    println!("testing raid6({:?})",
        blocks.iter()
            .map(|b| ascii(b))
            .collect::<String>()
    );

    raid6_format(&blocks, &mut parity1, &mut parity2);
    println!("{:<7} => {}  {}",
        "format",
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| hex(b))
            .collect::<String>()
    );

    let old = blocks[2][3];
    blocks[2][3] = b'!';
    raid6_update(2, &[old], &[b'!'], &mut parity1[3..4], &mut parity2[3..4]);
    println!("{:<7} => {}  {}",
        "update",
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| hex(b))
            .collect::<String>()
    );

    let mut rng = rand::thread_rng();
    let bad_blocks = rand::seq::index::sample(&mut rng, blocks.len()+2, 1).into_vec();
    for bad_block in bad_blocks.iter() {
        if *bad_block < blocks.len() {
            blocks[*bad_block].fill(b'x');
        } else if *bad_block == blocks.len()+0 {
            parity1.fill(b'x');
        } else if *bad_block == blocks.len()+1 {
            parity2.fill(b'x');
        } else {
            unreachable!();
        }
    }
    println!("{:<7} => {}  {}",
        "corrupt",
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| hex(b))
            .collect::<String>()
    );

    raid6_repair(&mut blocks, &mut parity1, &mut parity2, &bad_blocks).unwrap();
    println!("{:<7} => {}  {}",
        "repair",
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| hex(b))
            .collect::<String>()
    );
    assert_eq!(
        blocks.iter()
            .map(|b| ascii(b))
            .collect::<String>(),
        "Hello World!"
    );

    let mut rng = rand::thread_rng();
    let bad_blocks = rand::seq::index::sample(&mut rng, blocks.len()+2, 2).into_vec();
    for bad_block in bad_blocks.iter() {
        if *bad_block < blocks.len() {
            blocks[*bad_block].fill(b'x');
        } else if *bad_block == blocks.len()+0 {
            parity1.fill(b'x');
        } else if *bad_block == blocks.len()+1 {
            parity2.fill(b'x');
        } else {
            unreachable!();
        }
    }
    println!("{:<7} => {}  {}",
        "corrupt",
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| hex(b))
            .collect::<String>()
    );

    raid6_repair(&mut blocks, &mut parity1, &mut parity2, &bad_blocks).unwrap();
    println!("{:<7} => {}  {}",
        "repair",
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2])
            .map(|b| hex(b))
            .collect::<String>()
    );
    assert_eq!(
        blocks.iter()
            .map(|b| ascii(b))
            .collect::<String>(),
        "Hello World!"
    );

    println!();
}
