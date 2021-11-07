
use crate::macros::gf;

// Galois field type
#[gf(polynomial=0x11d, generator=0x02)]
pub type gf256;


#[cfg(test)]
mod test {
    use super::*;
    use crate::p::*;

    // Create a custom gf type here (Rijndael's finite field) to test a
    // different polynomial
    #[gf(polynomial=0x11b, generator=0x03)]
    type gf256_rijndael;

    // Test both table-based and Barret reduction implementations
    #[gf(polynomial=0x11d, generator=0x02, log_table)]
    type gf256_log_table;
    #[gf(polynomial=0x11d, generator=0x02, rem_table)]
    type gf256_rem_table;
    #[gf(polynomial=0x11d, generator=0x02, small_rem_table)]
    type gf256_small_rem_table;
    #[gf(polynomial=0x11d, generator=0x02, barret)]
    type gf256_barret;

    #[test]
    fn add() {
        assert_eq!(gf256(0x12).naive_add(gf256(0x34)), gf256(0x26));
        assert_eq!(gf256_rijndael(0x12).naive_add(gf256_rijndael(0x34)), gf256_rijndael(0x26));

        assert_eq!(gf256(0x12) + gf256(0x34), gf256(0x26));
        assert_eq!(gf256_rijndael(0x12) + gf256_rijndael(0x34), gf256_rijndael(0x26));

        assert_eq!(gf256_log_table(0x12).naive_add(gf256_log_table(0x34)), gf256_log_table(0x26));
        assert_eq!(gf256_rem_table(0x12).naive_add(gf256_rem_table(0x34)), gf256_rem_table(0x26));
        assert_eq!(gf256_small_rem_table(0x12).naive_add(gf256_small_rem_table(0x34)), gf256_small_rem_table(0x26));
        assert_eq!(gf256_barret(0x12).naive_add(gf256_barret(0x34)), gf256_barret(0x26));

        assert_eq!(gf256_log_table(0x12) + gf256_log_table(0x34), gf256_log_table(0x26));
        assert_eq!(gf256_rem_table(0x12) + gf256_rem_table(0x34), gf256_rem_table(0x26));
        assert_eq!(gf256_small_rem_table(0x12) + gf256_small_rem_table(0x34), gf256_small_rem_table(0x26));
        assert_eq!(gf256_barret(0x12) + gf256_barret(0x34), gf256_barret(0x26));
    }

    #[test]
    fn sub() {
        assert_eq!(gf256(0x12).naive_sub(gf256(0x34)), gf256(0x26));
        assert_eq!(gf256_rijndael(0x12).naive_sub(gf256_rijndael(0x34)), gf256_rijndael(0x26));

        assert_eq!(gf256(0x12) - gf256(0x34), gf256(0x26));
        assert_eq!(gf256_rijndael(0x12) - gf256_rijndael(0x34), gf256_rijndael(0x26));

        assert_eq!(gf256_log_table(0x12).naive_sub(gf256_log_table(0x34)), gf256_log_table(0x26));
        assert_eq!(gf256_rem_table(0x12).naive_sub(gf256_rem_table(0x34)), gf256_rem_table(0x26));
        assert_eq!(gf256_small_rem_table(0x12).naive_sub(gf256_small_rem_table(0x34)), gf256_small_rem_table(0x26));
        assert_eq!(gf256_barret(0x12).naive_sub(gf256_barret(0x34)), gf256_barret(0x26));

        assert_eq!(gf256_log_table(0x12) - gf256_log_table(0x34), gf256_log_table(0x26));
        assert_eq!(gf256_rem_table(0x12) - gf256_rem_table(0x34), gf256_rem_table(0x26));
        assert_eq!(gf256_small_rem_table(0x12) - gf256_small_rem_table(0x34), gf256_small_rem_table(0x26));
        assert_eq!(gf256_barret(0x12) - gf256_barret(0x34), gf256_barret(0x26));
    }

    #[test]
    fn mul() {
        assert_eq!(gf256(0x12).naive_mul(gf256(0x34)), gf256(0x0f));
        assert_eq!(gf256_rijndael(0x12).naive_mul(gf256_rijndael(0x34)), gf256_rijndael(0x05));

        assert_eq!(gf256(0x12) * gf256(0x34), gf256(0x0f));
        assert_eq!(gf256_rijndael(0x12) * gf256_rijndael(0x34), gf256_rijndael(0x05));

        assert_eq!(gf256_log_table(0x12).naive_mul(gf256_log_table(0x34)), gf256_log_table(0x0f));
        assert_eq!(gf256_rem_table(0x12).naive_mul(gf256_rem_table(0x34)), gf256_rem_table(0x0f));
        assert_eq!(gf256_small_rem_table(0x12).naive_mul(gf256_small_rem_table(0x34)), gf256_small_rem_table(0x0f));
        assert_eq!(gf256_barret(0x12).naive_mul(gf256_barret(0x34)), gf256_barret(0x0f));

        assert_eq!(gf256_log_table(0x12) * gf256_log_table(0x34), gf256_log_table(0x0f));
        assert_eq!(gf256_rem_table(0x12) * gf256_rem_table(0x34), gf256_rem_table(0x0f));
        assert_eq!(gf256_small_rem_table(0x12) * gf256_small_rem_table(0x34), gf256_small_rem_table(0x0f));
        assert_eq!(gf256_barret(0x12) * gf256_barret(0x34), gf256_barret(0x0f));
    }

