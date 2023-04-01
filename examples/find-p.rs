//! Find interesting polynomials
//!
//! This more a tool than an example, and is useful for finding the irreducible
//! polynomials and generator polynomials necessary to construct finite-fields
//! of different bit-widths
//!
//! The construction of a finite-field GF(2^n) can be done by defining the
//! operations +,-,*,% as operations on binary polynomials modulo an
//! irreducible polynomial with width n+1 (aka degree 2^n).
//!
//! A generator, aka primitive element, in the field is defined as an element
//! whose multiplicative cycle equals the 2^n-1. This means that successive
//! powers of the generator eventually generate all elements of the field
//! except 0. A generator is useful for creating log tables, and several
//! algorithms depend on a known generator in the field.
//!
//! For example, to find all irreducible polynomials and their minimum
//! generators for GF(2^32) (recall that a 2^n finite-field requires an
//! irreducible polynomial of width n+1):
//!
//! ``` bash
//! $ RUSTFLAGS="-Ctarget-cpu=native" cargo +nightly run --release --example find-p -- --width=33 -m=1
//! polynomial=0x10000008d, generator=0x3
//! polynomial=0x1000000af, generator=0x2
//! polynomial=0x1000000c5, generator=0x2
//! polynomial=0x1000000f5, generator=0x2
//! polynomial=0x100000125, generator=0x2
//! ...
//! ```
//!
//! More information about irreducible polynomials, generators, and their
//! use in constructing finite-fields can be found in [`gf`'s module-level
//! documentation][gf-mod].
//!
//! [gf-mod]: https://docs.rs/gf256/latest/gf256/gf

use std::iter;
use std::process;
use std::convert::TryFrom;
use std::io;
use std::io::Write;
use structopt::StructOpt;
use ::gf256::*;


/// Is a given polynomial irreducible?
///
/// This is roughly equivalent to asking if a number is prime
///
pub fn is_irreducible(p: p128) -> Option<p128> {
    // check for 2 so we can skip all multiples of 2, seems like
    // a minor optimization but speeds things up by ~2x
    if p % p128(2) == p128(0) {
        if p == p128(2) {
            return None;
        } else {
            return Some(p128(2));
        }
    }

    // test division of all polynomials < sqrt(p), or a simpler
    // heuristic of < 2^(log2(p)/2)
    let npw2 = 128 - (u128::from(p)-1).leading_zeros();
    let roughsqrt = 1u128 << ((npw2+1)/2);

    for x in (3..roughsqrt).step_by(2).map(p128) {
        if p % x == p128(0) {
            return Some(x);
        }
    }

    None
}

/// Find all irreducible polynomials of a given bit-width
pub fn irreducibles(width: usize) -> impl Iterator<Item=p128> {
    // find irreducible polynomials via brute force
    ((1u128 << (width-1)) .. (1u128 << width))
        .map(p128)
        .filter(|p| is_irreducible(*p).is_none())
}

#[cfg(test)]
#[test]
fn test_irreducibles() {
    // we know there are 30 irreducible polynomials in gf256
    assert_eq!(irreducibles(9).count(), 30);
}

/// Is a given polynomial a primitive element, aka generator, of the
/// finite-field defined by modulo the given irreducible polynomial?
///
/// That's a mouthful, the question being asked here is do successive
/// powers of the generator iterate over every non-zero element in the
/// finite-field defined by the given irreducible polynomial
///
pub fn is_generator(g: p128, p: p128) -> bool {
    if g == p128(0) {
        return false;
    }

    // Define a few operations over the finite field defined by the irreducible
    // polynomial p. Normally we could use our gf-types, except this tool
    // is being used to generate the polynomials for the gf-types, so...
    //
    let width = (128-p.leading_zeros()) - 1;

    // Multiplication uses carry-less multiplicatio modulo our irreducible
    // polynomial
    let gfmul = |a: p128, b: p128| -> p128 {
        (a * b) % p
    };

    // Exponentiation via squaring
    let gfpow = |mut a: p128, mut exp: u128| -> p128 {
        let mut x = p128(1);
        loop {
            if exp & 1 != 0 {
                x = gfmul(x, a);
            }

            exp >>= 1;
            if exp == 0 {
                return x;
            }
            a = gfmul(a, a);
        }
    };

    // We're trying to test if g generates a multiplicative cycle of
    // size n - 1, where n is the size of our field. For this to be
    // true, g^(n-1) = 1 and g^m != 1 for all m < n-1.
    //
    // However it turns out we don't need to test all m, just m < n-1
    // where (n-1)/m is a prime factor of n-1. This is because any
    // multiplicative group must divide the biggest multiplicative group
    // evenly.
    //
    let n = 1u128 << width;

    // Find prime factors
    let primes = |mut x: u128| {
        let mut prime = 2;
        iter::from_fn(move || {
            while prime <= x {
                if x % prime == 0 {
                    x /= prime;
                    return Some(prime);
                }

                prime += 1;
            }

            None
        })
    };

    // g^m != 1 for all m < n-1 where m is prime factor of n-1?
    //
    // note we can skip duplicate primes
    //
    let mut prev = 1;
    for prime in primes(n-1) {
        if prime != prev {
            prev = prime;

            if gfpow(g, (n-1)/prime) == p128(1) {
                return false;
            }
        }
    }

    // g^(n-1) = 1?
    gfpow(g, n-1) == p128(1)
}

