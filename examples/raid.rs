//! RAID-parity functions and macros, implemented using our Galois-field types
//!
//! RAID, short for a "Redundant Array of Independent Disks", is a set of
//! schemes commonly found in storage systems, with the purpose of using an
//! array of multiple disks to provide data redundancy and/or performance
//! improvements.
//!
//! The most interesting part for us is the higher-numbered RAID-levels, which
//! use extra disks to store parity information, capable of reconstructing any
//! one (for RAID 5), two (for RAID 6), and even three (for RAID 7, though this
//! name is not standardized) failed disks.
//!
//! More information on how RAID-parity works can be found in [`raid`'s
//! module-level documentation][raid-mod].
//!
//! [raid-mod]: https://docs.rs/gf256/latest/gf256/raid

use std::convert::TryFrom;
use std::fmt;
use rand;
use ::gf256::*;
use ::gf256::crc::crc32c;


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


//// RAID5 ////

/// Format blocks with RAID5, aka single block of parity
pub fn raid5_format<B: AsRef<[u8]>>(blocks: &[B], p: &mut [u8]) {
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
pub fn raid5_repair<B: AsMut<[u8]>>(
    blocks: &mut [B],
    p: &mut [u8],
    bad_blocks: &[usize]
) -> Result<(), RaidError> {
    let len = p.len();

    if bad_blocks.len() > 1 {
        // can't repair
        return Err(RaidError::TooManyBadBlocks);
    }

    if bad_blocks[0] < blocks.len() {
        // repair using p
        let (before, after) = blocks.split_at_mut(bad_blocks[0]);
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
    } else if bad_blocks[0] == blocks.len() {
        // regenerate p
        for i in 0..len {
            p[i] = 0;
        }

        for b in blocks.iter_mut() {
            for i in 0..len {
                p[i] ^= b.as_mut()[i];
            }
        }
    }

    Ok(())
}

/// Add a block to a RAID5 array
///
/// Note the block index must be unique in the array! This does not
/// update other block indices.
///
pub fn raid5_add(_j: usize, new: &[u8], p: &mut [u8]) {
    let len = p.len();

    for i in 0..len {
        // calculate new parity
        p[i] ^= new[i];
    }
}

/// Add a block from a RAID5 array
///
/// Note the block index must already exit in the array, otherwise the
/// array will become corrupted. This does not update other block indices.
///
pub fn raid5_remove(_j: usize, old: &[u8], p: &mut [u8]) {
    let len = p.len();

    for i in 0..len {
        // calculate new parity
        p[i] ^= old[i];
    }
}

/// Update a block in a RAID5 array
///
/// This is functionally equivalent to remove(i)+add(i), but more efficient.
///
pub fn raid5_update(_j: usize, old: &[u8], new: &[u8], p: &mut [u8]) {
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
    let p = gf256::slice_from_slice_mut(p);
    let q = gf256::slice_from_slice_mut(q);

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
    let p = gf256::slice_from_slice_mut(p);
    let q = gf256::slice_from_slice_mut(q);

    if bad_blocks.len() > 2 {
        // can't repair
        return Err(RaidError::TooManyBadBlocks);
    }

    // sort the data blocks without alloc, this is only so we can split
    // the mut blocks array safely
    let mut bad_blocks_array = [
        bad_blocks.get(0).copied().unwrap_or(0),
        bad_blocks.get(1).copied().unwrap_or(0),
    ];
    let bad_blocks = &mut bad_blocks_array[..bad_blocks.len()];
    bad_blocks.sort_unstable();

    if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 1
        && !bad_blocks.iter().any(|b| *b == blocks.len()+0)
    {
        // repair using p
        let (before, after) = blocks.split_at_mut(bad_blocks[0]);
        let (d, after) = after.split_first_mut().unwrap();
        let d = gf256::slice_from_slice_mut(d.as_mut());

        for i in 0..len {
            d[i] = p[i];
        }

        for b in before.iter_mut().chain(after.iter_mut()) {
            for i in 0..len {
                d[i] -= gf256(b.as_mut()[i]);
            }
        }
    } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 1
        && !bad_blocks.iter().any(|b| *b == blocks.len()+1)
    {
        // repair using q
        let (before, after) = blocks.split_at_mut(bad_blocks[0]);
        let (d, after) = after.split_first_mut().unwrap();
        let d = gf256::slice_from_slice_mut(d.as_mut());

        for i in 0..len {
            d[i] = q[i];
        }

        for (j, b) in before.iter_mut().enumerate()
            .chain((bad_blocks[0]+1..).zip(after.iter_mut()))
        {
            let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
            for i in 0..len {
                d[i] -= gf256(b.as_mut()[i]) * g;
            }
        }

        let g = gf256::GENERATOR.pow(u8::try_from(bad_blocks[0]).unwrap());
        for i in 0..len {
            d[i] /= g;
        }
    } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 2
        && !bad_blocks.iter().any(|b| *b == blocks.len()+0 || *b == blocks.len()+1)
    {
        // repair dx and dy using p and q
        let (before, between) = blocks.split_at_mut(bad_blocks[0]);
        let (dx, between) = between.split_first_mut().unwrap();
        let (between, after) = between.split_at_mut(bad_blocks[1]-(bad_blocks[0]+1));
        let (dy, after) = after.split_first_mut().unwrap();
        let dx = gf256::slice_from_slice_mut(dx.as_mut());
        let dy = gf256::slice_from_slice_mut(dy.as_mut());

        // find intermediate values
        //
        // p - Σ di
        //   i!=x,y
        // 
        // q - Σ di*g^i
        //   i!=x,y
        //
        for i in 0..len {
            dx[i] = p[i];
            dy[i] = q[i];
        }

        for (j, b) in before.iter_mut().enumerate()
            .chain((bad_blocks[0]+1..).zip(between.iter_mut()))
            .chain((bad_blocks[1]+1..).zip(after.iter_mut()))
        {
            let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
            for i in 0..len {
                dx[i] -= gf256(b.as_mut()[i]);
                dy[i] -= gf256(b.as_mut()[i]) * g;
            }
        }

        // find final dx/dy
        //
        //     (q - Σ di*g^i) - (p - Σ di)*g^y
        //        i!=x,y           i!=x,y
        // dx = -------------------------------
        //                g^x - g^y
        //
        // dy = p - Σ di - dx
        //        i!=x,y
        //
        let gx = gf256::GENERATOR.pow(u8::try_from(bad_blocks[0]).unwrap());
        let gy = gf256::GENERATOR.pow(u8::try_from(bad_blocks[1]).unwrap());
        for i in 0..len {
            let pdelta = dx[i];
            let qdelta = dy[i];
            dx[i] = (qdelta - pdelta*gy) / (gx - gy);
            dy[i] = pdelta - dx[i];
        }
    }

    if bad_blocks.iter().any(|x| *x == blocks.len()) {
        // regenerate p
        for i in 0..len {
            p[i] = gf256(0);
        }

        for b in blocks.iter_mut() {
            for i in 0..len {
                p[i] += gf256(b.as_mut()[i]);
            }
        }
    }

    if bad_blocks.iter().any(|x| *x == blocks.len()+1) {
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
    }

    Ok(())
}

/// Add a block to a RAID6 array
///
/// Note the block index must be unique in the array! This does not
/// update other block indices.
///
pub fn raid6_add(j: usize, new: &[u8], p: &mut [u8], q: &mut [u8]) {
    let len = p.len();
    let p = gf256::slice_from_slice_mut(p);
    let q = gf256::slice_from_slice_mut(q);

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
    let p = gf256::slice_from_slice_mut(p);
    let q = gf256::slice_from_slice_mut(q);

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
    let p = gf256::slice_from_slice_mut(p);
    let q = gf256::slice_from_slice_mut(q);

    let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
    for i in 0..len {
        // calculate new parity
        p[i] += gf256(new[i]) - gf256(old[i]);
        q[i] += (gf256(new[i]) - gf256(old[i])) * g;
    }
}


//// RAID7 ////

/// Format blocks with RAID7, aka three blocks of parity
pub fn raid7_format<B: AsRef<[u8]>>(blocks: &[B], p: &mut [u8], q: &mut [u8], r: &mut [u8]) {
    let len = p.len();
    assert!(q.len() == len);
    assert!(r.len() == len);
    assert!(blocks.iter().all(|b| b.as_ref().len() == len));
    assert!(blocks.len() <= 255);
    let p = gf256::slice_from_slice_mut(p);
    let q = gf256::slice_from_slice_mut(q);
    let r = gf256::slice_from_slice_mut(r);

    for i in 0..len {
        p[i] = gf256(0);
        q[i] = gf256(0);
        r[i] = gf256(0);
    }

    for (j, b) in blocks.iter().enumerate() {
        let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
        let h = g*g;
        for i in 0..len {
            p[i] += gf256(b.as_ref()[i]);
            q[i] += gf256(b.as_ref()[i]) * g;
            r[i] += gf256(b.as_ref()[i]) * h;
        }
    }
}

/// Repair up to three blocks of failure
pub fn raid7_repair<B: AsMut<[u8]>>(
    blocks: &mut [B],
    p: &mut [u8],
    q: &mut [u8],
    r: &mut [u8],
    bad_blocks: &[usize]
) -> Result<(), RaidError> {
    let len = p.len();
    let p = gf256::slice_from_slice_mut(p);
    let q = gf256::slice_from_slice_mut(q);
    let r = gf256::slice_from_slice_mut(r);

    if bad_blocks.len() > 3 {
        // can't repair
        return Err(RaidError::TooManyBadBlocks);
    }

    // sort the data blocks without alloc, this is only so we can split
    // the mut blocks array safely
    let mut bad_blocks_array = [
        bad_blocks.get(0).copied().unwrap_or(0),
        bad_blocks.get(1).copied().unwrap_or(0),
        bad_blocks.get(2).copied().unwrap_or(0),
    ];
    let bad_blocks = &mut bad_blocks_array[..bad_blocks.len()];
    bad_blocks.sort_unstable();

    if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 1
        && !bad_blocks.iter().any(|b| *b == blocks.len()+0)
    {
        // repair using p
        let (before, after) = blocks.split_at_mut(bad_blocks[0]);
        let (d, after) = after.split_first_mut().unwrap();
        let d = gf256::slice_from_slice_mut(d.as_mut());

        for i in 0..len {
            d[i] = p[i];
        }

        for b in before.iter_mut().chain(after.iter_mut()) {
            for i in 0..len {
                d[i] -= gf256(b.as_mut()[i]);
            }
        }
    } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 1
        && !bad_blocks.iter().any(|b| *b == blocks.len()+1)
    {
        // repair using q
        let (before, after) = blocks.split_at_mut(bad_blocks[0]);
        let (d, after) = after.split_first_mut().unwrap();
        let d = gf256::slice_from_slice_mut(d.as_mut());

        for i in 0..len {
            d[i] = q[i];
        }

        for (j, b) in before.iter_mut().enumerate()
            .chain((bad_blocks[0]+1..).zip(after.iter_mut()))
        {
            let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
            for i in 0..len {
                d[i] -= gf256(b.as_mut()[i]) * g;
            }
        }

        let g = gf256::GENERATOR.pow(u8::try_from(bad_blocks[0]).unwrap());
        for i in 0..len {
            d[i] /= g;
        }
    } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 1
        && !bad_blocks.iter().any(|b| *b == blocks.len()+2)
    {
        // repair using r
        let (before, after) = blocks.split_at_mut(bad_blocks[0]);
        let (d, after) = after.split_first_mut().unwrap();
        let d = gf256::slice_from_slice_mut(d.as_mut());

        for i in 0..len {
            d[i] = r[i];
        }

        for (j, b) in before.iter_mut().enumerate()
            .chain((bad_blocks[0]+1..).zip(after.iter_mut()))
        {
            let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
            let h = g*g;
            for i in 0..len {
                d[i] -= gf256(b.as_mut()[i]) * h;
            }
        }

        let g = gf256::GENERATOR.pow(u8::try_from(bad_blocks[0]).unwrap());
        let h = g*g;
        for i in 0..len {
            d[i] /= h;
        }
    } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 2
        && !bad_blocks.iter().any(|b| *b == blocks.len()+0 || *b == blocks.len()+1)
    {
        // repair dx and dy using p and q
        let (before, between) = blocks.split_at_mut(bad_blocks[0]);
        let (dx, between) = between.split_first_mut().unwrap();
        let (between, after) = between.split_at_mut(bad_blocks[1]-(bad_blocks[0]+1));
        let (dy, after) = after.split_first_mut().unwrap();
        let dx = gf256::slice_from_slice_mut(dx.as_mut());
        let dy = gf256::slice_from_slice_mut(dy.as_mut());

        // find intermediate values
        //
        // p - Σ di
        //   i!=x,y
        // 
        // q - Σ di*g^i
        //   i!=x,y
        //
        for i in 0..len {
            dx[i] = p[i];
            dy[i] = q[i];
        }

        for (j, b) in before.iter_mut().enumerate()
            .chain((bad_blocks[0]+1..).zip(between.iter_mut()))
            .chain((bad_blocks[1]+1..).zip(after.iter_mut()))
        {
            let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
            for i in 0..len {
                dx[i] -= gf256(b.as_mut()[i]);
                dy[i] -= gf256(b.as_mut()[i]) * g;
            }
        }

        // find final dx/dy
        //
        //      (q - Σ di*g^i) - (p - Σ di)*g^y
        //         i!=x,y           i!=x,y
        // dx = -------------------------------
        //               g^x - g^y
        //
        // dy = p - Σ di - dx
        //        i!=x,y
        //
        let gx = gf256::GENERATOR.pow(u8::try_from(bad_blocks[0]).unwrap());
        let gy = gf256::GENERATOR.pow(u8::try_from(bad_blocks[1]).unwrap());
        for i in 0..len {
            let pdelta = dx[i];
            let qdelta = dy[i];
            dx[i] = (qdelta - pdelta*gy) / (gx - gy);
            dy[i] = pdelta - dx[i];
        }
    } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 2
        && !bad_blocks.iter().any(|b| *b == blocks.len()+1 || *b == blocks.len()+2)
    {
        // repair dx and dy using q and r
        let (before, between) = blocks.split_at_mut(bad_blocks[0]);
        let (dx, between) = between.split_first_mut().unwrap();
        let (between, after) = between.split_at_mut(bad_blocks[1]-(bad_blocks[0]+1));
        let (dy, after) = after.split_first_mut().unwrap();
        let dx = gf256::slice_from_slice_mut(dx.as_mut());
        let dy = gf256::slice_from_slice_mut(dy.as_mut());

        // find intermediate values
        //
        // q - Σ di*g^i
        //   i!=x,y
        // 
        // r - Σ di*h^i
        //   i!=x,y
        //
        for i in 0..len {
            dx[i] = q[i];
            dy[i] = r[i];
        }

        for (j, b) in before.iter_mut().enumerate()
            .chain((bad_blocks[0]+1..).zip(between.iter_mut()))
            .chain((bad_blocks[1]+1..).zip(after.iter_mut()))
        {
            let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
            let h = g*g;
            for i in 0..len {
                dx[i] -= gf256(b.as_mut()[i]) * g;
                dy[i] -= gf256(b.as_mut()[i]) * h;
            }
        }

        // find final dx/dy
        //
        //      (r - Σ di*h^i) - (q - Σ di*g^i)*g^y
        //         i!=x,y           i!=x,y
        // dx = -----------------------------------
        //               g^x*(g^x - g^y)
        //
        //      q - Σ di*g^i - dx*g^x
        //        i!=x,y
        // dy = ---------------------
        //               g^y
        //
        let gx = gf256::GENERATOR.pow(u8::try_from(bad_blocks[0]).unwrap());
        let gy = gf256::GENERATOR.pow(u8::try_from(bad_blocks[1]).unwrap());
        for i in 0..len {
            let qdelta = dx[i];
            let rdelta = dy[i];
            dx[i] = (rdelta - qdelta*gy) / (gx*(gx - gy));
            dy[i] = (qdelta - dx[i]*gx) / gy;
        }
    } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 2
        && !bad_blocks.iter().any(|b| *b == blocks.len()+0 || *b == blocks.len()+2)
    {
        // repair dx and dy using p and r
        let (before, between) = blocks.split_at_mut(bad_blocks[0]);
        let (dx, between) = between.split_first_mut().unwrap();
        let (between, after) = between.split_at_mut(bad_blocks[1]-(bad_blocks[0]+1));
        let (dy, after) = after.split_first_mut().unwrap();
        let dx = gf256::slice_from_slice_mut(dx.as_mut());
        let dy = gf256::slice_from_slice_mut(dy.as_mut());

        // find intermediate values
        //
        // p - Σ di
        //   i!=x,y
        // 
        // r - Σ di*h^i
        //   i!=x,y
        //
        for i in 0..len {
            dx[i] = p[i];
            dy[i] = r[i];
        }

        for (j, b) in before.iter_mut().enumerate()
            .chain((bad_blocks[0]+1..).zip(between.iter_mut()))
            .chain((bad_blocks[1]+1..).zip(after.iter_mut()))
        {
            let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
            let h = g*g;
            for i in 0..len {
                dx[i] -= gf256(b.as_mut()[i]);
                dy[i] -= gf256(b.as_mut()[i]) * h;
            }
        }

        // find final dx/dy
        //
        //      (r - Σ di*h^i) - (p - Σ di)*h^y
        //         i!=x,y           i!=x,y
        // dx = -------------------------------
        //               h^x - h^y
        //
        // dy = p - Σ di - dx
        //        i!=x,y
        //
        let gx = gf256::GENERATOR.pow(u8::try_from(bad_blocks[0]).unwrap());
        let hx = gx*gx;
        let gy = gf256::GENERATOR.pow(u8::try_from(bad_blocks[1]).unwrap());
        let hy = gy*gy;
        for i in 0..len {
            let pdelta = dx[i];
            let rdelta = dy[i];
            dx[i] = (rdelta - pdelta*hy) / (hx - hy);
            dy[i] = pdelta - dx[i];
        }
    } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 3 {
        // repair dx, dy and dz using p, q and r
        let (before, between) = blocks.split_at_mut(bad_blocks[0]);
        let (dx, between) = between.split_first_mut().unwrap();
        let (between, between2) = between.split_at_mut(bad_blocks[1]-(bad_blocks[0]+1));
        let (dy, between2) = between2.split_first_mut().unwrap();
        let (between2, after) = between2.split_at_mut(bad_blocks[2]-(bad_blocks[1]+1));
        let (dz, after) = after.split_first_mut().unwrap(); 
        let dx = gf256::slice_from_slice_mut(dx.as_mut());
        let dy = gf256::slice_from_slice_mut(dy.as_mut());
        let dz = gf256::slice_from_slice_mut(dz.as_mut());

        // find intermediate values
        //
        // p - Σ di
        //  i!=x,y,z
        //
        // q - Σ di*g^i
        //  i!=x,y,z
        //
        // r - Σ di*h^i
        //  i!=x,y,z
        //
        for i in 0..len {
            dx[i] = p[i];
            dy[i] = q[i];
            dz[i] = r[i];
        }

        for (j, b) in before.iter_mut().enumerate()
            .chain((bad_blocks[0]+1..).zip(between.iter_mut()))
            .chain((bad_blocks[1]+1..).zip(between2.iter_mut()))
            .chain((bad_blocks[2]+1..).zip(after.iter_mut()))
        {
            let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
            let h = g*g;
            for i in 0..len {
                dx[i] -= gf256(b.as_mut()[i]);
                dy[i] -= gf256(b.as_mut()[i]) * g;
                dz[i] -= gf256(b.as_mut()[i]) * h;
            }
        }

        // find final dx/dy/dz
        //
        //      (r - Σ di*h^i) - (q - Σ di*g^i)*(g^y - g^z) - (p - Σ di)*g^y*g^z
        //        i!=x,y,z         i!=x,y,z                     i!=x,y,z
        // dx = ----------------------------------------------------------------
        //                      (g^x - g^y)*(g^x - g^z)
        //
        //      (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
        //        i!=x,y,z         i!=x,y,z
        // dy = ------------------------------------------------
        //                         g^y - g^z
        //
        // dz = p - Σ di - dx - dy
        //       i!=x,y,z
        //
        let gx = gf256::GENERATOR.pow(u8::try_from(bad_blocks[0]).unwrap());
        let gy = gf256::GENERATOR.pow(u8::try_from(bad_blocks[1]).unwrap());
        let gz = gf256::GENERATOR.pow(u8::try_from(bad_blocks[2]).unwrap());
        for i in 0..len {
            let pdelta = dx[i];
            let qdelta = dy[i];
            let rdelta = dz[i];
            dx[i] = (rdelta - qdelta*(gy - gz) - pdelta*gy*gz) / ((gx - gy) * (gx - gz));
            dy[i] = (qdelta - pdelta*gz - dx[i]*(gx - gz)) / (gy - gz);
            dz[i] = pdelta - dx[i] - dy[i];
        }
    }

    if bad_blocks.iter().any(|x| *x == blocks.len()) {
        // regenerate p
        for i in 0..len {
            p[i] = gf256(0);
        }

        for b in blocks.iter_mut() {
            for i in 0..len {
                p[i] += gf256(b.as_mut()[i]);
            }
        }
    }

    if bad_blocks.iter().any(|x| *x == blocks.len()+1) {
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
    }

    if bad_blocks.iter().any(|x| *x == blocks.len()+2) {
        // regenerate r
        for i in 0..len {
            r[i] = gf256(0);
        }

        for (j, b) in blocks.iter_mut().enumerate() {
            let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
            let h = g.pow(2);
            for i in 0..len {
                r[i] += gf256(b.as_mut()[i]) * h;
            }
        }
    }

    Ok(())
}

/// Add a block to a RAID7 array
///
/// Note the block index must be unique in the array! This does not
/// update other block indices.
///
pub fn raid7_add(j: usize, new: &[u8], p: &mut [u8], q: &mut [u8], r: &mut [u8]) {
    let len = p.len();
    let p = gf256::slice_from_slice_mut(p);
    let q = gf256::slice_from_slice_mut(q);
    let r = gf256::slice_from_slice_mut(r);

    let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
    let h = g*g;
    for i in 0..len {
        // calculate new parity
        p[i] += gf256(new[i]);
        q[i] += gf256(new[i]) * g;
        r[i] += gf256(new[i]) * h;
    }
}

/// Add a block from a RAID7 array
///
/// Note the block index must already exit in the array, otherwise the
/// array will become corrupted. This does not update other block indices.
///
pub fn raid7_remove(j: usize, old: &[u8], p: &mut [u8], q: &mut [u8], r: &mut [u8]) {
    let len = p.len();
    let p = gf256::slice_from_slice_mut(p);
    let q = gf256::slice_from_slice_mut(q);
    let r = gf256::slice_from_slice_mut(r);

    let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
    let h = g*g;
    for i in 0..len {
        // calculate new parity
        p[i] -= gf256(old[i]);
        q[i] -= gf256(old[i]) * g;
        r[i] -= gf256(old[i]) * h;
    }
}

/// Update a block in a RAID7 array
///
/// This is functionally equivalent to remove(i)+add(i), but more efficient.
///
pub fn raid7_update(j: usize, old: &[u8], new: &[u8], p: &mut [u8], q: &mut [u8], r: &mut [u8]) {
    let len = p.len();
    let p = gf256::slice_from_slice_mut(p);
    let q = gf256::slice_from_slice_mut(q);
    let r = gf256::slice_from_slice_mut(r);

    let g = gf256::GENERATOR.pow(u8::try_from(j).unwrap());
    let h = g*g;
    for i in 0..len {
        // calculate new parity
        p[i] += gf256(new[i]) - gf256(old[i]);
        q[i] += (gf256(new[i]) - gf256(old[i])) * g;
        r[i] += (gf256(new[i]) - gf256(old[i])) * h;
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


    // testing RAID5

    let mut blocks = [
        Vec::from(b"Hell".as_ref()),
        Vec::from(b"o Wo".as_ref()),
        Vec::from(b"rld?".as_ref()),
    ];
    let mut parity = Vec::from(b"    ".as_ref());

    println!();
    println!("testing raid5({:?})",
        blocks.iter()
            .map(|b| ascii(b))
            .collect::<String>()
    );

    raid5_format(&blocks, &mut parity);
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
    raid5_update(2, &[old], &[b'!'], &mut parity[3..4]);
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

    raid5_repair(&mut blocks, &mut parity, &bad_blocks).unwrap();
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


    // testing RAID7

    let mut blocks = [
        Vec::from(b"Hell".as_ref()),
        Vec::from(b"o Wo".as_ref()),
        Vec::from(b"rld?".as_ref()),
    ];
    let mut parity1 = Vec::from(b"    ".as_ref());
    let mut parity2 = Vec::from(b"    ".as_ref());
    let mut parity3 = Vec::from(b"    ".as_ref());

    println!();
    println!("testing raid7({:?})",
        blocks.iter()
            .map(|b| ascii(b))
            .collect::<String>()
    );

    raid7_format(&blocks, &mut parity1, &mut parity2, &mut parity3);
    println!("{:<7} => {}  {}",
        "format",
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| hex(b))
            .collect::<String>()
    );

    let old = blocks[2][3];
    blocks[2][3] = b'!';
    raid7_update(2, &[old], &[b'!'], &mut parity1[3..4], &mut parity2[3..4], &mut parity3[3..4]);
    println!("{:<7} => {}  {}",
        "update",
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| hex(b))
            .collect::<String>()
    );

    let mut rng = rand::thread_rng();
    let bad_blocks = rand::seq::index::sample(&mut rng, blocks.len()+3, 1).into_vec();
    for bad_block in bad_blocks.iter() {
        if *bad_block < blocks.len() {
            blocks[*bad_block].fill(b'x');
        } else if *bad_block == blocks.len()+0 {
            parity1.fill(b'x');
        } else if *bad_block == blocks.len()+1 {
            parity2.fill(b'x');
        } else if *bad_block == blocks.len()+2 {
            parity3.fill(b'x');
        } else {
            unreachable!();
        }
    }
    println!("{:<7} => {}  {}",
        "corrupt",
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| hex(b))
            .collect::<String>()
    );

    raid7_repair(&mut blocks, &mut parity1, &mut parity2, &mut parity3, &bad_blocks).unwrap();
    println!("{:<7} => {}  {}",
        "repair",
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
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
    let bad_blocks = rand::seq::index::sample(&mut rng, blocks.len()+3, 2).into_vec();
    for bad_block in bad_blocks.iter() {
        if *bad_block < blocks.len() {
            blocks[*bad_block].fill(b'x');
        } else if *bad_block == blocks.len()+0 {
            parity1.fill(b'x');
        } else if *bad_block == blocks.len()+1 {
            parity2.fill(b'x');
        } else if *bad_block == blocks.len()+2 {
            parity3.fill(b'x');
        } else {
            unreachable!();
        }
    }
    println!("{:<7} => {}  {}",
        "corrupt",
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| hex(b))
            .collect::<String>()
    );

    raid7_repair(&mut blocks, &mut parity1, &mut parity2, &mut parity3, &bad_blocks).unwrap();
    println!("{:<7} => {}  {}",
        "repair",
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
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
    let bad_blocks = rand::seq::index::sample(&mut rng, blocks.len()+3, 3).into_vec();
    for bad_block in bad_blocks.iter() {
        if *bad_block < blocks.len() {
            blocks[*bad_block].fill(b'x');
        } else if *bad_block == blocks.len()+0 {
            parity1.fill(b'x');
        } else if *bad_block == blocks.len()+1 {
            parity2.fill(b'x');
        } else if *bad_block == blocks.len()+2 {
            parity3.fill(b'x');
        } else {
            unreachable!();
        }
    }
    println!("{:<7} => {}  {}",
        "corrupt",
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| hex(b))
            .collect::<String>()
    );

    raid7_repair(&mut blocks, &mut parity1, &mut parity2, &mut parity3, &bad_blocks).unwrap();
    println!("{:<7} => {}  {}",
        "repair",
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
            .map(|b| ascii(b))
            .collect::<String>(),
        blocks.iter()
            .chain([&parity1, &parity2, &parity3])
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


    // for fun lets render a repaired image
    fn dots<'a>(width: usize, dots: &'a [u8]) -> impl Iterator<Item=String> + 'a {
        fn todots(d: u8) -> String {
            fn tod(d: u8) -> char {
                match d & 0x3 {
                    0x0 => ' ',
                    0x1 => '\'',
                    0x2 => '.',
                    0x3 => ':',
                    _ => unreachable!(),
                }
            }
            [tod(d >> 6), tod(d >> 4), tod(d >> 2), tod(d >> 0)]
                .iter()
                .collect::<String>()
        }

        (0..dots.len())
            .step_by(width)
            .map(move |d| {
                dots[d..d+width]
                    .iter()
                    .map(|d| todots(*d))
                    .collect::<String>()
            })
    }

    let width = 16;
    let image = [
        0x00, 0x00, 0x00, 0x00, 0x0a, 0xff, 0xff, 0xfe, 0xa0, 0x00, 0x00, 0x00, 0x00, 0xbf, 0xaa, 0xfe,
        0x00, 0x00, 0x00, 0x02, 0xff, 0xff, 0xff, 0xff, 0xfe, 0x80, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x2f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf8, 0x00, 0x00, 0x00, 0xff, 0xff, 0xf4,
        0x00, 0x00, 0x02, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x23, 0xff, 0xff, 0xc0,
        0x00, 0x00, 0x0b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc0, 0x2a, 0x3c, 0xdf, 0xff, 0x40,
        0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfd, 0x4a, 0x1f, 0xc4, 0x00, 0x54, 0x00,
        0x00, 0x00, 0x3f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x5a, 0xff, 0xc4, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x3f, 0xff, 0xff, 0xff, 0xff, 0xff, 0x52, 0xbf, 0xf5, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x3f, 0xff, 0xff, 0xff, 0xff, 0x52, 0xbf, 0xf5, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x2b, 0x3f, 0xff, 0xff, 0xfd, 0x52, 0xbf, 0xf5, 0x2b, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x0a, 0xfd, 0x0f, 0xff, 0xfd, 0x4a, 0xff, 0xf5, 0x2b, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x3f, 0x40, 0x07, 0xf5, 0x2a, 0xff, 0xf5, 0xab, 0xff, 0xff, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xf4, 0x00, 0x00, 0xab, 0xff, 0xd4, 0xaf, 0xff, 0xff, 0xfd, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x2a, 0xbf, 0xff, 0x52, 0xbf, 0xff, 0xff, 0xff, 0xf4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x7f, 0xff, 0xd5, 0x00, 0xff, 0xff, 0xff, 0xff, 0xfd, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x05, 0xff, 0xff, 0xfd, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    // store our image in blocks
    let block = 64;
    let columns = block / (image.len()/width);
    let mut blocks = (0..image.len()/block)
        .map(|i| {
            let mut data = (0 .. image.len()/width)
                .map(|j| {
                    image[i*columns+j*width..i*columns+j*width+columns].iter().copied()
                })
                .flatten()
                .collect::<Vec<u8>>();

            // make space for CRCs, this is one option for determining block failures
            //
            // a 32-bit CRC is probably overkill for 64 bytes of data, but it aligns
            // nicely with our image
            //
            data.resize(data.len()+4, 0);
            data
        })
        .collect::<Vec<Vec<u8>>>();
    blocks.resize_with(blocks.len()+3, || vec![0; block+4]);

    let mkcrcs = |blocks: &mut [Vec<u8>]| {
        for data in blocks {
            let data_len = data.len();
            let crc = crc32c(&data[..data_len-4], 0);
            data[data_len-4..].copy_from_slice(&crc.to_le_bytes());
        }
    };

    let chcrcs = |blocks: &[Vec<u8>]| -> Vec<usize> {
        let mut bad_blocks = vec![];
        for (i, data) in blocks.iter().enumerate() {
            let data_len = data.len();
            let crc = crc32c(&data[..data_len-4], 0);
            if crc != u32::from_le_bytes(<[u8; 4]>::try_from(&data[data_len-4..]).unwrap()) {
                bad_blocks.push(i);
            }
        }
        bad_blocks
    };

    let toimage = |blocks: &[Vec<u8>]| -> Vec<u8> {
        let mut image = vec![0; blocks[0].len()*blocks.len()];
        let image_len = image.len();
        for i in 0..blocks.len() {
            for j in 0..image_len/(width+3*columns) {
                image[i*columns + j*(width+3*columns)
                    .. i*columns + j*(width+3*columns) + columns]
                        .copy_from_slice(&blocks[i][j*columns .. (j+1)*columns]);
            }
        }
        image
    };

    // format with RAID 7
    {
        let (r, blocks) = blocks.split_last_mut().unwrap();
        let (q, blocks) = blocks.split_last_mut().unwrap();
        let (p, blocks) = blocks.split_last_mut().unwrap();
        raid7_format(blocks, p, q, r);
    }
    // update CRCs
    mkcrcs(&mut blocks);

    // randomly choose three blocks to damage, this is the worst case
    // we can recover from
    let mut rng = rand::thread_rng();
    let bad_blocks = rand::seq::index::sample(&mut rng, blocks.len(), 3).into_vec();
    for i in bad_blocks.iter().copied() {
        blocks[i].fill(0xff);
    }

    println!("block corrupted image (errors = {}/{}, {:.2}%):",
        3,
        blocks.len(),
        100.0 * (3.0 / (blocks.len() as f64)));
    println!();
    for line in dots(width+3*columns, &toimage(&blocks)) {
        println!("    {}", line);
    }
    println!();

    // find bad blocks by checking CRCs
    let bad_blocks = chcrcs(&blocks);
    // repair
    {
        let (r, blocks) = blocks.split_last_mut().unwrap();
        let (q, blocks) = blocks.split_last_mut().unwrap();
        let (p, blocks) = blocks.split_last_mut().unwrap();
        raid7_repair(blocks, p, q, r, &bad_blocks).unwrap();
    }
    // update CRCs
    mkcrcs(&mut blocks);

    println!("corrected:");
    println!();
    for line in dots(width+3*columns, &toimage(&blocks)) {
        println!("    {}", line);
    }
    println!();
}
