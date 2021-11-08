//! Template for RAID-parity structs
//!
//! See examples/raid.rs for a more detailed explanation of
//! where these implementations come from

use __crate::internal::cfg_if::cfg_if;
use __crate::traits::TryFrom;
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
pub fn format<B: AsRef<[u8]>>(
    blocks: &[B],
    #[cfg(__if(__parity >= 1))] p: &mut [u8],
    #[cfg(__if(__parity >= 2))] q: &mut [u8],
) {
    cfg_if! {
        if #[cfg(__if(__parity == 0))] {
            // do nothing
        } else if #[cfg(__if(__parity == 1))] {
            let len = p.len();
            assert!(blocks.iter().all(|b| b.as_ref().len() == len));
            let p = __gf::slice_from_slice_mut(p);

            for i in 0..len {
                p[i] = __gf(0);
            }

            for b in blocks {
                for i in 0..len {
                    p[i] += __gf(b.as_ref()[i]);
                }
            }
        } else if #[cfg(__if(__parity == 2))] {
            let len = p.len();
            assert!(q.len() == len);
            assert!(blocks.iter().all(|b| b.as_ref().len() == len));
            assert!(blocks.len() <= 255);
            let p = __gf::slice_from_slice_mut(p);
            let q = __gf::slice_from_slice_mut(q);

            for i in 0..len {
                p[i] = __gf(0);
                q[i] = __gf(0);
            }

            for (j, b) in blocks.iter().enumerate() {
                let g = __gf::GENERATOR.pow(u8::try_from(j).unwrap());
                for i in 0..len {
                    p[i] += __gf(b.as_ref()[i]);
                    q[i] += __gf(b.as_ref()[i]) * g;
                }
            }
        }
    }
}

// TODO
// /// Repair up to one block of failure
/// Repair up to two blocks of failure
pub fn repair<B: AsMut<[u8]>>(
    blocks: &mut [B],
    #[cfg(__if(__parity >= 1))] p: &mut [u8],
    #[cfg(__if(__parity >= 2))] q: &mut [u8],
    bad_blocks: &[usize]
) -> Result<(), Error> {
    cfg_if! {
        if #[cfg(__if(__parity == 0))] {
            if bad_blocks.len() == 0 {
                // no repair needed
                Ok(())
            } else {
                // can't repair
                Err(Error::TooManyBadBlocks)
            }
        } else if #[cfg(__if(__parity == 1))] {
            let len = p.len();
            let p = __gf::slice_from_slice_mut(p);

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
                    let d = __gf::slice_from_slice_mut(d.as_mut());

                    for i in 0..len {
                        d[i] = p[i];
                    }

                    for b in before.iter_mut().chain(after.iter_mut()) {
                        for i in 0..len {
                            d[i] -= __gf(b.as_mut()[i]);
                        }
                    }
                    Ok(())
                } else if bad_block == blocks.len()+0 {
                    // regenerate p
                    for i in 0..len {
                        p[i] = __gf(0);
                    }

                    for b in blocks.iter_mut() {
                        for i in 0..len {
                            p[i] += __gf(b.as_mut()[i]);
                        }
                    }
                    Ok(())
                } else {
                    unreachable!()
                }
            } else {
                // can't repair
                Err(Error::TooManyBadBlocks)
            }
        } else if #[cfg(__if(__parity == 2))] {
            let len = p.len();
            let p = __gf::slice_from_slice_mut(p);
            let q = __gf::slice_from_slice_mut(q);

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
                    let d = __gf::slice_from_slice_mut(d.as_mut());

                    for i in 0..len {
                        d[i] = p[i];
                    }

                    for b in before.iter_mut().chain(after.iter_mut()) {
                        for i in 0..len {
                            d[i] -= __gf(b.as_mut()[i]);
                        }
                    }
                    Ok(())
                } else if bad_block == blocks.len()+0 {
                    // regenerate p
                    for i in 0..len {
                        p[i] = __gf(0);
                    }

                    for b in blocks.iter_mut() {
                        for i in 0..len {
                            p[i] += __gf(b.as_mut()[i]);
                        }
                    }
                    Ok(())
                } else if bad_block == blocks.len()+1 {
                    // regenerate q
                    for i in 0..len {
                        q[i] = __gf(0);
                    }

                    for (j, b) in blocks.iter_mut().enumerate() {
                        let g = __gf::GENERATOR.pow(u8::try_from(j).unwrap());
                        for i in 0..len {
                            q[i] += __gf(b.as_mut()[i]) * g;
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
                    let d1 = __gf::slice_from_slice_mut(d1.as_mut());
                    let d2 = __gf::slice_from_slice_mut(d2.as_mut());

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
                        let g = __gf::GENERATOR.pow(u8::try_from(j).unwrap());
                        for i in 0..len {
                            d1[i] -= __gf(b.as_mut()[i]);
                            d2[i] -= __gf(b.as_mut()[i]) * g;
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
                    let g1 = __gf::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
                    let g2 = __gf::GENERATOR.pow(u8::try_from(bad_block2).unwrap());
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
                    let d = __gf::slice_from_slice_mut(d.as_mut());

                    for i in 0..len {
                        d[i] = q[i];
                        p[i] = __gf(0);
                    }

                    for (j, b) in before.iter_mut().enumerate()
                        .chain((bad_block1+1..).zip(after.iter_mut()))
                    {
                        let g = __gf::GENERATOR.pow(u8::try_from(j).unwrap());
                        for i in 0..len {
                            d[i] -= __gf(b.as_mut()[i]) * g;
                            p[i] += __gf(b.as_mut()[i]);
                        }
                    }

                    // update p and d based on final value of d
                    let g = __gf::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
                    for i in 0..len {
                        d[i] /= g;
                        p[i] += d[i];
                    }
                    Ok(())
                } else if bad_block1 < blocks.len() && bad_block2 == blocks.len()+1 {
                    // repair d using p, and then regenerate q
                    let (before, after) = blocks.split_at_mut(bad_block1);
                    let (d, after) = after.split_first_mut().unwrap();
                    let d = __gf::slice_from_slice_mut(d.as_mut());

                    for i in 0..len {
                        d[i] = p[i];
                        q[i] = __gf(0);
                    }

                    for (j, b) in before.iter_mut().enumerate()
                        .chain((bad_block1+1..).zip(after.iter_mut()))
                    {
                        let g = __gf::GENERATOR.pow(u8::try_from(j).unwrap());
                        for i in 0..len {
                            d[i] -= __gf(b.as_mut()[i]);
                            q[i] += __gf(b.as_mut()[i]) * g;
                        }
                    }

                    // update q based on final value of d
                    let g = __gf::GENERATOR.pow(u8::try_from(bad_block1).unwrap());
                    for i in 0..len {
                        q[i] += d[i] * g;
                    }
                    Ok(())
                } else if bad_block1 == blocks.len()+0 && bad_block2 == blocks.len()+1 {
                    // regenerate p and q
                    for i in 0..len {
                        p[i] = __gf(0);
                        q[i] = __gf(0);
                    }

                    for (j, b) in blocks.iter_mut().enumerate() {
                        let g = __gf::GENERATOR.pow(u8::try_from(j).unwrap());
                        for i in 0..len {
                            p[i] += __gf(b.as_mut()[i]);
                            q[i] += __gf(b.as_mut()[i]) * g;
                        }
                    }
                    Ok(())
                } else {
                    unreachable!()
                }
            } else {
                // can't repair
                Err(Error::TooManyBadBlocks)
            }
        }
    }
}

