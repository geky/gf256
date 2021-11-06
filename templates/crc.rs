//! Template for CRC functions
//!
//! See examples/crc.rs for a more detailed explanation of
//! where these implementations come from

use __crate::internal::cfg_if::cfg_if;
use __crate::traits::TryFrom;
use __crate::traits::FromLossy;
use core::mem::size_of;


// TODO doc?
pub fn __crc(data: &[u8]) -> __u {
    cfg_if! {
        if #[cfg(__if(__naive))] {
            cfg_if! {
                if #[cfg(__if(__inverted))] {
                    let mut crc = __p(__u::MAX);
                } else {
                    let mut crc = __p(0);
                }
            }

            // iterate over words
            let mut words = data.chunks_exact(size_of::<__u>());
            for word in &mut words {
                let word = <[u8; size_of::<__u>()]>::try_from(word).unwrap();
                cfg_if! {
                    if #[cfg(__if(__reversed))] {
                        crc = crc + __p::from_le_bytes(word).reverse_bits();
                        crc = __p::try_from(
                            (__p2::from(crc) << __width) % __p2(__polynomial)
                        ).unwrap();
                    } else {
                        crc = crc + __p::from_be_bytes(word);
                        crc = __p::try_from(
                            (__p2::from(crc) << __width) % __p2(__polynomial)
                        ).unwrap();
                    }
                }
            }

            // handle remainder
            for b in words.remainder() {
                cfg_if! {
                    if #[cfg(__if(__reversed))] {
                        crc = crc + (__p::from(b.reverse_bits()) << (__width-8));
                        crc = __p::try_from(
                            (__p2::from(crc) << 8) % __p2(__polynomial)
                        ).unwrap();
                    } else {
                        crc = crc + (__p::from(*b) << (__width-8));
                        crc = __p::try_from(
                            (__p2::from(crc) << 8) % __p2(__polynomial)
                        ).unwrap();
                    }
                }
            }

            cfg_if! {
                if #[cfg(__if(__reversed))] {
                    crc = crc.reverse_bits();
                }
            }

            cfg_if! {
                if #[cfg(__if(__inverted))] {
                    crc ^= __p(__u::MAX);
                }
            }

            __u::from(crc)
        } else if #[cfg(__if(__table))] {
            const CRC_TABLE: [__u; 256] = {
                let mut table = [0; 256];
                let mut i = 0;
                while i < table.len() {
                    cfg_if! {
                        if #[cfg(__if(__reversed))] {
                            let x = ((i as u8).reverse_bits() as __u) << (__width-8);
                            let x = __p2((x as __u2) << 8).naive_rem(__p2(__polynomial)).0 as __u;
                            table[i] = x.reverse_bits();
                            i += 1;
                        } else {
                            let x = (i as __u) << (__width-8);
                            let x = __p2((x as __u2) << 8).naive_rem(__p2(__polynomial)).0 as __u;
                            table[i] = x;
                            i += 1;
                        }
                    }
                }
                table
            };

            cfg_if! {
                if #[cfg(__if(__inverted))] {
                    let mut crc = __u::MAX;
                } else {
                    let mut crc = 0;
                }
            }

            for b in data {
                cfg_if! {
                    if #[cfg(__if(__reversed))] {
                        crc = (crc >> 8) ^ CRC_TABLE[usize::from((crc as u8) ^ b)];
                    } else {
                        crc = (crc << 8) ^ CRC_TABLE[usize::from(((crc >> (__width-8)) as u8) ^ b)];
                    }
                }
            }

            cfg_if! {
                if #[cfg(__if(__inverted))] {
                    crc ^= __u::MAX;
                }
            }

            crc
        } else if #[cfg(__if(__small_table))] {
            const CRC_TABLE: [__u; 16] = {
                let mut table = [0; 16];
                let mut i = 0;
                while i < table.len() {
                    cfg_if! {
                        if #[cfg(__if(__reversed))] {
                            let x = ((i as u8).reverse_bits() as __u) << (__width-8);
                            let x = __p2((x as __u2) << 4).naive_rem(__p2(__polynomial)).0 as __u;
                            table[i] = x.reverse_bits();
                            i += 1;
                        } else {
                            let x = (i as __u) << (__width-4);
                            let x = __p2((x as __u2) << 4).naive_rem(__p2(__polynomial)).0 as __u;
                            table[i] = x;
                            i += 1;
                        }
                    }
                }
                table
            };

            cfg_if! {
                if #[cfg(__if(__inverted))] {
                    let mut crc = __u::MAX;
                } else {
                    let mut crc = 0;
                }
            }

            for b in data {
                cfg_if! {
                    if #[cfg(__if(__reversed))] {
                        crc = (crc >> 4) ^ CRC_TABLE[usize::from((crc as u8) ^ (b >> 0)) & 0xf];
                        crc = (crc >> 4) ^ CRC_TABLE[usize::from((crc as u8) ^ (b >> 4)) & 0xf];
                    } else {
                        crc = (crc << 4) ^ CRC_TABLE[usize::from(((crc >> (__width-4)) as u8) ^ (b >> 4)) & 0xf];
                        crc = (crc << 4) ^ CRC_TABLE[usize::from(((crc >> (__width-4)) as u8) ^ (b >> 0)) & 0xf];
                    }
                }
            }

            cfg_if! {
                if #[cfg(__if(__inverted))] {
                    crc ^= __u::MAX;
                }
            }

            crc
        } else if #[cfg(__if(__barret))] {
            const BARRET_CONSTANT: __p = {
                __p(__p2((__polynomial as __u2) << __width).naive_div(__p2(__polynomial)).0 as __u)
            };

            cfg_if! {
                if #[cfg(__if(__inverted))] {
                    let mut crc = __p(__u::MAX);
                } else {
                    let mut crc = __p(0);
                }
            }

            // iterate over words
            let mut words = data.chunks_exact(size_of::<__u>());
            for word in &mut words {
                let word = <[u8; size_of::<__u>()]>::try_from(word).unwrap();
                cfg_if! {
                    if #[cfg(__if(__reversed))] {
                        crc = crc + __p::from_le_bytes(word).reverse_bits();
                        crc = (crc.widening_mul(BARRET_CONSTANT).1 + crc)
                                .wrapping_mul(__p::from_lossy(__polynomial as __u));
                    } else {
                        crc = crc + __p::from_be_bytes(word);
                        crc = (crc.widening_mul(BARRET_CONSTANT).1 + crc)
                                .wrapping_mul(__p::from_lossy(__polynomial as __u));
                    }
                }
            }

            // handle remainder
            for b in words.remainder() {
                cfg_if! {
                    if #[cfg(__if(__reversed))] {
                        crc = crc + (__p::from(b.reverse_bits()) << (__width-8));
                        crc = (crc << 8)
                            + ((crc >> (8*size_of::<__u>() - 8)).widening_mul(BARRET_CONSTANT).1 + (crc >> (8*size_of::<__u>() - 8)))
                                .wrapping_mul(__p::from_lossy(__polynomial as __u));
                    } else {
                        crc = crc + (__p::from(*b) << (__width-8));
                        crc = (crc << 8)
                            + ((crc >> (8*size_of::<__u>() - 8)).widening_mul(BARRET_CONSTANT).1 + (crc >> (8*size_of::<__u>() - 8)))
                                .wrapping_mul(__p::from_lossy(__polynomial as __u));
                    }
                }
            }

            cfg_if! {
                if #[cfg(__if(__reversed))] {
                    crc = crc.reverse_bits();
                }
            }

            cfg_if! {
                if #[cfg(__if(__inverted))] {
                    crc ^= __p(__u::MAX);
                }
            }

            __u::from(crc)
        }
    }
}

