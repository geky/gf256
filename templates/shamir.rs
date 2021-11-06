//! Template for Shamir secret-sharing functions
//!
//! See examples/shamir.rs for a more detailed explanation of
//! where these implementations come from

use __crate::internal::cfg_if::cfg_if;
use __crate::internal::rand::Rng;
use __crate::traits::TryFrom;

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;


/// Generate a random polynomial of a given degree, fixing f(0) = secret
fn poly_random(secret: __gf, degree: usize) -> Vec<__gf> {
    let mut rng = __rng();
    let mut f = vec![secret];
    for _ in 0..degree {
        f.push(__gf(rng.gen_range(1..=255)));
    }
    f
}

/// Evaluate a polynomial at x using Horner's method
fn poly_eval(f: &[__gf], x: __gf) -> __gf {
    let mut y = __gf(0);
    for c in f.iter().rev() {
        y = y*x + c;
    }
    y
}

/// Find f(0) using Lagrange interpolation
fn poly_interpolate(xs: &[__gf], ys: &[__gf]) -> __gf {
    assert!(xs.len() == ys.len());

    let mut y = __gf(0);
    for (i, (x0, y0)) in xs.iter().zip(ys).enumerate() {
        let mut li = __gf(1);
        for (j, (x1, _y1)) in xs.iter().zip(ys).enumerate() {
            if i != j {
                li *= x1 / (x0+x1);
            }
        }

        y += li*y0;
    }

    y
}

/// Generate n shares requiring k shares to reconstruct
pub fn generate(secret: &[u8], n: usize, k: usize) -> Vec<Vec<u8>> {
    // only support up to 255 shares
    assert!(n <= 255, "exceeded 255 shares");
    let mut shares = vec![vec![]; n];

    // we need to store the x coord somewhere, so just prepend the share with it
    for i in 0..n {
        shares[i].push(u8::try_from(i+1).unwrap());
    }

    for x in secret {
        // generate a random polynomial for each byte
        let f = poly_random(__gf(*x), k-1);

        // assign each share with a point at f(i)
        for i in 0..n {
            shares[i].push(u8::from(
                poly_eval(&f, __gf::try_from(i+1).unwrap())
            ));
        }
    }

    shares
}

/// Reconstruct a secret
pub fn reconstruct<S: AsRef<[u8]>>(shares: &[S]) -> Vec<u8> {
    // matching lengths?
    assert!(
        shares.windows(2).all(|ss| ss[0].as_ref().len() == ss[1].as_ref().len()),
        "mismatched share length?"
    );

    let mut secret = vec![];
    let len = shares.get(0).map(|s| s.as_ref().len()).unwrap_or(0);
    if len == 0 {
        return secret;
    }

    // x is prepended to each share
    let xs = shares.iter().map(|s| __gf(s.as_ref()[0])).collect::<Vec<_>>();
    for i in 1..len {
        let ys = shares.iter().map(|s| __gf(s.as_ref()[i])).collect::<Vec<_>>();
        secret.push(u8::from(poly_interpolate(&xs, &ys)));
    }

    secret
}

