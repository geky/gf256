//! Template for RAID-parity structs
//!
//! See examples/raid.rs for a more detailed explanation of
//! where these implementations come from

use __crate::internal::cfg_if::cfg_if;
use __crate::traits::TryFrom;
use __crate::traits::FromLossy;
use core::slice;
use core::cmp::min;
use core::cmp::max;
use core::fmt;


/// Error codes for RAID arrays
#[derive(Debug, Clone)]
pub enum Error {
    /// RAID-parity can fail to decode if there are more bad-blocks
    /// than there are parity blocks
    ///
    TooManyBadBlocks,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TooManyBadBlocks => write!(f, "Too many bad-blocks to repair"),
        }
    }
}


// TODO
// /// Format blocks with RAID4, aka single block of parity
/// Format blocks with RAID6, aka two blocks of parity
pub fn format<B: AsRef<[__u]>>(
    blocks: &[B],
    #[cfg(__if(__parity >= 1))] p: &mut [__u],
    #[cfg(__if(__parity >= 2))] q: &mut [__u],
    #[cfg(__if(__parity >= 3))] r: &mut [__u],
) {
    assert!(blocks.len() >= 1);
    #[cfg(__if(__parity >= 2))] { assert!(blocks.len() <= usize::try_from(__gf::NONZEROS).unwrap_or(usize::MAX)); }

    let len = blocks[0].as_ref().len();
    assert!(blocks.iter().all(|b| b.as_ref().len() == len));
    #[cfg(__if(__parity >= 1))] { assert!(p.len() == len); }
    #[cfg(__if(__parity >= 1))] let p = unsafe { __gf::slice_from_slice_mut_unchecked(p) };
    #[cfg(__if(__parity >= 2))] { assert!(q.len() == len); }
    #[cfg(__if(__parity >= 2))] let q = unsafe { __gf::slice_from_slice_mut_unchecked(q) };
    #[cfg(__if(__parity >= 3))] { assert!(r.len() == len); }
    #[cfg(__if(__parity >= 3))] let r = unsafe { __gf::slice_from_slice_mut_unchecked(r) };

    for i in 0..len {
        #[cfg(__if(__parity >= 1))] { p[i] = __gf::new(0); }
        #[cfg(__if(__parity >= 2))] { q[i] = __gf::new(0); }
        #[cfg(__if(__parity >= 3))] { r[i] = __gf::new(0); }
    }

    for (j, b) in blocks.iter().enumerate() {
        #[cfg(__if(__parity >= 2))] let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
        #[cfg(__if(__parity >= 3))] let h = g*g;
        for i in 0..len {
            #[cfg(__if(__parity >= 1))] { p[i] += __gf::from_lossy(b.as_ref()[i]); }
            #[cfg(__if(__parity >= 2))] { q[i] += __gf::from_lossy(b.as_ref()[i]) * g; }
            #[cfg(__if(__parity >= 3))] { r[i] += __gf::from_lossy(b.as_ref()[i]) * h; }
        }
    }
}