    #[test]
    fn div() {
        assert_eq!(gf256(0x12).naive_div(gf256(0x34)), gf256(0xc7));
        assert_eq!(gf256_rijndael(0x12).naive_div(gf256_rijndael(0x34)), gf256_rijndael(0x54));

        assert_eq!(gf256(0x12) / gf256(0x34), gf256(0xc7));
        assert_eq!(gf256_rijndael(0x12) / gf256_rijndael(0x34), gf256_rijndael(0x54));

        assert_eq!(gf256_log_table(0x12).naive_div(gf256_log_table(0x34)), gf256_log_table(0xc7));
        assert_eq!(gf256_rem_table(0x12).naive_div(gf256_rem_table(0x34)), gf256_rem_table(0xc7));
        assert_eq!(gf256_small_rem_table(0x12).naive_div(gf256_small_rem_table(0x34)), gf256_small_rem_table(0xc7));
        assert_eq!(gf256_barret(0x12).naive_div(gf256_barret(0x34)), gf256_barret(0xc7));

        assert_eq!(gf256_log_table(0x12) / gf256_log_table(0x34), gf256_log_table(0xc7));
        assert_eq!(gf256_rem_table(0x12) / gf256_rem_table(0x34), gf256_rem_table(0xc7));
        assert_eq!(gf256_small_rem_table(0x12) / gf256_small_rem_table(0x34), gf256_small_rem_table(0xc7));
        assert_eq!(gf256_barret(0x12) / gf256_barret(0x34), gf256_barret(0xc7));
    }

    #[test]
    fn all_mul() {
        // test all multiplications
        for a in 0..=255 {
            for b in 0..=255 {
                let x = gf256(a).naive_mul(gf256(b));
                let y = gf256(a) * gf256(b);
                let z = gf256_barret(a) * gf256_barret(b);
                let w = gf256_log_table(a) * gf256_log_table(b);
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
                let w = gf256_log_table(a) / gf256_log_table(b);
                assert_eq!(u8::from(x), u8::from(y));
                assert_eq!(u8::from(x), u8::from(z));
                assert_eq!(u8::from(x), u8::from(w));
            }
        }
    }

