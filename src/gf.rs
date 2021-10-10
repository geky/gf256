
use crate::macros::gf;

// Galois field type
#[gf(polynomial=0x11d, generator=0x02)]
pub type gf256;


#[cfg(test)]
mod test {
    use super::*;

    // Create a custom gf type here (Rijndael's finite field) to test that
    // the generator search works
    #[gf(polynomial=0x11b)]
    type gf256_rijndael;

    // Test both table-based and Barret reduction implementations
    #[gf(polynomial=0x11d, generator=0x02, table)]
    type gf256_table;
    #[gf(polynomial=0x11d, generator=0x02, barret)]
    type gf256_barret;

    #[test]
    fn add() {
        assert_eq!(gf256(0x12) + gf256(0x34), gf256(0x26));
        assert_eq!(gf256_table(0x12) + gf256_table(0x34), gf256_table(0x26));
        assert_eq!(gf256_barret(0x12) + gf256_barret(0x34), gf256_barret(0x26));
        assert_eq!(gf256_rijndael(0x12) + gf256_rijndael(0x34), gf256_rijndael(0x26));
    }

    #[test]
    fn sub() {
        assert_eq!(gf256(0x12) - gf256(0x34), gf256(0x26));
        assert_eq!(gf256_table(0x12) - gf256_table(0x34), gf256_table(0x26));
        assert_eq!(gf256_barret(0x12) - gf256_barret(0x34), gf256_barret(0x26));
        assert_eq!(gf256_rijndael(0x12) - gf256_rijndael(0x34), gf256_rijndael(0x26));
    }

    #[test]
    fn mul() {
        assert_eq!(gf256(0x12).naive_mul(gf256(0x34)), gf256(0x0f));
        assert_eq!(gf256_table(0x12).naive_mul(gf256_table(0x34)), gf256_table(0x0f));
        assert_eq!(gf256_barret(0x12).naive_mul(gf256_barret(0x34)), gf256_barret(0x0f));
        assert_eq!(gf256_rijndael(0x12).naive_mul(gf256_rijndael(0x34)), gf256_rijndael(0x05));

        assert_eq!(gf256(0x12) * gf256(0x34), gf256(0x0f));
        assert_eq!(gf256_table(0x12) * gf256_table(0x34), gf256_table(0x0f));
        assert_eq!(gf256_barret(0x12) * gf256_barret(0x34), gf256_barret(0x0f));
        assert_eq!(gf256_rijndael(0x12) * gf256_rijndael(0x34), gf256_rijndael(0x05));
    }

    #[test]
    fn div() {
        assert_eq!(gf256(0x12).naive_div(gf256(0x34)), gf256(0xc7));
        assert_eq!(gf256_table(0x12).naive_div(gf256_table(0x34)), gf256_table(0xc7));
        assert_eq!(gf256_barret(0x12).naive_div(gf256_barret(0x34)), gf256_barret(0xc7));
        assert_eq!(gf256_rijndael(0x12).naive_div(gf256_rijndael(0x34)), gf256_rijndael(0x54));

        assert_eq!(gf256(0x12) / gf256(0x34), gf256(0xc7));
        assert_eq!(gf256_table(0x12) / gf256_table(0x34), gf256_table(0xc7));
        assert_eq!(gf256_barret(0x12) / gf256_barret(0x34), gf256_barret(0xc7));
        assert_eq!(gf256_rijndael(0x12) / gf256_rijndael(0x34), gf256_rijndael(0x54));
    }

    #[test]
    fn all_mul() {
        // test all multiplications
        for a in 0..=255 {
            for b in 0..=255 {
                let x = gf256(a).naive_mul(gf256(b));
                let y = gf256(a) * gf256(b);
                let z = gf256_barret(a) * gf256_barret(b);
                let w = gf256_table(a) * gf256_table(b);
                assert_eq!(u8::from(x), u8::from(y));
                assert_eq!(u8::from(x), u8::from(z));
                assert_eq!(u8::from(x), u8::from(w));
            }
        }
    }

    #[test]
    fn all_div() {
        // test all divisions
        for a in 0..=255 {
            for b in 1..=255 {
                let x = gf256(a).naive_div(gf256(b));
                let y = gf256(a) / gf256(b);
                let z = gf256_barret(a) / gf256_barret(b);
                let w = gf256_table(a) / gf256_table(b);
                assert_eq!(u8::from(x), u8::from(y));
                assert_eq!(u8::from(x), u8::from(z));
                assert_eq!(u8::from(x), u8::from(w));
            }
        }
    }

    #[test]
    fn recip() {
        // test all reciprocals
        for a in (1..=255).map(gf256) {
            let x = a.naive_recip();
            let y = a.recip();
            let z = a.naive_pow(254);
            let w = a.pow(254);
            let v = gf256(1).naive_div(a);
            let u = gf256(1) / a;
            assert_eq!(x, y);
            assert_eq!(x, z);
            assert_eq!(x, w);
            assert_eq!(x, v);
            assert_eq!(x, u);
        }
    }

    #[test]
    fn mul_div() {
        // test that div is the inverse of mul
        for a in (1..=255).map(gf256) {
            for b in (1..=255).map(gf256) {
                let c = a * b;
                assert_eq!(c / b, a);
                assert_eq!(c / a, b);
            }
        }
    }

    #[test]
    fn pow() {
        // gf256::naive_pow just uses gf256::naive_mul, we want
        // to test with a truely naive pow
        fn naive_pow(a: gf256, exp: u32) -> gf256 {
            let mut x = gf256(1);
            for _ in 0..exp {
                x *= a;
            }
            x
        }

        for a in (0..=255).map(gf256) {
            for b in 0..=255 {
                assert_eq!(a.pow(b), naive_pow(a, b));
            }
        }
    }
}