// TODO
// /// Repair up to one block of failure
/// Repair up to two blocks of failure
pub fn repair<B: AsMut<[__u]>>(
    blocks: &mut [B],
    #[cfg(__if(__parity >= 1))] p: &mut [__u],
    #[cfg(__if(__parity >= 2))] q: &mut [__u],
    #[cfg(__if(__parity >= 3))] r: &mut [__u],
    bad_blocks: &[usize]
) -> Result<(), Error> {
    let len = blocks[0].as_mut().len();
    #[cfg(__if(__parity >= 1))] let p = unsafe { __gf::slice_from_slice_mut_unchecked(p) };
    #[cfg(__if(__parity >= 2))] let q = unsafe { __gf::slice_from_slice_mut_unchecked(q) };
    #[cfg(__if(__parity >= 3))] let r = unsafe { __gf::slice_from_slice_mut_unchecked(r) };

    if bad_blocks.len() > __parity {
        // can't repair
        return Err(Error::TooManyBadBlocks);
    }


// TODO try removing with cfg_if if parity < 2?

    // sort the data blocks without alloc, this is only so we can split
    // the mut blocks array safely
    let mut bad_blocks_array = [
        bad_blocks.get(0).copied().unwrap_or(0),
        bad_blocks.get(1).copied().unwrap_or(0),
        bad_blocks.get(2).copied().unwrap_or(0),
    ];
    let mut bad_blocks = &mut bad_blocks_array[..bad_blocks.len()];
    bad_blocks.sort_unstable();

    #[cfg(__if(__parity >= 1))] {
        if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 1
            && !bad_blocks.iter().any(|b| *b == blocks.len()+0)
        {
            // repair using p
            let (before, after) = blocks.split_at_mut(bad_blocks[0]);
            let (d, after) = after.split_first_mut().unwrap();
            let d = unsafe { __gf::slice_from_slice_mut_unchecked(d.as_mut()) };

            for i in 0..len {
                d[i] = p[i];
            }

            for b in before.iter_mut().chain(after.iter_mut()) {
                for i in 0..len {
                    d[i] -= __gf::from_lossy(b.as_mut()[i]);
                }
            }

            bad_blocks = &mut bad_blocks[1..];
        }
    }

    #[cfg(__if(__parity >= 2))] {
        if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 1
            && !bad_blocks.iter().any(|b| *b == blocks.len()+1)
        {
            // repair using q
            let (before, after) = blocks.split_at_mut(bad_blocks[0]);
            let (d, after) = after.split_first_mut().unwrap();
            let d = unsafe { __gf::slice_from_slice_mut_unchecked(d.as_mut()) };

            for i in 0..len {
                d[i] = q[i];
            }

            for (j, b) in before.iter_mut().enumerate()
                .chain((bad_blocks[0]+1..).zip(after.iter_mut()))
            {
                let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
                for i in 0..len {
                    d[i] -= __gf::from_lossy(b.as_mut()[i]) * g;
                }
            }

            let g = __gf::GENERATOR.pow(__u::try_from(bad_blocks[0]).unwrap());
            for i in 0..len {
                d[i] /= g;
            }

            bad_blocks = &mut bad_blocks[1..];
        } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 2
            && !bad_blocks.iter().any(|b| *b == blocks.len()+0 || *b == blocks.len()+1)
        {
            // repair dx and dy using p and q
            let (before, between) = blocks.split_at_mut(bad_blocks[0]);
            let (dx, between) = between.split_first_mut().unwrap();
            let (between, after) = between.split_at_mut(bad_blocks[1]-(bad_blocks[0]+1));
            let (dy, after) = after.split_first_mut().unwrap();
            let dx = unsafe { __gf::slice_from_slice_mut_unchecked(dx.as_mut()) };
            let dy = unsafe { __gf::slice_from_slice_mut_unchecked(dy.as_mut()) };

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
                let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
                for i in 0..len {
                    dx[i] -= __gf::from_lossy(b.as_mut()[i]);
                    dy[i] -= __gf::from_lossy(b.as_mut()[i]) * g;
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
            let gx = __gf::GENERATOR.pow(__u::try_from(bad_blocks[0]).unwrap());
            let gy = __gf::GENERATOR.pow(__u::try_from(bad_blocks[1]).unwrap());
            for i in 0..len {
                let pdelta = dx[i];
                let qdelta = dy[i];
                dx[i] = (qdelta - pdelta*gy) / (gx - gy);
                dy[i] = pdelta - dx[i];
            }

            bad_blocks = &mut bad_blocks[2..];
        }
    }

    #[cfg(__if(__parity >= 3))] {
        if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 1
            && !bad_blocks.iter().any(|b| *b == blocks.len()+2)
        {
            // repair using r
            let (before, after) = blocks.split_at_mut(bad_blocks[0]);
            let (d, after) = after.split_first_mut().unwrap();
            let d = unsafe { __gf::slice_from_slice_mut_unchecked(d.as_mut()) };

            for i in 0..len {
                d[i] = r[i];
            }

            for (j, b) in before.iter_mut().enumerate()
                .chain((bad_blocks[0]+1..).zip(after.iter_mut()))
            {
                let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
                let h = g*g;
                for i in 0..len {
                    d[i] -= __gf::from_lossy(b.as_mut()[i]) * h;
                }
            }

            let g = __gf::GENERATOR.pow(__u::try_from(bad_blocks[0]).unwrap());
            let h = g*g;
            for i in 0..len {
                d[i] /= h;
            }

            bad_blocks = &mut bad_blocks[1..];
        } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 2
            && !bad_blocks.iter().any(|b| *b == blocks.len()+1 || *b == blocks.len()+2)
        {
            // repair dx and dy using q and r
            let (before, between) = blocks.split_at_mut(bad_blocks[0]);
            let (dx, between) = between.split_first_mut().unwrap();
            let (between, after) = between.split_at_mut(bad_blocks[1]-(bad_blocks[0]+1));
            let (dy, after) = after.split_first_mut().unwrap();
            let dx = unsafe { __gf::slice_from_slice_mut_unchecked(dx.as_mut()) };
            let dy = unsafe { __gf::slice_from_slice_mut_unchecked(dy.as_mut()) };

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
                let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
                let h = g*g;
                for i in 0..len {
                    dx[i] -= __gf::from_lossy(b.as_mut()[i]) * g;
                    dy[i] -= __gf::from_lossy(b.as_mut()[i]) * h;
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
            let gx = __gf::GENERATOR.pow(__u::try_from(bad_blocks[0]).unwrap());
            let gy = __gf::GENERATOR.pow(__u::try_from(bad_blocks[1]).unwrap());
            for i in 0..len {
                let qdelta = dx[i];
                let rdelta = dy[i];
                dx[i] = (rdelta - qdelta*gy) / (gx*(gx - gy));
                dy[i] = (qdelta - dx[i]*gx) / gy;
            }

            bad_blocks = &mut bad_blocks[2..];
        } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 2
            && !bad_blocks.iter().any(|b| *b == blocks.len()+0 || *b == blocks.len()+2)
        {
            // repair dx and dy using p and r
            let (before, between) = blocks.split_at_mut(bad_blocks[0]);
            let (dx, between) = between.split_first_mut().unwrap();
            let (between, after) = between.split_at_mut(bad_blocks[1]-(bad_blocks[0]+1));
            let (dy, after) = after.split_first_mut().unwrap();
            let dx = unsafe { __gf::slice_from_slice_mut_unchecked(dx.as_mut()) };
            let dy = unsafe { __gf::slice_from_slice_mut_unchecked(dy.as_mut()) };

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
                let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
                let h = g*g;
                for i in 0..len {
                    dx[i] -= __gf::from_lossy(b.as_mut()[i]);
                    dy[i] -= __gf::from_lossy(b.as_mut()[i]) * h;
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
            let gx = __gf::GENERATOR.pow(__u::try_from(bad_blocks[0]).unwrap());
            let hx = gx*gx;
            let gy = __gf::GENERATOR.pow(__u::try_from(bad_blocks[1]).unwrap());
            let hy = gy*gy;
            for i in 0..len {
                let pdelta = dx[i];
                let rdelta = dy[i];
                dx[i] = (rdelta - pdelta*hy) / (hx - hy);
                dy[i] = pdelta - dx[i];
            }

            bad_blocks = &mut bad_blocks[2..];
        } else if bad_blocks.iter().filter(|b| **b < blocks.len()).count() == 3 {
            // repair dx, dy and dz using p, q and r
            let (before, between) = blocks.split_at_mut(bad_blocks[0]);
            let (dx, between) = between.split_first_mut().unwrap();
            let (between, between2) = between.split_at_mut(bad_blocks[1]-(bad_blocks[0]+1));
            let (dy, between2) = between2.split_first_mut().unwrap();
            let (between2, after) = between2.split_at_mut(bad_blocks[2]-(bad_blocks[1]+1));
            let (dz, after) = after.split_first_mut().unwrap();
            let dx = unsafe { __gf::slice_from_slice_mut_unchecked(dx.as_mut()) };
            let dy = unsafe { __gf::slice_from_slice_mut_unchecked(dy.as_mut()) };
            let dz = unsafe { __gf::slice_from_slice_mut_unchecked(dz.as_mut()) };

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
                let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
                let h = g*g;
                for i in 0..len {
                    dx[i] -= __gf::from_lossy(b.as_mut()[i]);
                    dy[i] -= __gf::from_lossy(b.as_mut()[i]) * g;
                    dz[i] -= __gf::from_lossy(b.as_mut()[i]) * h;
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
            let gx = __gf::GENERATOR.pow(__u::try_from(bad_blocks[0]).unwrap());
            let gy = __gf::GENERATOR.pow(__u::try_from(bad_blocks[1]).unwrap());
            let gz = __gf::GENERATOR.pow(__u::try_from(bad_blocks[2]).unwrap());
            for i in 0..len {
                let pdelta = dx[i];
                let qdelta = dy[i];
                let rdelta = dz[i];
                dx[i] = (rdelta - qdelta*(gy - gz) - pdelta*gy*gz) / ((gx - gy)*(gx - gz));
                dy[i] = (qdelta - pdelta*gz - dx[i]*(gx - gz)) / (gy - gz);
                dz[i] = pdelta - dx[i] - dy[i];
            }

            bad_blocks = &mut bad_blocks[3..];
        }
    }

    #[cfg(__if(__parity >= 1))] {
        if bad_blocks.iter().any(|x| *x == blocks.len()) {
            // regenerate p
            for i in 0..len {
                p[i] = __gf::new(0);
            }

            for b in blocks.iter_mut() {
                for i in 0..len {
                    p[i] += __gf::from_lossy(b.as_mut()[i]);
                }
            }
        }
    }

    #[cfg(__if(__parity >= 2))] {
        if bad_blocks.iter().any(|x| *x == blocks.len()+1) {
            // regenerate q
            for i in 0..len {
                q[i] = __gf::new(0);
            }

            for (j, b) in blocks.iter_mut().enumerate() {
                let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
                for i in 0..len {
                    q[i] += __gf::from_lossy(b.as_mut()[i]) * g;
                }
            }
        }
    }

    #[cfg(__if(__parity >= 3))] {
        if bad_blocks.iter().any(|x| *x == blocks.len()+2) {
            // regenerate r
            for i in 0..len {
                r[i] = __gf::new(0);
            }

            for (j, b) in blocks.iter_mut().enumerate() {
                let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
                let h = g.pow(2);
                for i in 0..len {
                    r[i] += __gf::from_lossy(b.as_mut()[i]) * h;
                }
            }
        }
    }

    Ok(())
}

