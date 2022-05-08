//! Shamir's secret-sharing scheme using our Galois-field types
//!
//! Shamir's secret-sharing scheme is an algorithm for splitting a secret into
//! some number of shares `n`, such that you need at minimum some number of
//! shares `k`, to reconstruct the original secret.
//!
//! More information on how Shamir's secret-sharing scheme works can be found
//! in [`shamir`'s module-level documentation][shamir-mod].
//!
//! [shamir-mod]: https://docs.rs/gf256/latest/gf256/shamir

use rand;
use rand::Rng;
use std::convert::TryFrom;
use ::gf256::gf;


// We could use the default gf256 type available in the gf256 crate, but
// it defaults to a table-based implementation, which risks leaking timing
// information due to caching
//
// Instead we use a Barret implementation here, which provide constant-time
// operations, but is slower
//
#[gf(polynomial=0x11d, generator=0x02, barret)]
type gf256;


/// Generate a random polynomial of a given degree, fixing f(0) = secret
fn shamir_poly_random(secret: gf256, degree: usize) -> Vec<gf256> {
    let mut rng = rand::thread_rng();
    let mut f = vec![secret];
    for _ in 0..degree {
        f.push(gf256(rng.gen_range(1..=255)));
    }
    f
}

/// Evaluate a polynomial at x using Horner's method
fn shamir_poly_eval(f: &[gf256], x: gf256) -> gf256 {
    let mut y = gf256(0);
    for c in f.iter().rev() {
        y = y*x + c;
    }
    y
}

/// Find f(0) using Lagrange interpolation
fn shamir_poly_interpolate(xs: &[gf256], ys: &[gf256]) -> gf256 {
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
        let f = shamir_poly_random(gf256(*x), k-1);

        // assign each share with a point at f(i)
        for i in 0..n {
            shares[i].push(u8::from(
                shamir_poly_eval(&f, gf256::try_from(i+1).unwrap())
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
        secret.push(u8::from(shamir_poly_interpolate(&xs, &ys)));
    }

    secret
}


fn main() {
    fn hex(xs: &[u8]) -> String {
        xs.iter()
            .map(|x| format!("{:02x}", x))
            .collect()
    }

    fn ascii(xs: &[u8]) -> String {
        xs.iter()
            .map(|x| {
                if *x < b' ' || *x > b'~' {
                    '.'
                } else {
                    char::from(*x)
                }
            })
            .collect::<String>()
    }


    let input = b"Hello World!";
    println!();
    println!("testing shamir({:?})", String::from_utf8_lossy(input));

    let shares = shamir_generate(input, 5, 4);
    println!("{} => {}  {}", "generate share1", ascii(&shares[0]), hex(&shares[0]));
    println!("{} => {}  {}", "generate share2", ascii(&shares[1]), hex(&shares[1]));
    println!("{} => {}  {}", "generate share3", ascii(&shares[2]), hex(&shares[2]));
    println!("{} => {}  {}", "generate share4", ascii(&shares[3]), hex(&shares[3]));
    println!("{} => {}  {}", "generate share5", ascii(&shares[4]), hex(&shares[4]));

    let output = shamir_reconstruct(&shares[..1]);
    println!("{} => {}  {}", "reconstruct 1 shares", ascii(&output), hex(&output));
    assert_ne!(output, input);

    let output = shamir_reconstruct(&shares[..2]);
    println!("{} => {}  {}", "reconstruct 2 shares", ascii(&output), hex(&output));
    assert_ne!(output, input);

    let output = shamir_reconstruct(&shares[..3]);
    println!("{} => {}  {}", "reconstruct 3 shares", ascii(&output), hex(&output));
    assert_ne!(output, input);

    let output = shamir_reconstruct(&shares[..4]);
    println!("{} => {}  {}", "reconstruct 4 shares", ascii(&output), hex(&output));
    assert_eq!(output, input);

    let output = shamir_reconstruct(&shares[..5]);
    println!("{} => {}  {}", "reconstruct 5 shares", ascii(&output), hex(&output));
    assert_eq!(output, input);

    println!();
}