/// Add a block to a RAID array
///
/// Note the block index must be unique in the array! This does not
/// update other block indices.
///
pub fn add(
    j: usize,
    new: &[u8],
    #[cfg(__if(__parity >= 1))] p: &mut [u8],
    #[cfg(__if(__parity >= 2))] q: &mut [u8],
) {
    cfg_if! {
        if #[cfg(__if(__parity == 0))] {
            // do nothing
        } else if #[cfg(__if(__parity == 1))] {
            let len = p.len();
            let p = __gf::slice_from_slice_mut(p);

            for i in 0..len {
                // calculate new parity
                p[i] += __gf(new[i]);
            }
        } else if #[cfg(__if(__parity == 2))] {
            let len = p.len();
            let p = __gf::slice_from_slice_mut(p);
            let q = __gf::slice_from_slice_mut(q);

            let g = __gf::GENERATOR.pow(u8::try_from(j).unwrap());
            for i in 0..len {
                // calculate new parity
                p[i] += __gf(new[i]);
                q[i] += __gf(new[i]) * g;
            }
        }
    }
}

/// Add a block from a RAID array
///
/// Note the block index must already exit in the array, otherwise the
/// array will become corrupted. This does not update other block indices.
///
pub fn remove(
    j: usize,
    old: &[u8],
    #[cfg(__if(__parity >= 1))] p: &mut [u8],
    #[cfg(__if(__parity >= 2))] q: &mut [u8],
) {
    cfg_if! {
        if #[cfg(__if(__parity == 0))] {
            // do nothing
        } else if #[cfg(__if(__parity == 1))] {
            let len = p.len();
            let p = __gf::slice_from_slice_mut(p);

            for i in 0..len {
                // calculate new parity
                p[i] -= __gf(old[i]);
            }
        } else if #[cfg(__if(__parity == 2))] {
            let len = p.len();
            let p = __gf::slice_from_slice_mut(p);
            let q = __gf::slice_from_slice_mut(q);

            let g = __gf::GENERATOR.pow(u8::try_from(j).unwrap());
            for i in 0..len {
                // calculate new parity
                p[i] -= __gf(old[i]);
                q[i] -= __gf(old[i]) * g;
            }
        }
    }
}

/// Update a block in a RAID array
///
/// This is functionally equivalent to remove(i)+add(i), but more efficient.
///
pub fn update(
    j: usize,
    old: &[u8],
    new: &[u8],
    #[cfg(__if(__parity >= 1))] p: &mut [u8],
    #[cfg(__if(__parity >= 2))] q: &mut [u8],
) {
    cfg_if! {
        if #[cfg(__if(__parity == 0))] {
            // do nothing
        } else if #[cfg(__if(__parity == 1))] {
            let len = p.len();
            let p = __gf::slice_from_slice_mut(p);

            for i in 0..len {
                // calculate new parity
                p[i] += __gf(new[i]) - __gf(old[i]);
            }
        } else if #[cfg(__if(__parity == 2))] {
            let len = p.len();
            let p = __gf::slice_from_slice_mut(p);
            let q = __gf::slice_from_slice_mut(q);

            let g = __gf::GENERATOR.pow(u8::try_from(j).unwrap());
            for i in 0..len {
                // calculate new parity
                p[i] += __gf(new[i]) - __gf(old[i]);
                q[i] += (__gf(new[i]) - __gf(old[i])) * g;
            }
        }
    }
}

