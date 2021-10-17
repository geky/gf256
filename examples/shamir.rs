//! Shamir's secret sharing using the Galois-field types
//!
//! The main idea behind Shamir's secret sharing is to represent a secret
//! as a point in an unknown polynomial, where each participant holds a
//! different point on the polynomial. By adjusting the degree of the
//! polynomial, you can change how many participant's points are required in
//! order to find the original polynomial, and the original point.
//!
//! To make this scheme usable with arbitrary bytes, we can represent the
//! polynomial in a Galois-field using our gf256 type. This way all points
//! in our polynomial "wrap around" the field, remaining representable in a
//! byte.
//!
//! We repeat this scheme for each byte in the data, finding a different
//! polynomial for each byte. Each participant gets a share, prefixed with
//! an arbitrary x-coordinate, with each following byte being the y-coordinate
//! of that byte's polynomial
//!

use rand;
use rand::Rng;
use std::convert::TryFrom;
use ::gf256::*;


/// Generate a random polynomial of a given degree, fixing f(0) = secret
fn shamir_random_poly(secret: gf256, degree: usize) -> Vec<gf256> {
    let mut rng = rand::thread_rng();
    let mut f = vec![secret];
    for _ in 0..degree {
        f.push(gf256(rng.gen_range(1..=255)));
    }
    f
}

/// Evaluate a polynomial at x using Horner's method
fn shamir_eval_poly(f: &[gf256], x: gf256) -> gf256 {
    let mut y = gf256(0);
    for c in f.iter().rev() {
        y = y*x + c;
    }
    y
}

/// Find f(0) using Lagrange interpolation
fn shamir_interpolate_poly(xs: &[gf256], ys: &[gf256]) -> gf256 {
    assert!(xs.len() == ys.len());

    let mut y = gf256(0);
    for (i, (x0, y0)) in xs.iter().zip(ys).enumerate() {
        let mut li = gf256(1);
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
pub fn shamir_generate(secret: &[u8], n: usize, k: usize) -> Vec<Vec<u8>> {
    // only support up to 255 shares
    assert!(n <= 255, "exceeded 255 shares");
    let mut shares = vec![vec![]; n];

    // we need to store the x coord somewhere, so just prepend the share with it
    for i in 0..n {
        shares[i].push(u8::try_from(i+1).unwrap());
    }

    for x in secret {
        // generate a random polynomial for each byte
        let f = shamir_random_poly(gf256(*x), k-1);

        // assign each share with a point at f(i)
        for i in 0..n {
            shares[i].push(u8::from(
                shamir_eval_poly(&f, gf256::try_from(i+1).unwrap())
            ));
        }
    }

    shares
}

/// Reconstruct a secret
pub fn shamir_reconstruct<S: AsRef<[u8]>>(shares: &[S]) -> Vec<u8> {
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
    let xs = shares.iter().map(|s| gf256(s.as_ref()[0])).collect::<Vec<_>>();
    for i in 1..len {
        let ys = shares.iter().map(|s| gf256(s.as_ref()[i])).collect::<Vec<_>>();
        secret.push(u8::from(shamir_interpolate_poly(&xs, &ys)));
    }

    secret
}


fn main() {
    fn hex(xs: &[u8]) -> String {
        xs.iter()
            .map(|x| format!("{:02x}", x))
            .collect()
    }

    let input = b"Hello World!";
    println!();
    println!("{:<6} => {:?}", "input", String::from_utf8_lossy(input));

    let shares = shamir_generate(input, 5, 4);
    println!("{:<6} => {}", "share1", hex(&shares[0]));
    println!("{:<6} => {}", "share2", hex(&shares[1]));
    println!("{:<6} => {}", "share3", hex(&shares[2]));
    println!("{:<6} => {}", "share4", hex(&shares[3]));
    println!("{:<6} => {}", "share5", hex(&shares[4]));

    let output = shamir_reconstruct(&shares[..1]);
    println!("{:<16} => {:?}", "output(1 shares)", String::from_utf8_lossy(&output));
    assert_ne!(output, input);

    let output = shamir_reconstruct(&shares[..2]);
    println!("{:<16} => {:?}", "output(2 shares)", String::from_utf8_lossy(&output));
    assert_ne!(output, input);

    let output = shamir_reconstruct(&shares[..3]);
    println!("{:<16} => {:?}", "output(3 shares)", String::from_utf8_lossy(&output));
    assert_ne!(output, input);

    let output = shamir_reconstruct(&shares[..4]);
    println!("{:<16} => {:?}", "output(4 shares)", String::from_utf8_lossy(&output));
    assert_eq!(output, input);

    let output = shamir_reconstruct(&shares[..5]);
    println!("{:<16} => {:?}", "output(5 shares)", String::from_utf8_lossy(&output));
    assert_eq!(output, input);

    println!();
}
