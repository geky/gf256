
use crate::macros::gf;

// Galois field type
#[gf(polynomial=0x11d, generator=0x02)]
pub type gf256;


#[cfg(test)]
mod test {
    use super::*;

    // Create a custom gf type here (Rijndael's finite field) to test that
    // the generator search works
    #[gf(polynomial=0x11b, generator=0x03)]
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
        fn naive_pow(a: gf256, exp: u8) -> gf256 {
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

    // Test higher/lower order fields
    //
    // These polynomials/generators were all found using the find-p
    // program in the examples in the examples
    //
    #[gf(polynomial=0x13, generator=0x2)]
    type gf16;
    #[gf(polynomial=0x1009, generator=0x003)]
    type gf4096;
    #[gf(polynomial=0x1002b, generator=0x0003)]
    type gf2p16;
    #[gf(polynomial=0x10000008d, generator=0x00000003)]
    type gf2p32;
    #[gf(polynomial=0x1000000000000001b, generator=0x0000000000000002)]
    type gf2p64;

    #[test]
    fn axioms() {
        assert_eq!(gf256::NONZEROS, 255);

        let xs = [
            gf256(0x11),
            gf256(0x22),
            gf256(0x33),
            gf256(0x44),
        ];

        for x in xs {
            for y in xs {
                for z in xs {
                    // 0 is the identity of addition
                    assert_eq!(x + gf256(0), x);
                    // 1 is the identity of multiplication
                    assert_eq!(x * gf256(1), x);
                    // addition and subtraction are inverses
                    assert_eq!((x + y) - y, x);
                    // multiplication and division are inverses
                    assert_eq!((x * y) / y, x);
                    // addition is distributive over multiplication
                    assert_eq!(x*(y + z), x*y + x*z);
                    // haha math
                    assert_eq!((x+y).pow(2), x.pow(2) + y.pow(2));
                }
            }
        }
    }

    #[test]
    fn gf16_axioms() {
        assert_eq!(gf16::NONZEROS, 15);

        let xs = [
            gf16(0x1),
            gf16(0x2),
            gf16(0x3),
            gf16(0x4),
        ];

        for x in xs {
            for y in xs {
                for z in xs {
                    // 0 is the identity of addition
                    assert_eq!(x + gf16(0), x);
                    // 1 is the identity of multiplication
                    assert_eq!(x * gf16(1), x);
                    // addition and subtraction are inverses
                    assert_eq!((x + y) - y, x);
                    // multiplication and division are inverses
                    assert_eq!((x * y) / y, x);
                    // addition is distributive over multiplication
                    assert_eq!(x*(y+z), x*y + x*z);
                    // haha math
                    assert_eq!((x+y).pow(2), x.pow(2) + y.pow(2));
                }
            }
        }
    }

    #[test]
    fn gf4096_axioms() {
        assert_eq!(gf4096::NONZEROS, 4095);

        let xs = [
            gf4096(0x111),
            gf4096(0x222),
            gf4096(0x333),
            gf4096(0x444),
        ];

        for x in xs {
            for y in xs {
                for z in xs {
                    // 0 is the identity of addition
                    assert_eq!(x + gf4096(0), x);
                    // 1 is the identity of multiplication
                    assert_eq!(x * gf4096(1), x);
                    // addition and subtraction are inverses
                    assert_eq!((x + y) - y, x);
                    // multiplication and division are inverses
                    assert_eq!((x * y) / y, x);
                    // addition is distributive over multiplication
                    assert_eq!(x*(y+z), x*y + x*z);
                    // haha math
                    assert_eq!((x+y).pow(2), x.pow(2) + y.pow(2));
                }
            }
        }
    }

    #[test]
    fn gf2p16_axioms() {
        assert_eq!(gf2p16::NONZEROS, 65535);

        let xs = [
            gf2p16(0x1111),
            gf2p16(0x2222),
            gf2p16(0x3333),
            gf2p16(0x4444),
        ];

        for x in xs {
            for y in xs {
                for z in xs {
                    // 0 is the identity of addition
                    assert_eq!(x + gf2p16(0), x);
                    // 1 is the identity of multiplication
                    assert_eq!(x * gf2p16(1), x);
                    // addition and subtraction are inverses
                    assert_eq!((x + y) - y, x);
                    // multiplication and division are inverses
                    assert_eq!((x * y) / y, x);
                    // addition is distributive over multiplication
                    assert_eq!(x*(y+z), x*y + x*z);
                    // haha math
                    assert_eq!((x+y).pow(2), x.pow(2) + y.pow(2));
                }
            }
        }
    }

    #[test]
    fn gf2p32_axioms() {
        assert_eq!(gf2p32::NONZEROS, 4294967295);

        let xs = [
            gf2p32(0x11111111),
            gf2p32(0x22222222),
            gf2p32(0x33333333),
            gf2p32(0x44444444),
        ];

        for x in xs {
            for y in xs {
                for z in xs {
                    // 0 is the identity of addition
                    assert_eq!(x + gf2p32(0), x);
                    // 1 is the identity of multiplication
                    assert_eq!(x * gf2p32(1), x);
                    // addition and subtraction are inverses
                    assert_eq!((x + y) - y, x);
                    // multiplication and division are inverses
                    assert_eq!((x * y) / y, x);
                    // addition is distributive over multiplication
                    assert_eq!(x*(y+z), x*y + x*z);
                    // haha math
                    assert_eq!((x+y).pow(2), x.pow(2) + y.pow(2));
                }
            }
        }
    }

    #[test]
    fn gf2p64_axioms() {
        assert_eq!(gf2p64::NONZEROS, 18446744073709551615);

        let xs = [
            gf2p64(0x1111111111111111),
            gf2p64(0x2222222222222222),
            gf2p64(0x3333333333333333),
            gf2p64(0x4444444444444444),
        ];

        for x in xs {
            for y in xs {
                for z in xs {
                    // 0 is the identity of addition
                    assert_eq!(x + gf2p64(0), x);
                    // 1 is the identity of multiplication
                    assert_eq!(x * gf2p64(1), x);
                    // addition and subtraction are inverses
                    assert_eq!((x + y) - y, x);
                    // multiplication and division are inverses
                    assert_eq!((x * y) / y, x);
                    // addition is distributive over multiplication
                    assert_eq!(x*(y+z), x*y + x*z);
                    // haha math
                    assert_eq!((x+y).pow(2), x.pow(2) + y.pow(2));
                }
            }
        }
    }
}
