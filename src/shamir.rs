
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
    use super::*;
    use crate::gf::*;
    use rand::rngs::ThreadRng;

    #[cfg(feature="thread-rng")]
    #[test]
    fn shamir5w4() {
        let input = b"Hello World!";
        let shares = shamir::generate(input, 5, 4);
        assert_eq!(shares.len(), 5);
        for i in 0..5 {
            let output = shamir::reconstruct(&shares[..i]);
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
        let shares = shamir::generate(input, 255, 100);
        assert_eq!(shares.len(), 255);
        for i in (0..255).step_by(51) {
            let output = shamir::reconstruct(&shares[..i]);
            if i < 100 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    // all Shamir parameters 
    #[crate::macros::shamir(gf=gf256, rng=ThreadRng::default())]
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
