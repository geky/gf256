
#[allow(unused)]
use crate::macros::shamir;


// Shamir secret-sharing functions
//
// Note we can only provide a default if we have ThreadRng available,
// otherwise we can only provide the shamir macro which accepts a
// custom Rng type
//
#[cfg(feature="thread-rng")]
#[shamir]
pub mod shamir {}


#[cfg(test)]
mod test {
    use super::shamir as gf256_shamir;
    use crate::macros::*;
    use crate::gf::*;
    use rand::rngs::ThreadRng;
    use core::convert::TryFrom;

    #[cfg(feature="thread-rng")]
    #[test]
    fn shamir5w4() {
        let input = b"Hello World!";
        let shares = gf256_shamir::generate(input, 5, 4);
        assert_eq!(shares.len(), 5);
        for i in 0..5 {
            let output = gf256_shamir::reconstruct(&shares[..i]);
            if i < 4 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    #[cfg(feature="thread-rng")]
    #[test]
    fn shamir255w100() {
        let input = b"Hello World!";
        let shares = gf256_shamir::generate(input, 255, 100);
        assert_eq!(shares.len(), 255);
        for i in (0..255).step_by(51) {
            let output = gf256_shamir::reconstruct(&shares[..i]);
            if i < 100 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    // multi-byte Shamir secrets
    #[cfg(feature="thread-rng")]
    #[shamir(gf=gf2p64, u=u64)]
    mod gf2p64_shamir {}

    #[cfg(feature="thread-rng")]
    #[test]
    fn gf2p64_shamir300w100() {
        let input = b"Hello World!\0\0\0\0"
            .chunks(8)
            .map(|chunk| u64::from_le_bytes(<_>::try_from(chunk).unwrap()))
            .collect::<Vec<_>>();
        let shares = gf2p64_shamir::generate(&input, 300, 100);
        assert_eq!(shares.len(), 300);
        for i in (0..300).step_by(50) {
            let output = gf2p64_shamir::reconstruct(&shares[..i]);
            if i < 100 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    // Shamir with very odd sizes
    #[cfg(feature="thread-rng")]
    #[gf(polynomial=0x13, generator=0x2)]
    type gf16;
    #[cfg(feature="thread-rng")]
    #[shamir(gf=gf16, u=u8)]
    mod gf16_shamir {}

    #[cfg(feature="thread-rng")]
    #[gf(polynomial=0x800021, generator=0x2)]
    type gf2p23;
    #[cfg(feature="thread-rng")]
    #[shamir(gf=gf2p23, u=u32)]
    mod gf2p23_shamir {}

    #[cfg(feature="thread-rng")]
    #[test]
    fn gf16_shamir15w10() {
        let input = b"Hello World!"
            .iter()
            .map(|b| [(b >> 0) & 0xf, (b >> 4) & 0xf])
            .flatten()
            .collect::<Vec<_>>();
        let shares = gf16_shamir::generate(&input, 15, 10);
        assert_eq!(shares.len(), 15);
        for i in 0..15 {
            let output = gf16_shamir::reconstruct(&shares[..i]);
            if i < 10 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    #[cfg(feature="thread-rng")]
    #[test]
    fn gf2p23_shamir300w100() {
        let input = b"Hello World!"
            .chunks(2)
            .map(|chunk| u32::from(u16::from_le_bytes(<_>::try_from(chunk).unwrap())))
            .collect::<Vec<_>>();
        let shares = gf2p23_shamir::generate(&input, 300, 100);
        assert_eq!(shares.len(), 300);
        for i in (0..300).step_by(50) {
            let output = gf2p23_shamir::reconstruct(&shares[..i]);
            if i < 100 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    // TODO test this without ThreadRng?

    // all Shamir parameters 
    #[shamir(gf=gf256, u=u8, rng=ThreadRng::default())]
    mod shamir_all_params {}

    #[test]
    fn shamir_all_params() {
        let input = b"Hello World!";
        let shares = shamir_all_params::generate(input, 255, 100);
        assert_eq!(shares.len(), 255);
        for i in (0..255).step_by(10) {
            let output = shamir_all_params::reconstruct(&shares[..i]);
            if i < 100 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }
}
