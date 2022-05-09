// Template for RAID-parity functions
//
// See examples/raid.rs for a more detailed explanation of
// where these implementations come from

//! RAID-parity functions.
//!
//! ``` rust
//! # use gf256::raid::raid7;
//! #
//! // format
//! let mut buf = b"Hello World!".to_vec();
//! let mut parity1 = vec![0u8; 4];
//! let mut parity2 = vec![0u8; 4];
//! let mut parity3 = vec![0u8; 4];
//! let slices = buf.chunks(4).collect::<Vec<_>>();
//! raid7::format(&slices, &mut parity1, &mut parity2, &mut parity3);
//!
//! // corrupt
//! buf[0..8].fill(b'x');
//!
//! // repair
//! let mut slices = buf.chunks_mut(4).collect::<Vec<_>>();
//! raid7::repair(&mut slices, &mut parity1, &mut parity2, &mut parity3, &[0, 1]);
//! assert_eq!(&buf, b"Hello World!");
//! ```
//!
//! See the [module-level documentation](../../raid) for more info.
//!


use __crate::internal::cfg_if::cfg_if;
use __crate::traits::TryFrom;
use __crate::traits::FromLossy;
use core::slice;
use core::cmp::min;
use core::cmp::max;
use core::fmt;


/// Error codes for RAID arrays
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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


/// Format blocks as a RAID array.
///
/// This writes the parity data to the provided parity blocks based on the
/// provided data blocks.
///
/// ``` rust
/// # use ::gf256::raid::*;
/// let mut data = b"Hello World!".to_vec();
/// let datas = data.chunks(4).collect::<Vec<_>>();
/// let mut parity1 = vec![0u8; 4];
/// let mut parity2 = vec![0u8; 4];
/// let mut parity3 = vec![0u8; 4];
/// raid7::format(&datas, &mut parity1, &mut parity2, &mut parity3);
///
/// assert_eq!(&datas[0], b"Hell");
/// assert_eq!(&datas[1], b"o Wo");
/// assert_eq!(&datas[2], b"rld!");
/// assert_eq!(&parity1,  b"\x55\x29\x5f\x22");
/// assert_eq!(&parity2,  b"\x43\x88\x4f\x36");
/// assert_eq!(&parity3,  b"\x9a\x6b\x23\xe7");
/// ```
///
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

/// Repair up to `n` bad blocks.
///
/// Where `n` <= the number of parity blocks. This can include the parity
/// blocks themselves. `bad_blocks` must be an array of indices indicating
/// which blocks are bad.
///
/// ``` rust
/// # use ::gf256::raid::*;
/// let mut data = b"Hellxxxxxxxx".to_vec();
/// let mut datas = data.chunks_mut(4).collect::<Vec<_>>();
/// let mut parity1 = b"xxxx".to_vec();
/// let mut parity2 = b"\x43\x88\x4f\x36".to_vec();
/// let mut parity3 = b"\x9a\x6b\x23\xe7".to_vec();
///
/// // repair
/// raid7::repair(&mut datas, &mut parity1, &mut parity2, &mut parity3, &[1, 2, 3]);
/// assert_eq!(&data, b"Hello World!");
/// ```
///
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

/// Add a block to a RAID array.
///
/// Note the block index must be unique in the array, otherwise the array will
/// become corrupted. This does not update other block indices.
///
/// ``` rust
/// # use ::gf256::raid::*;
/// let mut data = b"xxxxo World!".to_vec();
/// let mut datas = data.chunks_mut(4).collect::<Vec<_>>();
/// let mut parity1 = b"\x1d\x4c\x33\x4e".to_vec();
/// let mut parity2 = b"\x0b\xed\x23\x5a".to_vec();
/// let mut parity3 = b"\xd2\x0e\x4f\x8b".to_vec();
///
/// // add
/// let new_data = b"Jell";
/// raid7::add(0, new_data, &mut parity1, &mut parity2, &mut parity3);
/// datas[0].copy_from_slice(new_data);
///
/// assert_eq!(&datas[0], b"Jell");
/// assert_eq!(&datas[1], b"o Wo");
/// assert_eq!(&datas[2], b"rld!");
/// assert_eq!(&parity1,  b"\x57\x29\x5f\x22");
/// assert_eq!(&parity2,  b"\x41\x88\x4f\x36");
/// assert_eq!(&parity3,  b"\x98\x6b\x23\xe7");
/// ```
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

/// Remove a block from a RAID array.
///
/// Note the block index must already exist in the array, otherwise the
/// array will become corrupted. This does not update other block indices.
///
/// ``` rust
/// # use ::gf256::raid::*;
/// let mut data = b"Hello World!".to_vec();
/// let mut datas = data.chunks_mut(4).collect::<Vec<_>>();
/// let mut parity1 = b"\x55\x29\x5f\x22".to_vec();
/// let mut parity2 = b"\x43\x88\x4f\x36".to_vec();
/// let mut parity3 = b"\x9a\x6b\x23\xe7".to_vec();
///
/// // remove 
/// raid7::remove(0, datas[0], &mut parity1, &mut parity2, &mut parity3);
///
/// assert_eq!(&datas[1], b"o Wo");
/// assert_eq!(&datas[2], b"rld!");
/// assert_eq!(&parity1,  b"\x1d\x4c\x33\x4e");
/// assert_eq!(&parity2,  b"\x0b\xed\x23\x5a");
/// assert_eq!(&parity3,  b"\xd2\x0e\x4f\x8b");
/// ```
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

/// Update a block in a RAID array.
///
/// ``` rust
/// # use ::gf256::raid::*;
/// let mut data = b"Hello World!".to_vec();
/// let mut datas = data.chunks_mut(4).collect::<Vec<_>>();
/// let mut parity1 = b"\x55\x29\x5f\x22".to_vec();
/// let mut parity2 = b"\x43\x88\x4f\x36".to_vec();
/// let mut parity3 = b"\x9a\x6b\x23\xe7".to_vec();
///
/// // update
/// let new_data = b"Jell";
/// raid7::update(0, datas[0], new_data, &mut parity1, &mut parity2, &mut parity3);
/// datas[0].copy_from_slice(new_data);
///
/// assert_eq!(&datas[0], b"Jell");
/// assert_eq!(&datas[1], b"o Wo");
/// assert_eq!(&datas[2], b"rld!");
/// assert_eq!(&parity1,  b"\x57\x29\x5f\x22");
/// assert_eq!(&parity2,  b"\x41\x88\x4f\x36");
/// assert_eq!(&parity3,  b"\x98\x6b\x23\xe7");
/// ```
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

