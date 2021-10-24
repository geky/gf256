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
//! generators for GF(2^16):
//!
//! RUSTFLAGS="-Ctarget-cpu=native" cargo +nightly run --release --features use-nightly-features --example find-p -- --width=17 -m=1
//!

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
pub fn is_irreducible(p: p128) -> bool {
    // some corner cases
    if p == p128(0) || p == p128(1) {
        return false;
    }

    // test division of all polynomials < sqrt(p), or a simpler
    // heuristic of < 2^(log2(p)/2)
    let npw2 = 128 - (u128::from(p)-1).leading_zeros();
    let roughsqrt = 1u128 << ((npw2+1)/2);

    for x in 2..roughsqrt {
        if p % p128(x) == p128(0) {
            return false;
        }
    }

    true
}

/// Find all irreducible polynomials of a given bit-width
pub fn irreducibles(width: usize) -> impl Iterator<Item=p128> {
    // find irreducible polynomials via brute force
    ((1u128 << (width-1)) .. (1u128 << width))
        .map(p128)
        .filter(|p| is_irreducible(*p))
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

    let width = (128-p.leading_zeros()) - 1;

    // We could naively calculate x' = (g*x)%p each iteration, however
    // we need to do this a lot, so it's better to precalculate Barret's
    // constant for Barret reduction. This has more multiplications, but
    // this way we can leveral polynomial-multiplication hardware.
    //
    // normally this is just (1 << (2*width)) / p, but we can precompute
    // one step of division to avoid needing a 4x wide type
    //
    let mask = (1u128 << width) - 1;
    let barret_constant = (((mask & p) << width) / p) + (p128(1) << width);
    let mut mgroup = iter::successors(
        Some(p128(1)),
        |x| {
            let x = g * x;
            let q = ((x >> width) * barret_constant) >> width;
            Some(mask & ((q * p) + x))
        }
    );

    // Use a simple cycle detection algorithm, Brent's algorithm,
    // to find the cycle of the multiplicative group. This has the benefit
    // of terminating early if a smaller cycle is found
    let mut pw2 = 1;
    let mut len = 1;
    let mut tortoise = mgroup.next().unwrap();
    let mut hare = mgroup.next().unwrap();
    while tortoise != hare {
        if pw2 == len {
            tortoise = hare;
            pw2 *= 2;
            len = 0;
        }
        hare = mgroup.next().unwrap();
        len += 1;
    }

    // if len(multiplicative cycle) == field size-1, we found a generator
    let field_size = 1u128 << width;
    debug_assert!(len <= field_size-1);
    len == field_size-1
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

    let ps: Box<dyn Iterator<Item=p128>> = match opt.polynomial {
        Some(p) => Box::new(iter::once(p)),
        None    => Box::new(((1u128 << (width-1)) .. (1u128 << width)).map(p128)),
    };

    let gs = || -> Box<dyn Iterator<Item=p128>> {
        match opt.generator {
            Some(g) => Box::new(iter::once(g)),
            None    => Box::new((0 .. (1u128 << (width-1))).map(p128)),
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

        if p_is_irreducible {
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