/// Add a block to a RAID array
///
/// Note the block index must be unique in the array! This does not
/// update other block indices.
///
pub fn add(
    j: usize,
    new: &[__u],
    #[cfg(__if(__parity >= 1))] p: &mut [__u],
    #[cfg(__if(__parity >= 2))] q: &mut [__u],
    #[cfg(__if(__parity >= 3))] r: &mut [__u],
) {
    let len = new.len();
    #[cfg(__if(__parity >= 1))] let p = unsafe { __gf::slice_from_slice_mut_unchecked(p) };
    #[cfg(__if(__parity >= 2))] let q = unsafe { __gf::slice_from_slice_mut_unchecked(q) };
    #[cfg(__if(__parity >= 3))] let r = unsafe { __gf::slice_from_slice_mut_unchecked(r) };

    #[cfg(__if(__parity >= 2))] let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
    #[cfg(__if(__parity >= 3))] let h = g*g;
    for i in 0..len {
        // calculate new parity
        #[cfg(__if(__parity >= 1))] { p[i] += __gf::from_lossy(new[i]); }
        #[cfg(__if(__parity >= 2))] { q[i] += __gf::from_lossy(new[i]) * g; }
        #[cfg(__if(__parity >= 3))] { r[i] += __gf::from_lossy(new[i]) * h; }
    }
}

