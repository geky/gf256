//! Template for CRC functions
//!
//! See examples/crc.rs for a more detailed explanation of
//! where these implementations come from

use __crate::internal::cfg_if::cfg_if;
use __crate::traits::TryFrom;
use __crate::traits::FromLossy;
use core::mem::size_of;


/// Calculate the CRC for a piece of data.
///
/// ``` rust
/// # use ::gf256::crc::*;
/// assert_eq!(crc32c(b"Hello World!", 0), 0xfe6cf1dc);
/// ```
///
/// Note that this takes the previous state of the CRC as an argument,
/// allowing the CRC to be computed incrementally:
///
/// ``` rust
/// # use ::gf256::crc::*;
/// assert_eq!(crc32c(b"Hell", 0x00000000), 0x77bce1bf);
/// assert_eq!(crc32c(b"o Wo", 0x77bce1bf), 0xf92d22b8);
/// assert_eq!(crc32c(b"rld!", 0xf92d22b8), 0xfe6cf1dc);
/// assert_eq!(crc32c(b"Hello World!", 0), 0xfe6cf1dc);
/// ```
///
/// See the [module-level documentation](../crc) for more info.
///
pub fn __crc(data: &[u8], crc: __u) -> __u {
    cfg_if! {
        if #[cfg(__if(__naive))] {
            let mut crc = __p(crc ^ __xor);

            cfg_if! {
                if #[cfg(__if(__reflected))] {
                    crc = crc.reverse_bits() >> (8*size_of::<__u>()-__width);
                }
            }

            crc = crc << 8*size_of::<__u>()-__width;

            // iterate over words
            let mut words = data.chunks_exact(size_of::<__u>());
            for word in &mut words {
                let word = <[u8; size_of::<__u>()]>::try_from(word).unwrap();
                cfg_if! {
                    if #[cfg(__if(__reflected))] {
                        crc = crc + __p::from_le_bytes(word).reverse_bits();
                    } else {
                        crc = crc + __p::from_be_bytes(word);
                    }
                }
                crc = __p::try_from(
                    (__p2::from(crc) << 8*size_of::<__u>()) % __p2(__polynomial << (8*size_of::<__u>()-__width))
                ).unwrap();
            }

            // handle remainder
            for b in words.remainder() {
                cfg_if! {
                    if #[cfg(__if(__reflected))] {
                        crc = crc + (__p::from(b.reverse_bits()) << (8*size_of::<__u>()-8));
                    } else {
                        crc = crc + (__p::from(*b) << (8*size_of::<__u>()-8));
                    }
                }
                crc = __p::try_from(
                    (__p2::from(crc) << 8) % __p2(__polynomial << (8*size_of::<__u>()-__width))
                ).unwrap();
            }

            // our division is always 8-bit aligned, so we need to do some
            // finagling if our crc is not 8-bit aligned
            crc = crc >> 8*size_of::<__u>()-__width;

            cfg_if! {
                if #[cfg(__if(__reflected))] {
                    crc = crc.reverse_bits() >> (8*size_of::<__u>()-__width);
                }
            }

            __u::from(crc) ^ __xor
        } else if #[cfg(__if(__table))] {
            const CRC_TABLE: [__u; 256] = {
                let mut table = [0; 256];
                let mut i = 0;
                while i < table.len() {
                    cfg_if! {
                        if #[cfg(__if(__reflected))] {
                            let x = ((i as u8).reverse_bits() as __u) << (8*size_of::<__u>()-8);
                            let x = __p2((x as __u2) << 8)
                                .naive_rem(__p2(__polynomial << (8*size_of::<__u>()-__width))).0 as __u;
                            table[i] = x.reverse_bits();
                            i += 1;
                        } else {
                            let x = (i as __u) << (8*size_of::<__u>()-8);
                            let x = __p2((x as __u2) << 8)
                                .naive_rem(__p2(__polynomial << (8*size_of::<__u>()-__width))).0 as __u;
                            table[i] = x;
                            i += 1;
                        }
                    }
                }
                table
            };

            cfg_if! {
                if #[cfg(__if(__reflected))] {
                    let mut crc = crc ^ __xor;
                } else {
                    let mut crc = (crc ^ __xor) << (8*size_of::<__u>()-__width);
                }
            }

            for b in data {
                cfg_if! {
                    if #[cfg(__if(__width <= 8))] {
                        crc = CRC_TABLE[usize::from((crc as u8) ^ b)];
                    } else if #[cfg(__if(__reflected))] {
                        crc = (crc >> 8) ^ CRC_TABLE[usize::from((crc as u8) ^ b)];
                    } else {
                        crc = (crc << 8) ^ CRC_TABLE[usize::from(((crc >> (8*size_of::<__u>()-8)) as u8) ^ b)];
                    }
                }
            }

            // our division is always 8-bit aligned, so we need to do some
            // finagling if our crc is not 8-bit aligned
            cfg_if! {
                if #[cfg(__if(__reflected))] {
                    crc = crc & __nonzeros;
                } else {
                    crc = crc >> (8*size_of::<__u>()-__width);
                }
            }

            crc ^ __xor
        } else if #[cfg(__if(__small_table))] {
            const CRC_TABLE: [__u; 16] = {
                let mut table = [0; 16];
                let mut i = 0;
                while i < table.len() {
                    cfg_if! {
                        if #[cfg(__if(__reflected))] {
                            let x = ((i as u8).reverse_bits() as __u) << (8*size_of::<__u>()-8);
                            let x = __p2((x as __u2) << 4)
                                .naive_rem(__p2(__polynomial << (8*size_of::<__u>()-__width))).0 as __u;
                            table[i] = x.reverse_bits();
                            i += 1;
                        } else {
                            let x = (i as __u) << (8*size_of::<__u>()-4);
                            let x = __p2((x as __u2) << 4)
                                .naive_rem(__p2(__polynomial << (8*size_of::<__u>()-__width))).0 as __u;
                            table[i] = x;
                            i += 1;
                        }
                    }
                }
                table
            };

            cfg_if! {
                if #[cfg(__if(__reflected))] {
                    let mut crc = crc ^ __xor;
                } else {
                    let mut crc = (crc ^ __xor) << (8*size_of::<__u>()-__width);
                }
            }

            for b in data {
                cfg_if! {
                    if #[cfg(__if(__reflected))] {
                        crc = (crc >> 4) ^ CRC_TABLE[usize::from((crc as u8) ^ (b >> 0)) & 0xf];
                        crc = (crc >> 4) ^ CRC_TABLE[usize::from((crc as u8) ^ (b >> 4)) & 0xf];
                    } else {
                        crc = (crc << 4) ^ CRC_TABLE[usize::from(((crc >> (8*size_of::<__u>()-4)) as u8) ^ (b >> 4)) & 0xf];
                        crc = (crc << 4) ^ CRC_TABLE[usize::from(((crc >> (8*size_of::<__u>()-4)) as u8) ^ (b >> 0)) & 0xf];
                    }
                }
            }

            // our division is always 8-bit aligned, so we need to do some
            // finagling if our crc is not 8-bit aligned
            cfg_if! {
                if #[cfg(__if(__reflected))] {
                    crc = crc & __nonzeros;
                } else {
                    crc = crc >> (8*size_of::<__u>()-__width);
                }
            }

            crc ^ __xor
        } else if #[cfg(__if(__barret))] {
            const BARRET_CONSTANT: __p = {
                __p(
                    __p2((__polynomial & __nonzeros) << ((8*size_of::<__u>()-__width) + 8*size_of::<__u>()))
                        .naive_div(__p2(__polynomial << (8*size_of::<__u>()-__width)))
                        .0 as __u
                )
            };

            let mut crc = __p(crc ^ __xor);

            cfg_if! {
                if #[cfg(__if(__reflected))] {
                    crc = crc.reverse_bits() >> (8*size_of::<__u>()-__width);
                }
            }

            crc = crc << 8*size_of::<__u>()-__width;

            // iterate over words
            let mut words = data.chunks_exact(size_of::<__u>());
            for word in &mut words {
                let word = <[u8; size_of::<__u>()]>::try_from(word).unwrap();
                cfg_if! {
                    if #[cfg(__if(__reflected))] {
                        crc = crc + __p::from_le_bytes(word).reverse_bits();
                    } else {
                        crc = crc + __p::from_be_bytes(word);
                    }
                }
                crc = (crc.widening_mul(BARRET_CONSTANT).1 + crc)
                        .wrapping_mul(__p((__polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
            }

            // handle remainder
            for b in words.remainder() {
                cfg_if! {
                    if #[cfg(__if(__reflected))] {
                        crc = crc + (__p::from(b.reverse_bits()) << (8*size_of::<__u>()-8));
                    } else {
                        crc = crc + (__p::from(*b) << (8*size_of::<__u>()-8));
                    }
                }
                crc = (crc << 8)
                    + ((crc >> (8*size_of::<__u>()-8)).widening_mul(BARRET_CONSTANT).1 + (crc >> (8*size_of::<__u>()-8)))
                        .wrapping_mul(__p((__polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
            }

            // our division is always 8-bit aligned, so we need to do some
            // finagling if our crc is not 8-bit aligned
            crc = crc >> (8*size_of::<__u>()-__width);

            cfg_if! {
                if #[cfg(__if(__reflected))] {
                    crc = crc.reverse_bits() >> (8*size_of::<__u>()-__width);
                }
            }

            __u::from(crc) ^ __xor
        }
    }
}

