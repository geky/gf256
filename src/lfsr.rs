
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

    extern crate alloc;
    use alloc::vec::Vec;

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

    // TODO test reflected
    // TODO test odd sizes (4, 12, 23)
    // TODO test odd nexts!

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
}
