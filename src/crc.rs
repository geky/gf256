
use crate::macros::crc;

// CRC functions
//
// Hamming distance (HD) info from here:
// http://users.ece.cmu.edu/~koopman/crc/index.html

// HD=3,4, up to 119+8 bits
#[crc(polynomial=0x107)]
pub fn crc8() {}

// HD=3,4, up to 32751+16 bits
#[crc(polynomial=0x11021)]
pub fn crc16() {}

// HD=3, up to 4294967263+32 bits
// HD=4, up to 91607+32 bits
// HD=5, up to 2974+32 bits
// HD=6, up to 268+32 bits
#[crc(polynomial=0x104c11db7)]
pub fn crc32() {}

// HD=3,4, up to 2147483615+32 bits
// HD=5,6, up to 5243+32 bits
#[crc(polynomial=0x11edc6f41)]
pub fn crc32c() {}

// HD=3,4, up to 8589606850+64 bits
// HD=5,6, up to 126701+64 bits
#[crc(polynomial=0x142f0e1eba9ea3693)]
pub fn crc64() {}


#[cfg(test)]
mod test {
    use super::*;
    use crate::p::*;

    #[test]
    fn crc() {
        assert_eq!(crc8(b"Hello World!"),   0xb3);
        assert_eq!(crc16(b"Hello World!"),  0x0bbb);
        assert_eq!(crc32(b"Hello World!"),  0x1c291ca3);
        assert_eq!(crc32c(b"Hello World!"), 0xfe6cf1dc);
        assert_eq!(crc64(b"Hello World!"),  0x75045245c9ea6fe2);
    }

    // explicit modes
    #[crc(polynomial=0x107, naive)] fn crc8_naive() {}
    #[crc(polynomial=0x11021, naive)] fn crc16_naive() {}
    #[crc(polynomial=0x104c11db7, naive)] fn crc32_naive() {}
    #[crc(polynomial=0x11edc6f41, naive)] fn crc32c_naive() {}
    #[crc(polynomial=0x142f0e1eba9ea3693, naive)] fn crc64_naive() {}

    #[crc(polynomial=0x107, table)] fn crc8_table() {}
    #[crc(polynomial=0x11021, table)] fn crc16_table() {}
    #[crc(polynomial=0x104c11db7, table)] fn crc32_table() {}
    #[crc(polynomial=0x11edc6f41, table)] fn crc32c_table() {}
    #[crc(polynomial=0x142f0e1eba9ea3693, table)] fn crc64_table() {}

    #[crc(polynomial=0x107, small_table)] fn crc8_small_table() {}
    #[crc(polynomial=0x11021, small_table)] fn crc16_small_table() {}
    #[crc(polynomial=0x104c11db7, small_table)] fn crc32_small_table() {}
    #[crc(polynomial=0x11edc6f41, small_table)] fn crc32c_small_table() {}
    #[crc(polynomial=0x142f0e1eba9ea3693, small_table)] fn crc64_small_table() {}

    #[crc(polynomial=0x107, barret)] fn crc8_barret() {}
    #[crc(polynomial=0x11021, barret)] fn crc16_barret() {}
    #[crc(polynomial=0x104c11db7, barret)] fn crc32_barret() {}
    #[crc(polynomial=0x11edc6f41, barret)] fn crc32c_barret() {}
    #[crc(polynomial=0x142f0e1eba9ea3693, barret)] fn crc64_barret() {}

    #[test]
    fn crc_naive() {
        assert_eq!(crc8_naive(b"Hello World!"),   0xb3);
        assert_eq!(crc16_naive(b"Hello World!"),  0x0bbb);
        assert_eq!(crc32_naive(b"Hello World!"),  0x1c291ca3);
        assert_eq!(crc32c_naive(b"Hello World!"), 0xfe6cf1dc);
        assert_eq!(crc64_naive(b"Hello World!"),  0x75045245c9ea6fe2);
    }

    #[test]
    fn crc_table() {
        assert_eq!(crc8_table(b"Hello World!"),   0xb3);
        assert_eq!(crc16_table(b"Hello World!"),  0x0bbb);
        assert_eq!(crc32_table(b"Hello World!"),  0x1c291ca3);
        assert_eq!(crc32c_table(b"Hello World!"), 0xfe6cf1dc);
        assert_eq!(crc64_table(b"Hello World!"),  0x75045245c9ea6fe2);
    }