/// Find all generators in a field defined by the given irreducible polynomial
pub fn generators(p: p128) -> impl Iterator<Item=p128> {
    let width = 128-p.leading_zeros();

    // find generators via brute force
    (0 .. (1u128 << (width-1)))
        .map(p128)
        .filter(move |g| is_generator(*g, p))
}

#[cfg(test)]
#[test]
fn test_generators() {
    // we know there are 128 primitive elements in gf256, and since all
    // representations of gf256 are isomorphic, the irreducible polynomial
    // shouldn't matter
    //
    // (we only check the first couple irreducible polynomials to make the test
    // run faster)
    //
    for p in irreducibles(9).take(3) {
        assert_eq!(generators(p).count(), 128);
    }
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all="kebab")]
struct Opt {
    /// Quiet mode, only output found polynomials
    #[structopt(short, long)]
    quiet: bool,

    /// Bit-width of polynomials to search for
    ///
    /// Note that an n-bit finite field needs an n+1-bit irreducible
    /// polynomial, so you would need --width=9 to find irreducible
    /// polynomials in gf(256)
    ///
    #[structopt(short, long, required_unless("polynomial"))]
    width: Option<usize>,

    /// Polynomial to use, if provided we test that the polynomial is
    /// irreducible, and then search for generators
    ///
    #[structopt(short, long)]
    polynomial: Option<p128>,

    /// Generator to use, if provided we test that the generator is a
    /// valid primitive element
    ///
    #[structopt(short, long, requires("polynomial"))]
    generator: Option<p128>,

    /// Number of polynomials to find
    #[structopt(short)]
    n: Option<usize>,

    /// Number of generators to find
    #[structopt(short)]
    m: Option<usize>,
}

fn main() {
    let opt = Opt::from_args();

    // note we don't use the iterators, this is so we can print some progress

    // find iterators of polynomials to test
    let width = match (opt.width, opt.polynomial) {
        (Some(width), _      ) => width,
        (_,           Some(p)) => usize::try_from(128-p.leading_zeros()).unwrap(),
        (None,        None   ) => unreachable!(),
    };

    let ps: Box<dyn Iterator<Item=p128>> = match (opt.polynomial, opt.n) {
        (Some(p), None   ) => Box::new(iter::once(p)),
        (Some(p), Some(_)) => Box::new((u128::from(p) .. (1u128 << width)).map(p128)),
        (None,    _      ) => Box::new(((1u128 << (width-1)) .. (1u128 << width)).map(p128)),
    };

    let gs = || -> Box<dyn Iterator<Item=p128>> {
        match (opt.generator, opt.m) {
            (Some(g), None   ) => Box::new(iter::once(g)),
            (Some(g), Some(_)) => Box::new((u128::from(g) .. (1u128 << (width-1))).map(p128)),
            (None,    _      ) => Box::new((0 .. (1u128 << (width-1))).map(p128)),
        }
    };

    let n = opt.n.unwrap_or(usize::MAX);
    let m = opt.m.unwrap_or(usize::MAX);
    let mut p_count = 0;
    let mut g_count = 0;

    // find irreducible polynomials via brute force
    for p in ps {
        if !opt.quiet {
            print!("testing polynomial={}...", p);
            io::stdout().flush().unwrap();
        }

        let p_is_irreducible = is_irreducible(p);

        if !opt.quiet {
            print!("\r\x1b[K");
        }

        // if we explicitly requested a polynomial, warn when the polynomial
        // is not irreducible
        if let (Some(p), Some(r)) = (opt.polynomial, p_is_irreducible) {
            println!("uh oh, polynomial {} divisible by {}", p, r);
        }

        if p_is_irreducible.is_none() {
            if m == 0 {
                println!("polynomial={}", p);
                continue;
            }

            // find generators via brute force
            for g in gs() {

                if !opt.quiet {
                    print!("testing polynomial={}, generator={}...", p, g);
                    io::stdout().flush().unwrap();
                }

                let g_is_generator = is_generator(g, p);

                if !opt.quiet {
                    print!("\r\x1b[K");
                }

                if g_is_generator {
                    println!("polynomial={}, generator={}", p, g);

                    g_count += 1;
                    if g_count >= m {
                        break;
                    }
                }
            }

            p_count += 1;
            if p_count >= n {
                break;
            }
        }
    }

    // found any?
    if p_count == 0 || (m > 0 && g_count == 0) {
        process::exit(1);
    }
}
