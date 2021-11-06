
use crate::macros::crc;

// crc functions
#[crc(polynomial=0x11021)] fn crc16();
#[crc(polynomial=0x104c11db7)] fn crc32();


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn crc() {
        assert_eq!(crc16(b"Hello World!"), 0x0bbb);
        assert_eq!(crc32(b"Hello World!"), 0x1c291ca3);
    }

    // explicit modes
    #[crc(polynomial=0x11021, naive)] fn crc16_naive();
    #[crc(polynomial=0x104c11db7, naive)] fn crc32_naive();

    #[crc(polynomial=0x11021, table)] fn crc16_table();
    #[crc(polynomial=0x104c11db7, table)] fn crc32_table();

    #[crc(polynomial=0x11021, small_table)] fn crc16_small_table();
    #[crc(polynomial=0x104c11db7, small_table)] fn crc32_small_table();

    #[crc(polynomial=0x11021, barret)] fn crc16_barret();
    #[crc(polynomial=0x104c11db7, barret)] fn crc32_barret();

    #[test]
    fn crc_naive() {
        assert_eq!(crc16_naive(b"Hello World!"), 0x0bbb);
        assert_eq!(crc32_naive(b"Hello World!"), 0x1c291ca3);
    }

    #[test]
    fn crc_table() {
        assert_eq!(crc16_table(b"Hello World!"), 0x0bbb);
        assert_eq!(crc32_table(b"Hello World!"), 0x1c291ca3);
    }

    #[test]
    fn crc_small_table() {
        assert_eq!(crc16_small_table(b"Hello World!"), 0x0bbb);
        assert_eq!(crc32_small_table(b"Hello World!"), 0x1c291ca3);
    }

    #[test]
    fn crc_barret() {
        assert_eq!(crc16_barret(b"Hello World!"), 0x0bbb);
        assert_eq!(crc32_barret(b"Hello World!"), 0x1c291ca3);
    }

    // bit reversed 
    #[crc(polynomial=0x104c11db7, naive, reversed=false)] fn crc32_naive_unreversed();
    #[crc(polynomial=0x104c11db7, table, reversed=false)] fn crc32_table_unreversed();
    #[crc(polynomial=0x104c11db7, small_table, reversed=false)] fn crc32_small_table_unreversed();
    #[crc(polynomial=0x104c11db7, barret, reversed=false)] fn crc32_barret_unreversed();

    #[test]
    fn crc_unreversed() {
        assert_eq!(crc32_naive_unreversed(b"Hello World!"), 0x6b1a7cae);
        assert_eq!(crc32_table_unreversed(b"Hello World!"), 0x6b1a7cae);
        assert_eq!(crc32_small_table_unreversed(b"Hello World!"), 0x6b1a7cae);
        assert_eq!(crc32_barret_unreversed(b"Hello World!"), 0x6b1a7cae);
    }

    // bit inverted 
    #[crc(polynomial=0x104c11db7, naive, inverted=false)] fn crc32_naive_uninverted();
    #[crc(polynomial=0x104c11db7, table, inverted=false)] fn crc32_table_uninverted();
    #[crc(polynomial=0x104c11db7, small_table, inverted=false)] fn crc32_small_table_uninverted();
    #[crc(polynomial=0x104c11db7, barret, inverted=false)] fn crc32_barret_uninverted();

    #[test]
    fn crc_uninverted() {
        assert_eq!(crc32_naive_uninverted(b"Hello World!"), 0x67fcdacc);
        assert_eq!(crc32_table_uninverted(b"Hello World!"), 0x67fcdacc);
        assert_eq!(crc32_small_table_uninverted(b"Hello World!"), 0x67fcdacc);
        assert_eq!(crc32_barret_uninverted(b"Hello World!"), 0x67fcdacc);
    }
}