    #[test]
    fn crc_small_table() {
        assert_eq!(crc8_small_table(b"Hello World!"),   0xb3);
        assert_eq!(crc16_small_table(b"Hello World!"),  0x0bbb);
        assert_eq!(crc32_small_table(b"Hello World!"),  0x1c291ca3);
        assert_eq!(crc32c_small_table(b"Hello World!"), 0xfe6cf1dc);
        assert_eq!(crc64_small_table(b"Hello World!"),  0x75045245c9ea6fe2);
    }

    #[test]
    fn crc_barret() {
        assert_eq!(crc8_barret(b"Hello World!"),   0xb3);
        assert_eq!(crc16_barret(b"Hello World!"),  0x0bbb);
        assert_eq!(crc32_barret(b"Hello World!"),  0x1c291ca3);
        assert_eq!(crc32c_barret(b"Hello World!"), 0xfe6cf1dc);
        assert_eq!(crc64_barret(b"Hello World!"),  0x75045245c9ea6fe2);
    }

    #[test]
    fn crc_unaligned() {
        assert_eq!(crc8_naive(b"Hello World!!"),   0x2f);
        assert_eq!(crc16_naive(b"Hello World!!"),  0xcba0);
        assert_eq!(crc32_naive(b"Hello World!!"),  0xd1a8249d);
        assert_eq!(crc32c_naive(b"Hello World!!"), 0x1ec51c06);
        assert_eq!(crc64_naive(b"Hello World!!"),  0xf5a8a397b60da2e1);

        assert_eq!(crc8_table(b"Hello World!!"),   0x2f);
        assert_eq!(crc16_table(b"Hello World!!"),  0xcba0);
        assert_eq!(crc32_table(b"Hello World!!"),  0xd1a8249d);
        assert_eq!(crc32c_table(b"Hello World!!"), 0x1ec51c06);
        assert_eq!(crc64_table(b"Hello World!!"),  0xf5a8a397b60da2e1);

        assert_eq!(crc8_small_table(b"Hello World!!"),   0x2f);
        assert_eq!(crc16_small_table(b"Hello World!!"),  0xcba0);
        assert_eq!(crc32_small_table(b"Hello World!!"),  0xd1a8249d);
        assert_eq!(crc32c_small_table(b"Hello World!!"), 0x1ec51c06);
        assert_eq!(crc64_small_table(b"Hello World!!"),  0xf5a8a397b60da2e1);

        assert_eq!(crc8_barret(b"Hello World!!"),   0x2f);
        assert_eq!(crc16_barret(b"Hello World!!"),  0xcba0);
        assert_eq!(crc32_barret(b"Hello World!!"),  0xd1a8249d);
        assert_eq!(crc32c_barret(b"Hello World!!"), 0x1ec51c06);
        assert_eq!(crc64_barret(b"Hello World!!"),  0xf5a8a397b60da2e1);
    }

    // odd-sized crcs
    #[crc(polynomial=0x13, naive)] fn crc4_naive() {}
    #[crc(polynomial=0x13, table)] fn crc4_table() {}
    #[crc(polynomial=0x13, small_table)] fn crc4_small_table() {}
    #[crc(polynomial=0x13, barret)] fn crc4_barret() {}

    #[crc(polynomial=0x11e7, naive)] fn crc12_naive() {}
    #[crc(polynomial=0x11e7, table)] fn crc12_table() {}
    #[crc(polynomial=0x11e7, small_table)] fn crc12_small_table() {}
    #[crc(polynomial=0x11e7, barret)] fn crc12_barret() {}

    #[crc(polynomial=0x8002a9, naive)] fn crc23_naive() {}
    #[crc(polynomial=0x8002a9, table)] fn crc23_table() {}
    #[crc(polynomial=0x8002a9, small_table)] fn crc23_small_table() {}
    #[crc(polynomial=0x8002a9, barret)] fn crc23_barret() {}