/// Add a block from a RAID array
///
/// Note the block index must already exit in the array, otherwise the
/// array will become corrupted. This does not update other block indices.
///
pub fn remove(
    j: usize,
    old: &[__u],
    #[cfg(__if(__parity >= 1))] p: &mut [__u],
    #[cfg(__if(__parity >= 2))] q: &mut [__u],
    #[cfg(__if(__parity >= 3))] r: &mut [__u],
) {
    let len = old.len();
    #[cfg(__if(__parity >= 1))] let p = unsafe { __gf::slice_from_slice_mut_unchecked(p) };
    #[cfg(__if(__parity >= 2))] let q = unsafe { __gf::slice_from_slice_mut_unchecked(q) };
    #[cfg(__if(__parity >= 3))] let r = unsafe { __gf::slice_from_slice_mut_unchecked(r) };

    #[cfg(__if(__parity >= 2))] let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
    #[cfg(__if(__parity >= 3))] let h = g*g;
    for i in 0..len {
        // calculate new parity
        #[cfg(__if(__parity >= 1))] { p[i] -= __gf::from_lossy(old[i]); }
        #[cfg(__if(__parity >= 2))] { q[i] -= __gf::from_lossy(old[i]) * g; }
        #[cfg(__if(__parity >= 3))] { r[i] -= __gf::from_lossy(old[i]) * h; }
    }
}

/// Update a block in a RAID array
///
/// This is functionally equivalent to remove(i)+add(i), but more efficient.
///
pub fn update(
    j: usize,
    old: &[__u],
    new: &[__u],
    #[cfg(__if(__parity >= 1))] p: &mut [__u],
    #[cfg(__if(__parity >= 2))] q: &mut [__u],
    #[cfg(__if(__parity >= 3))] r: &mut [__u],
) {
    let len = old.len();
    assert!(new.len() == old.len());
    #[cfg(__if(__parity >= 1))] let p = unsafe { __gf::slice_from_slice_mut_unchecked(p) };
    #[cfg(__if(__parity >= 2))] let q = unsafe { __gf::slice_from_slice_mut_unchecked(q) };
    #[cfg(__if(__parity >= 3))] let r = unsafe { __gf::slice_from_slice_mut_unchecked(r) };

    #[cfg(__if(__parity >= 2))] let g = __gf::GENERATOR.pow(__u::try_from(j).unwrap());
    #[cfg(__if(__parity >= 3))] let h = g*g;
    for i in 0..len {
        // calculate new parity
        #[cfg(__if(__parity >= 1))] { p[i] += (__gf::from_lossy(new[i])-__gf::from_lossy(old[i])); }
        #[cfg(__if(__parity >= 2))] { q[i] += (__gf::from_lossy(new[i])-__gf::from_lossy(old[i])) * g; }
        #[cfg(__if(__parity >= 3))] { r[i] += (__gf::from_lossy(new[i])-__gf::from_lossy(old[i])) * h; }
    }
}