    #[test]
    fn all_recip() {
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

    macro_rules! test_axioms {
        ($name:ident; $t:ty; $gf:expr; $nz:expr; $x:expr) => {
            #[test]
            fn $name() {
                assert_eq!(<$t>::NONZEROS, $nz);

                let xs = [
                    $gf(1*$x),
                    $gf(2*$x),
                    $gf(3*$x),
                    $gf(4*$x),
                ];

                for x in xs {
                    for y in xs {
                        for z in xs {
                            // 0 is the identity of addition
                            assert_eq!(x + $gf(0), x);
                            // 1 is the identity of multiplication
                            assert_eq!(x * $gf(1), x);
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
        }
    }

    test_axioms! { gf16_axioms;    gf16;   |x| gf16::new(x).unwrap(); 15;  0x1 }
    test_axioms! { gf256_axioms;   gf256;  gf256; 255; 0x11 }
    test_axioms! { gf4096_axioms;  gf4096; |x| gf4096::new(x).unwrap(); 4095; 0x111 }
    test_axioms! { gf2p16_axioms;  gf2p16; gf2p16; 65535; 0x1111 }
    test_axioms! { gf2p32_axioms;  gf2p32; gf2p32; 4294967295; 0x11111111 }
    test_axioms! { gf2p64_axioms;  gf2p64; gf2p64; 18446744073709551615; 0x1111111111111111 }

    // Test with explicit implementations
    //
    // This introduces a lot of things to compile, but is important to cover
    // niche implementations that are very prone to bugs
    //

    #[gf(polynomial=0x13, generator=0x2, log_table)]
    type gf16_log_table;

    test_axioms! { gf16_log_table_axioms;    gf16_log_table;   |x| gf16_log_table::new(x).unwrap(); 15;  0x1 }
    test_axioms! { gf256_log_table_axioms;   gf256_log_table;  gf256_log_table; 255; 0x11 }

    #[gf(polynomial=0x13, generator=0x2, rem_table)]
    type gf16_rem_table;
    #[gf(polynomial=0x1009, generator=0x003, rem_table)]
    type gf4096_rem_table;
    #[gf(polynomial=0x1002b, generator=0x0003, rem_table)]
    type gf2p16_rem_table;
    #[gf(polynomial=0x10000008d, generator=0x00000003, rem_table)]
    type gf2p32_rem_table;
    #[gf(polynomial=0x1000000000000001b, generator=0x0000000000000002, rem_table)]
    type gf2p64_rem_table;

    test_axioms! { gf16_rem_table_axioms;    gf16_rem_table;   |x| gf16_rem_table::new(x).unwrap(); 15;  0x1 }
    test_axioms! { gf256_rem_table_axioms;   gf256_rem_table;  gf256_rem_table; 255; 0x11 }
    test_axioms! { gf4096_rem_table_axioms;  gf4096_rem_table; |x| gf4096_rem_table::new(x).unwrap(); 4095; 0x111 }
    test_axioms! { gf2p16_rem_table_axioms;  gf2p16_rem_table; gf2p16_rem_table; 65535; 0x1111 }
    test_axioms! { gf2p32_rem_table_axioms;  gf2p32_rem_table; gf2p32_rem_table; 4294967295; 0x11111111 }
    test_axioms! { gf2p64_rem_table_axioms;  gf2p64_rem_table; gf2p64_rem_table; 18446744073709551615; 0x1111111111111111 }

    #[gf(polynomial=0x13, generator=0x2, small_rem_table)]
    type gf16_small_rem_table;
    #[gf(polynomial=0x1009, generator=0x003, small_rem_table)]
    type gf4096_small_rem_table;
    #[gf(polynomial=0x1002b, generator=0x0003, small_rem_table)]
    type gf2p16_small_rem_table;
    #[gf(polynomial=0x10000008d, generator=0x00000003, small_rem_table)]
    type gf2p32_small_rem_table;
    #[gf(polynomial=0x1000000000000001b, generator=0x0000000000000002, small_rem_table)]
    type gf2p64_small_rem_table;

    test_axioms! { gf16_small_rem_table_axioms;    gf16_small_rem_table;   |x| gf16_small_rem_table::new(x).unwrap(); 15;  0x1 }
    test_axioms! { gf256_small_rem_table_axioms;   gf256_small_rem_table;  gf256_small_rem_table; 255; 0x11 }
    test_axioms! { gf4096_small_rem_table_axioms;  gf4096_small_rem_table; |x| gf4096_small_rem_table::new(x).unwrap(); 4095; 0x111 }
    test_axioms! { gf2p16_small_rem_table_axioms;  gf2p16_small_rem_table; gf2p16_small_rem_table; 65535; 0x1111 }
    test_axioms! { gf2p32_small_rem_table_axioms;  gf2p32_small_rem_table; gf2p32_small_rem_table; 4294967295; 0x11111111 }
    test_axioms! { gf2p64_small_rem_table_axioms;  gf2p64_small_rem_table; gf2p64_small_rem_table; 18446744073709551615; 0x1111111111111111 }

    #[gf(polynomial=0x13, generator=0x2, barret)]
    type gf16_barret;
    #[gf(polynomial=0x1009, generator=0x003, barret)]
    type gf4096_barret;
    #[gf(polynomial=0x1002b, generator=0x0003, barret)]
    type gf2p16_barret;
    #[gf(polynomial=0x10000008d, generator=0x00000003, barret)]
    type gf2p32_barret;
    #[gf(polynomial=0x1000000000000001b, generator=0x0000000000000002, barret)]
    type gf2p64_barret;

    test_axioms! { gf16_barret_axioms;    gf16_barret;   |x| gf16_barret::new(x).unwrap(); 15;  0x1 }
    test_axioms! { gf256_barret_axioms;   gf256_barret;  gf256_barret; 255; 0x11 }
    test_axioms! { gf4096_barret_axioms;  gf4096_barret; |x| gf4096_barret::new(x).unwrap(); 4095; 0x111 }
    test_axioms! { gf2p16_barret_axioms;  gf2p16_barret; gf2p16_barret; 65535; 0x1111 }
    test_axioms! { gf2p32_barret_axioms;  gf2p32_barret; gf2p32_barret; 4294967295; 0x11111111 }
    test_axioms! { gf2p64_barret_axioms;  gf2p64_barret; gf2p64_barret; 18446744073709551615; 0x1111111111111111 }

    // all Galois-field params
    #[gf(
        polynomial=0x11d,
        generator=0x02,
        usize=false,
        u=u8,
        u2=u16,
        p=p8,
        p2=p16,
    )]
    type gf256_all_params;

    test_axioms! { gf_all_params; gf256_all_params; gf256_all_params; 255; 0x11 }
}
