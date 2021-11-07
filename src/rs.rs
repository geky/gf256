
use crate::macros::rs;


// Reed-Solomon error-correction functions
//

#[rs(block=26, data=16)]
pub mod rs26w16 {}

#[rs(block=255, data=223)]
pub mod rs255w223 {}


#[cfg(test)]
mod test {
    use super::*;
    use crate::gf::*;

    extern crate alloc;
    use alloc::vec::Vec;

    #[test]
    fn rs26w16() {
        let mut data = (0..26).collect::<Vec<u8>>();
        rs26w16::encode(&mut data);
        assert!(rs26w16::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(26-16) {
            data[0..i].fill(b'x');
            let res = rs26w16::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(26-16)/2 {
            data[0..i].fill(b'x');
            let res = rs26w16::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn rs26w16_any() {
        let mut data = (0..26).collect::<Vec<u8>>();
        rs26w16::encode(&mut data);

        // try any single error
        for i in 0..26 {
            data[i] = b'x';
            let res = rs26w16::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(1));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn rs26w16_burst() {
        let mut data = (0..26).collect::<Vec<u8>>();
        rs26w16::encode(&mut data);

        // try any burst of k/2 errors
        for i in 0..26-((26-16)/2) {
            data[i..i+((26-16)/2)].fill(b'x');
            let res = rs26w16::correct_errors(&mut data);
            assert_eq!(res.ok(), Some((26-16)/2));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn rs255w223() {
        let mut data = (0..255).collect::<Vec<u8>>();
        rs255w223::encode(&mut data);
        assert!(rs255w223::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(255-223) {
            data[0..i].fill(b'x');
            let res = rs255w223::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..223], &(0..223).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(255-223)/2 {
            data[0..i].fill(b'x');
            let res = rs255w223::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..223], &(0..223).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn rs255w223_any() {
        let mut data = (0..255).collect::<Vec<u8>>();
        rs255w223::encode(&mut data);

        // try any single error
        for i in 0..255 {
            data[i] = b'\xff';
            let res = rs255w223::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(1));
            assert_eq!(&data[0..223], &(0..223).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn rs255w223_burst() {
        let mut data = (0..255).collect::<Vec<u8>>();
        rs255w223::encode(&mut data);

        // try any burst of k/2 errors
        for i in 0..255-((255-223)/2) {
            data[i..i+((255-223)/2)].fill(b'\xff');
            let res = rs255w223::correct_errors(&mut data);
            assert_eq!(res.ok(), Some((255-223)/2));
            assert_eq!(&data[0..223], &(0..223).collect::<Vec<u8>>());
        }
    }

    // try a shortened message
    #[test]
    fn rs255w223_shortened() {
        let mut data = (0..40).collect::<Vec<u8>>();
        rs255w223::encode(&mut data);
        assert!(rs255w223::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(40-8) {
            data[0..i].fill(b'x');
            let res = rs255w223::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..8], &(0..8).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(40-8)/2 {
            data[0..i].fill(b'x');
            let res = rs255w223::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..8], &(0..8).collect::<Vec<u8>>());
        }
    }

    // try an overly saturated RS scheme
    #[rs(block=64, data=8)]
    mod rs64w8 {}

    #[test]
    fn rs64w8() {
        let mut data = (0..64).collect::<Vec<u8>>();
        rs64w8::encode(&mut data);
        assert!(rs64w8::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(64-8) {
            data[0..i].fill(b'x');
            let res = rs64w8::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..8], &(0..8).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(64-8)/2 {
            data[0..i].fill(b'x');
            let res = rs64w8::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..8], &(0..8).collect::<Vec<u8>>());
        }
    }

    // all RS params
    #[rs(block=26, data=16, gf=gf256)]
    mod rs26w16_all_params {}

    #[test]
    fn rs_all_params() {
        let mut data = (0..26).collect::<Vec<u8>>();
        rs26w16_all_params::encode(&mut data);
        assert!(rs26w16_all_params::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(26-16) {
            data[0..i].fill(b'x');
            let res = rs26w16_all_params::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(26-16)/2 {
            data[0..i].fill(b'x');
            let res = rs26w16_all_params::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }
    }
}
