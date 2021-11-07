
use crate::macros::gf;
use crate::macros::lfsr;


// Default LFSR structs
//
// Note these use a polynomial and generator with a fairly even
// distribution of ones and zeros
//

#[gf(polynomial=0x1234567a7, generator=0x1234567a)]
type gf2p32;
#[lfsr(gf=gf2p32, u=u32)]
pub struct Lfsr32 {}

#[gf(polynomial=0x123456789abcdef6b, generator=0x123456789abcdef3)]
type gf2p64;
#[lfsr(gf=gf2p64, u=u64)]
pub struct Lfsr64 {}


#[cfg(test)]
mod test {
    use super::*;
    use core::iter;
    use core::iter::FromIterator;
    use core::cmp::min;

    extern crate alloc;
    use alloc::vec::Vec;
    use alloc::collections::BTreeSet;

    #[test]
    fn lfsr32() {
        let mut lfsr = Lfsr32::new(42);
        let mut lfsr_clone = lfsr.clone();
        let some = (&mut lfsr).take(100).collect::<Vec<_>>();
        let unique = BTreeSet::from_iter(&some);
        assert_eq!(unique.len(), min(100, gf2p32::NONZEROS) as usize);

        let again = (&mut lfsr_clone).take(100).collect::<Vec<_>>();
        assert_eq!(some, again);

        let mut rev = iter::repeat_with(|| (&mut lfsr).prev()).take(100).collect::<Vec<_>>();
        rev.reverse();
        assert_eq!(some, rev);

        let again = (&mut lfsr).take(100).collect::<Vec<_>>();
        assert_eq!(some, again);
    }

    #[test]
    fn lfsr64() {
        let mut lfsr = Lfsr64::new(42);
        let mut lfsr_clone = lfsr.clone();
        let some = (&mut lfsr).take(100).collect::<Vec<_>>();
        let unique = BTreeSet::from_iter(&some);
        assert_eq!(unique.len(), min(100, gf2p64::NONZEROS) as usize);

        let again = (&mut lfsr_clone).take(100).collect::<Vec<_>>();
        assert_eq!(some, again);

        let mut rev = iter::repeat_with(|| (&mut lfsr).prev()).take(100).collect::<Vec<_>>();
        rev.reverse();
        assert_eq!(some, rev);

        let again = (&mut lfsr).take(100).collect::<Vec<_>>();
        assert_eq!(some, again);
    }

    // odd-sized lfsrs
    #[gf(polynomial=0x13, generator=0x2)]
    type gf16;
    #[lfsr(gf=gf16, u=u8)]
    pub struct Lfsr4 {}

    #[test]
    fn lfsr4() {
        let mut lfsr = Lfsr4::new(42);
        let mut lfsr_clone = lfsr.clone();
        let some = (&mut lfsr).take(100).collect::<Vec<_>>();
        let unique = BTreeSet::from_iter(&some);
        assert_eq!(unique.len(), min(100, gf16::NONZEROS) as usize);

        let again = (&mut lfsr_clone).take(100).collect::<Vec<_>>();
        assert_eq!(some, again);

        let mut rev = iter::repeat_with(|| (&mut lfsr).prev()).take(100).collect::<Vec<_>>();
        rev.reverse();
        assert_eq!(some, rev);

        let again = (&mut lfsr).take(100).collect::<Vec<_>>();
        assert_eq!(some, again);
    }

    #[gf(polynomial=0x1237, generator=0x123)]
    type gf4096;
    #[lfsr(gf=gf4096, u=u16)]
    pub struct Lfsr12 {}

    #[test]
    fn lfsr12() {
        let mut lfsr = Lfsr12::new(42);
        let mut lfsr_clone = lfsr.clone();
        let some = (&mut lfsr).take(100).collect::<Vec<_>>();
        let unique = BTreeSet::from_iter(&some);
        assert_eq!(unique.len(), min(100, gf4096::NONZEROS) as usize);

        let again = (&mut lfsr_clone).take(100).collect::<Vec<_>>();
        assert_eq!(some, again);

        let mut rev = iter::repeat_with(|| (&mut lfsr).prev()).take(100).collect::<Vec<_>>();
        rev.reverse();
        assert_eq!(some, rev);

        let again = (&mut lfsr).take(100).collect::<Vec<_>>();
        assert_eq!(some, again);
    }
}
