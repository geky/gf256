
use crate::macros::lfsr;


// Default LFSR structs
//

#[lfsr(polynomial=0x11d)]
pub struct Lfsr8 {}
#[lfsr(polynomial=0x1002d)]
pub struct Lfsr16 {}
#[lfsr(polynomial=0x1000000af)]
pub struct Lfsr32 {}
#[lfsr(polynomial=0x1000000000000001b)]
pub struct Lfsr64 {}


#[cfg(test)]
mod test {
    use super::*;
    use crate::p::p64;
    use crate::p::p128;
    use core::num::NonZeroU64;
    use core::num::NonZeroU128;
    use core::iter::FromIterator;
    use rand::Rng;

    extern crate alloc;
    use alloc::vec::Vec;
    use alloc::collections::BTreeSet;

    extern crate std;
    use std::iter;

    #[test]
    fn lfsr() {
        let mut lfsr8 = Lfsr8::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16 = Lfsr16::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32 = Lfsr32::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64 = Lfsr64::new(1);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_skip() {
        let mut lfsr8 = Lfsr8::new(1);
        lfsr8.skip(8*8);
        let buf = iter::repeat_with(|| lfsr8.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16 = Lfsr16::new(1);
        lfsr16.skip(16*8);
        let buf = iter::repeat_with(|| lfsr16.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32 = Lfsr32::new(1);
        lfsr32.skip(32*8);
        let buf = iter::repeat_with(|| lfsr32.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64 = Lfsr64::new(1);
        lfsr64.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_skip_backwards() {
        let mut lfsr8 = Lfsr8::new(1);
        lfsr8.skip(8*16);
        lfsr8.skip_backwards(8*8);
        let buf = iter::repeat_with(|| lfsr8.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16 = Lfsr16::new(1);
        lfsr16.skip(16*16);
        lfsr16.skip_backwards(16*8);
        let buf = iter::repeat_with(|| lfsr16.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32 = Lfsr32::new(1);
        lfsr32.skip(32*16);
        lfsr32.skip_backwards(32*8);
        let buf = iter::repeat_with(|| lfsr32.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64 = Lfsr64::new(1);
        lfsr64.skip(64*16);
        lfsr64.skip_backwards(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    // explicit modes
    #[lfsr(polynomial=0x11d, naive, naive_skip)]               pub struct Lfsr8Naive {}
    #[lfsr(polynomial=0x11d, table, table_skip)]               pub struct Lfsr8Table {}
    #[lfsr(polynomial=0x11d, small_table, small_table_skip)]   pub struct Lfsr8SmallTable {}
    #[lfsr(polynomial=0x11d, barret, barret_skip)]             pub struct Lfsr8Barret {}
    #[lfsr(polynomial=0x11d, table_barret, barret_skip)]       pub struct Lfsr8TableBarret {}
    #[lfsr(polynomial=0x11d, small_table_barret, barret_skip)] pub struct Lfsr8SmallTableBarret {}

    #[lfsr(polynomial=0x1002d, naive, naive_skip)]               pub struct Lfsr16Naive {}
    #[lfsr(polynomial=0x1002d, table, table_skip)]               pub struct Lfsr16Table {}
    #[lfsr(polynomial=0x1002d, small_table, small_table_skip)]   pub struct Lfsr16SmallTable {}
    #[lfsr(polynomial=0x1002d, barret, barret_skip)]             pub struct Lfsr16Barret {}
    #[lfsr(polynomial=0x1002d, table_barret, barret_skip)]       pub struct Lfsr16TableBarret {}
    #[lfsr(polynomial=0x1002d, small_table_barret, barret_skip)] pub struct Lfsr16SmallTableBarret {}

    #[lfsr(polynomial=0x1000000af, naive, naive_skip)]               pub struct Lfsr32Naive {}
    #[lfsr(polynomial=0x1000000af, table, table_skip)]               pub struct Lfsr32Table {}
    #[lfsr(polynomial=0x1000000af, small_table, small_table_skip)]   pub struct Lfsr32SmallTable {}
    #[lfsr(polynomial=0x1000000af, barret, barret_skip)]             pub struct Lfsr32Barret {}
    #[lfsr(polynomial=0x1000000af, table_barret, barret_skip)]       pub struct Lfsr32TableBarret {}
    #[lfsr(polynomial=0x1000000af, small_table_barret, barret_skip)] pub struct Lfsr32SmallTableBarret {}

    #[lfsr(polynomial=0x1000000000000001b, naive, naive_skip)]               pub struct Lfsr64Naive {}
    #[lfsr(polynomial=0x1000000000000001b, table, table_skip)]               pub struct Lfsr64Table {}
    #[lfsr(polynomial=0x1000000000000001b, small_table, small_table_skip)]   pub struct Lfsr64SmallTable {}
    #[lfsr(polynomial=0x1000000000000001b, barret, barret_skip)]             pub struct Lfsr64Barret {}
    #[lfsr(polynomial=0x1000000000000001b, table_barret, barret_skip)]       pub struct Lfsr64TableBarret {}
    #[lfsr(polynomial=0x1000000000000001b, small_table_barret, barret_skip)] pub struct Lfsr64SmallTableBarret {}

    // test explicit div/rem modes
    #[test]
    fn lfsr_naive() {
        let mut lfsr8_naive = Lfsr8Naive::new(1);
        let buf = iter::repeat_with(|| lfsr8_naive.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_naive.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_naive = Lfsr16Naive::new(1);
        let buf = iter::repeat_with(|| lfsr16_naive.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_naive.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_naive = Lfsr32Naive::new(1);
        let buf = iter::repeat_with(|| lfsr32_naive.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_naive.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_naive = Lfsr64Naive::new(1);
        let buf = iter::repeat_with(|| lfsr64_naive.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_naive.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_table() {
        let mut lfsr8_table = Lfsr8Table::new(1);
        let buf = iter::repeat_with(|| lfsr8_table.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_table.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_table = Lfsr16Table::new(1);
        let buf = iter::repeat_with(|| lfsr16_table.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_table.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_table = Lfsr32Table::new(1);
        let buf = iter::repeat_with(|| lfsr32_table.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_table.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_table = Lfsr64Table::new(1);
        let buf = iter::repeat_with(|| lfsr64_table.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_table.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_small_table() {
        let mut lfsr8_small_table = Lfsr8SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr8_small_table.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_small_table.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_small_table = Lfsr16SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr16_small_table.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_small_table.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_small_table = Lfsr32SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr32_small_table.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_small_table.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_small_table = Lfsr64SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr64_small_table.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_small_table.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_barret() {
        let mut lfsr8_barret = Lfsr8Barret::new(1);
        let buf = iter::repeat_with(|| lfsr8_barret.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_barret.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_barret = Lfsr16Barret::new(1);
        let buf = iter::repeat_with(|| lfsr16_barret.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_barret.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_barret = Lfsr32Barret::new(1);
        let buf = iter::repeat_with(|| lfsr32_barret.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_barret.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_barret = Lfsr64Barret::new(1);
        let buf = iter::repeat_with(|| lfsr64_barret.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_barret.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_table_barret() {
        let mut lfsr8_table_barret = Lfsr8TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr8_table_barret.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_table_barret.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_table_barret = Lfsr16TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr16_table_barret.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_table_barret.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_table_barret = Lfsr32TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr32_table_barret.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_table_barret.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_table_barret = Lfsr64TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr64_table_barret.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_table_barret.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_small_table_barret() {
        let mut lfsr8_small_table_barret = Lfsr8SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr8_small_table_barret.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_small_table_barret.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_small_table_barret = Lfsr16SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr16_small_table_barret.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_small_table_barret.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_small_table_barret = Lfsr32SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr32_small_table_barret.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_small_table_barret.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_small_table_barret = Lfsr64SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr64_small_table_barret.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_small_table_barret.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    // test explicit skip modes
    #[test]
    fn lfsr_naive_skip() {
        let mut lfsr8_naive = Lfsr8Naive::new(1);
        lfsr8_naive.skip(8*8);
        let buf = iter::repeat_with(|| lfsr8_naive.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_naive = Lfsr16Naive::new(1);
        lfsr16_naive.skip(16*8);
        let buf = iter::repeat_with(|| lfsr16_naive.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_naive = Lfsr32Naive::new(1);
        lfsr32_naive.skip(32*8);
        let buf = iter::repeat_with(|| lfsr32_naive.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_naive = Lfsr64Naive::new(1);
        lfsr64_naive.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64_naive.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_table_skip() {
        let mut lfsr8_table = Lfsr8Table::new(1);
        lfsr8_table.skip(8*8);
        let buf = iter::repeat_with(|| lfsr8_table.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_table = Lfsr16Table::new(1);
        lfsr16_table.skip(16*8);
        let buf = iter::repeat_with(|| lfsr16_table.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_table = Lfsr32Table::new(1);
        lfsr32_table.skip(32*8);
        let buf = iter::repeat_with(|| lfsr32_table.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_table = Lfsr64Table::new(1);
        lfsr64_table.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64_table.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_small_table_skip() {
        let mut lfsr8_small_table = Lfsr8SmallTable::new(1);
        lfsr8_small_table.skip(8*8);
        let buf = iter::repeat_with(|| lfsr8_small_table.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_small_table = Lfsr16SmallTable::new(1);
        lfsr16_small_table.skip(16*8);
        let buf = iter::repeat_with(|| lfsr16_small_table.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_small_table = Lfsr32SmallTable::new(1);
        lfsr32_small_table.skip(32*8);
        let buf = iter::repeat_with(|| lfsr32_small_table.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_small_table = Lfsr64SmallTable::new(1);
        lfsr64_small_table.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64_small_table.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_barret_skip() {
        let mut lfsr8_barret = Lfsr8Barret::new(1);
        lfsr8_barret.skip(8*8);
        let buf = iter::repeat_with(|| lfsr8_barret.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_barret = Lfsr16Barret::new(1);
        lfsr16_barret.skip(16*8);
        let buf = iter::repeat_with(|| lfsr16_barret.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_barret = Lfsr32Barret::new(1);
        lfsr32_barret.skip(32*8);
        let buf = iter::repeat_with(|| lfsr32_barret.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_barret = Lfsr64Barret::new(1);
        lfsr64_barret.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64_barret.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    // odd step sizes
    #[test]
    fn lfsr_odd_nexts() {
        let mut lfsr8 = Lfsr8Naive::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16Naive::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32Naive::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);

        let mut lfsr8 = Lfsr8Table::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16Table::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32Table::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);

        let mut lfsr8 = Lfsr8SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);

        let mut lfsr8 = Lfsr8Barret::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16Barret::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32Barret::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);

        let mut lfsr8 = Lfsr8TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);

        let mut lfsr8 = Lfsr8SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);
    }

    // odd LFSR sizes
    #[lfsr(polynomial=0x13, naive, naive_skip)]               pub struct Lfsr4Naive {}
    #[lfsr(polynomial=0x13, table, table_skip)]               pub struct Lfsr4Table {}
    #[lfsr(polynomial=0x13, small_table, small_table_skip)]   pub struct Lfsr4SmallTable {}
    #[lfsr(polynomial=0x13, barret, barret_skip)]             pub struct Lfsr4Barret {}
    #[lfsr(polynomial=0x13, table_barret, barret_skip)]       pub struct Lfsr4TableBarret {}
    #[lfsr(polynomial=0x13, small_table_barret, barret_skip)] pub struct Lfsr4SmallTableBarret {}

    #[lfsr(polynomial=0x1053, naive, naive_skip)]               pub struct Lfsr12Naive {}
    #[lfsr(polynomial=0x1053, table, table_skip)]               pub struct Lfsr12Table {}
    #[lfsr(polynomial=0x1053, small_table, small_table_skip)]   pub struct Lfsr12SmallTable {}
    #[lfsr(polynomial=0x1053, barret, barret_skip)]             pub struct Lfsr12Barret {}
    #[lfsr(polynomial=0x1053, table_barret, barret_skip)]       pub struct Lfsr12TableBarret {}
    #[lfsr(polynomial=0x1053, small_table_barret, barret_skip)] pub struct Lfsr12SmallTableBarret {}

    #[lfsr(polynomial=0x800021, naive, naive_skip)]               pub struct Lfsr23Naive {}
    #[lfsr(polynomial=0x800021, table, table_skip)]               pub struct Lfsr23Table {}
    #[lfsr(polynomial=0x800021, small_table, small_table_skip)]   pub struct Lfsr23SmallTable {}
    #[lfsr(polynomial=0x800021, barret, barret_skip)]             pub struct Lfsr23Barret {}
    #[lfsr(polynomial=0x800021, table_barret, barret_skip)]       pub struct Lfsr23TableBarret {}
    #[lfsr(polynomial=0x800021, small_table_barret, barret_skip)] pub struct Lfsr23SmallTableBarret {}

    #[test]
    fn lfsr_odd_sizes() {
        let mut lfsr4 = Lfsr4Naive::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Naive::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Naive::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4Table::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Table::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Table::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4Barret::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Barret::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Barret::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);
    }

    #[test]
    fn lfsr_odd_sizes_skip() {
        let mut lfsr4 = Lfsr4Naive::new(1);
        lfsr4.skip(4*8);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Naive::new(1);
        lfsr12.skip(12*8);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Naive::new(1);
        lfsr23.skip(23*8);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4Table::new(1);
        lfsr4.skip(4*8);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Table::new(1);
        lfsr12.skip(12*8);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Table::new(1);
        lfsr23.skip(23*8);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4SmallTable::new(1);
        lfsr4.skip(4*8);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12SmallTable::new(1);
        lfsr12.skip(12*8);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23SmallTable::new(1);
        lfsr23.skip(23*8);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4Barret::new(1);
        lfsr4.skip(4*8);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Barret::new(1);
        lfsr12.skip(12*8);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Barret::new(1);
        lfsr23.skip(23*8);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);
    }

    // bit-reflected LFSRs
    #[lfsr(polynomial=0x1000000000000001b, naive, naive_skip, reflected=true)]               pub struct Lfsr64NaiveReflected {}
    #[lfsr(polynomial=0x1000000000000001b, table, table_skip, reflected=true)]               pub struct Lfsr64TableReflected {}
    #[lfsr(polynomial=0x1000000000000001b, small_table, small_table_skip, reflected=true)]   pub struct Lfsr64SmallTableReflected {}
    #[lfsr(polynomial=0x1000000000000001b, barret, barret_skip, reflected=true)]             pub struct Lfsr64BarretReflected {}
    #[lfsr(polynomial=0x1000000000000001b, table_barret, barret_skip, reflected=true)]       pub struct Lfsr64TableBarretReflected {}
    #[lfsr(polynomial=0x1000000000000001b, small_table_barret, barret_skip, reflected=true)] pub struct Lfsr64SmallTableBarretReflected {}

    #[test]
    fn lfsr_reflected() {
        let mut lfsr64 = Lfsr64NaiveReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64TableReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64SmallTableReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64BarretReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64TableBarretReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64SmallTableBarretReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);
    }

    #[test]
    fn lfsr_reflected_skip() {
        let mut lfsr64 = Lfsr64NaiveReflected::new(0x8000000000000000);
        lfsr64.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64TableReflected::new(0x8000000000000000);
        lfsr64.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64SmallTableReflected::new(0x8000000000000000);
        lfsr64.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64BarretReflected::new(0x8000000000000000);
        lfsr64.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);
    }

    // all LFSR params
    #[lfsr(
        polynomial=0x1000000000000001b,
        u=u64,
        u2=u128,
        nzu=NonZeroU64,
        nzu2=NonZeroU128,
        p=p64,
        p2=p128,
        reflected=false,
    )]
    struct Lfsr64AllParams {}

    #[test]
    fn lfsr_all_params() {
        let mut lfsr64_all_params = Lfsr64AllParams::new(1);
        let buf = iter::repeat_with(|| lfsr64_all_params.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_all_params.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    // other LFSR things

    #[test]
    fn lfsr_rng_consistency() {
        // normal order
        let mut lfsr = Lfsr8::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr8::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr16::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr16::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr32::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr32::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr64::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr64::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr4Table::new(1);
        let next_bytes = iter::repeat_with(|| (lfsr.next(4) << 4) | lfsr.next(4)).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr4Table::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr12Table::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr12Table::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr23Table::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr23Table::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        // reflected order
        let mut lfsr = Lfsr64TableReflected::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr64TableReflected::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);
    }

    #[test]
    fn lfsr_uniqueness() {
        let mut lfsr = Lfsr8::new(1);
        let unique = BTreeSet::from_iter(iter::repeat_with(|| lfsr.next(8)).take(255));
        assert_eq!(unique.len(), 255);

        let mut lfsr = Lfsr64::new(1);
        let unique = BTreeSet::from_iter(iter::repeat_with(|| lfsr.next(64)).take(255));
        assert_eq!(unique.len(), 255);
    }
}