    #[test]
    fn crc_odd_sizes() {
        assert_eq!(crc4_naive(b"Hello World!"),       0x7);
        assert_eq!(crc4_table(b"Hello World!"),       0x7);
        assert_eq!(crc4_small_table(b"Hello World!"), 0x7);
        assert_eq!(crc4_barret(b"Hello World!"),      0x7);

        assert_eq!(crc12_naive(b"Hello World!"),       0x1d4);
        assert_eq!(crc12_table(b"Hello World!"),       0x1d4);
        assert_eq!(crc12_small_table(b"Hello World!"), 0x1d4);
        assert_eq!(crc12_barret(b"Hello World!"),      0x1d4);

        assert_eq!(crc23_naive(b"Hello World!"),       0x32da1c);
        assert_eq!(crc23_table(b"Hello World!"),       0x32da1c);
        assert_eq!(crc23_small_table(b"Hello World!"), 0x32da1c);
        assert_eq!(crc23_barret(b"Hello World!"),      0x32da1c);

        assert_eq!(crc4_naive(b"Hello World!!"),       0x1);
        assert_eq!(crc4_table(b"Hello World!!"),       0x1);
        assert_eq!(crc4_small_table(b"Hello World!!"), 0x1);
        assert_eq!(crc4_barret(b"Hello World!!"),      0x1);

        assert_eq!(crc12_naive(b"Hello World!!"),       0xb8d);
        assert_eq!(crc12_table(b"Hello World!!"),       0xb8d);
        assert_eq!(crc12_small_table(b"Hello World!!"), 0xb8d);
        assert_eq!(crc12_barret(b"Hello World!!"),      0xb8d);

        assert_eq!(crc23_naive(b"Hello World!!"),       0x11685a);
        assert_eq!(crc23_table(b"Hello World!!"),       0x11685a);
        assert_eq!(crc23_small_table(b"Hello World!!"), 0x11685a);
        assert_eq!(crc23_barret(b"Hello World!!"),      0x11685a);
    }

    // bit reflected 
    #[crc(polynomial=0x104c11db7, naive, reflected=false)] fn crc32_naive_unreflected() {}
    #[crc(polynomial=0x104c11db7, table, reflected=false)] fn crc32_table_unreflected() {}
    #[crc(polynomial=0x104c11db7, small_table, reflected=false)] fn crc32_small_table_unreflected() {}
    #[crc(polynomial=0x104c11db7, barret, reflected=false)] fn crc32_barret_unreflected() {}

    #[test]
    fn crc_unreflected() {
        assert_eq!(crc32_naive_unreflected(b"Hello World!"),       0x6b1a7cae);
        assert_eq!(crc32_table_unreflected(b"Hello World!"),       0x6b1a7cae);
        assert_eq!(crc32_small_table_unreflected(b"Hello World!"), 0x6b1a7cae);
        assert_eq!(crc32_barret_unreflected(b"Hello World!"),      0x6b1a7cae);
    }

    // bit inverted 
    #[crc(polynomial=0x104c11db7, naive, init=0, xor=0)] fn crc32_naive_uninverted() {}
    #[crc(polynomial=0x104c11db7, table, init=0, xor=0)] fn crc32_table_uninverted() {}
    #[crc(polynomial=0x104c11db7, small_table, init=0, xor=0)] fn crc32_small_table_uninverted() {}
    #[crc(polynomial=0x104c11db7, barret, init=0, xor=0)] fn crc32_barret_uninverted() {}

    #[test]
    fn crc_uninverted() {
        assert_eq!(crc32_naive_uninverted(b"Hello World!"),       0x67fcdacc);
        assert_eq!(crc32_table_uninverted(b"Hello World!"),       0x67fcdacc);
        assert_eq!(crc32_small_table_uninverted(b"Hello World!"), 0x67fcdacc);
        assert_eq!(crc32_barret_uninverted(b"Hello World!"),      0x67fcdacc);
    }

    // all CRC params
    #[crc(
        polynomial=0x104c11db7,
        u=u32,
        u2=u64,
        p=p32,
        p2=p64,
        reflected=true,
        init=0xffffffff,
        xor=0xffffffff,
    )]
    fn crc32_all_params() {}

    #[test]
    fn crc_all_params() {
        assert_eq!(crc32_all_params(b"Hello World!"), 0x1c291ca3);
    }
}
