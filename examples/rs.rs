//! Reed-Solomon error-correction codes (BCH-view), using our Galois-field types
//!
//! Reed-Solomon error-correction is a scheme for creating error-correction
//! codes (ECC) capable of detecting and correcting multiple byte-level errors.
//! By adding `n` extra bytes to a message, Reed-Solomon is able to correct up
//! to `n` byte-errors in known locations, called "erasures", and `n/2`
//! byte-errors in unknown locations, called "errors".
//!
//! Reed-Solomon accomplishes this by viewing the entire codeword (message + ecc)
//! as a polynomial in `GF(256)`, and limiting valid codewords to polynomials
//! that are a multiple of a special "generator polynomial" `G(x)`.
//!
//! More information on how Reed-Solomon error-correction codes work can be
//! found in [`rs`'s module-level documentation][rs-mod].
//!
//! [rs-mod]: https://docs.rs/gf256/latest/gf256/rs

#![allow(non_snake_case)]
#![allow(mixed_script_confusables)]

use std::convert::TryFrom;
use std::fmt;
use rand;
use rand::Rng;
use ::gf256::*;


// Constants for Reed-Solomon error correction
//
// Reed-Solomon can correct ECC_SIZE known erasures and ECC_SIZE/2 unknown
// erasures. DATA_SIZE is arbitrary, however the total size is limited to
// 255 bytes in a GF(256) field.
//
pub const DATA_SIZE:  usize = 223;
pub const ECC_SIZE:   usize = 32;
pub const BLOCK_SIZE: usize = DATA_SIZE + ECC_SIZE;

// The generator polynomial in Reed-Solomon is a polynomial with roots (f(x) = 0)
// at fixed points (g^i) in the finite-field.
//
//     ECC_SIZE
// G(x) = ∏ (x - g^i)
//        i
//
// Note that G(g^i) = 0 when i < ECC_SIZE, and that this holds for any
// polynomial * G(x). And we can make a message polynomial a multiple of G(x)
// by appending the remainder, message % G(x), much like CRC.
//
// Thanks to Rust's const evaluation, we can, and do, evaluate this at
// compile time. However, this has a tendency to hit the limit of
// const_eval_limit for large values of ECC_SIZE.
//
// The only current workaround for this is nightly + #![feature(const_eval_limit="0")].
//
// See:
// https://github.com/rust-lang/rust/issues/67217
//
pub const GENERATOR_POLY: [gf256; ECC_SIZE+1] = {
    let mut g = [gf256(0); ECC_SIZE+1];
    g[ECC_SIZE] = gf256(1);

    // find G(x)
    //
    //     ECC_SIZE
    // G(x) = ∏  (x - g^i)
    //        i
    //
    let mut i = 0usize;
    while i < ECC_SIZE {
        // x - g^i
        let root = [
            gf256(1),
            gf256::GENERATOR.naive_pow(i as u8),
        ];

        // G(x)*(x - g^i)
        let mut product = [gf256(0); ECC_SIZE+1];
        let mut j = 0usize;
        while j < i+1 {
            let mut k = 0usize;
            while k < root.len() {
                product[product.len()-1-(j+k)] = product[product.len()-1-(j+k)].naive_add(
                    g[g.len()-1-j].naive_mul(root[root.len()-1-k])
                );
                k += 1;
            }
            j += 1;
        }
        g = product;

        i += 1;
    }

    g
};


/// Error codes for Reed-Solomon
#[derive(Debug, Clone)]
pub enum RsError {
    /// Reed-Solomon can fail to decode if:
    /// - errors > ECC_SIZE/2
    /// - erasures > ECC_SIZE
    /// - 2*errors + erasures > ECC_SIZE
    ///
    TooManyErrors,
}

impl fmt::Display for RsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RsError::TooManyErrors => write!(f, "Too many errors to correct"),
        }
    }
}


/// Evaluate a polynomial at x using Horner's method
///
/// Note polynomials here are ordered biggest-coefficient first
///
fn rs_poly_eval(f: &[gf256], x: gf256) -> gf256 {
    let mut y = gf256(0);
    for c in f {
        y = y*x + c;
    }
    y
}

/// Multiply a polynomial by a scalar
fn rs_poly_scale(f: &mut [gf256], c: gf256) {
    for i in 0..f.len() {
        f[i] *= c;
    }
}

/// Add two polynomials together
fn rs_poly_add(f: &mut [gf256], g: &[gf256]) {
    debug_assert!(f.len() >= g.len());

    // note g.len() may be <= f.len()!
    for i in 0..f.len() {
        f[f.len()-1-i] += g[g.len()-1-i];
    }
}

/// Multiply two polynomials together
fn rs_poly_mul(f: &mut [gf256], g: &[gf256]) {
    debug_assert!(f[..g.len()-1].iter().all(|x| *x == gf256(0)));

    // This is in-place, at the cost of being a bit confusing,
    // note that we only write to i+j, and i+j is always >= i
    //
    // What makes this confusing is that f and g are both big-endian
    // polynomials, reverse order from what you would expect. And in
    // order to leverage the i+j non-overlap, we need to write to 
    // f in reverse-reverse order.
    //
    for i in (0..f.len()-g.len()+1).rev() {
        let fi = f[f.len()-1-i];
        f[f.len()-1-i] = gf256(0);

        for j in 0..g.len() {
            f[f.len()-1-(i+j)] += fi * g[g.len()-1-j];
        }
    }
}

/// Divide polynomials via synthetic division
///
/// Note both the quotient and remainder are left in the dividend
///
fn rs_poly_divrem(f: &mut [gf256], g: &[gf256]) {
    debug_assert!(f.len() >= g.len());

    // find leading coeff to normalize g, note you could avoid
    // this if g is already normalized
    let leading_coeff = g[0];

    for i in 0 .. (f.len() - g.len() + 1) {
        if f[i] != gf256(0) {
            f[i] /= leading_coeff;

            for j in 1..g.len() {
                f[i+j] -= f[i] * g[j];
            }
        }
    }
}

/// Encode using Reed-Solomon error correction
///
/// Much like in CRC, we want to make the message a multiple of G(x),
/// our generator polynomial. We can do this by appending the remainder
/// of our message after division by G(x).
///
/// ``` text
/// c(x) = m(x) - (m(x) % G(x))
/// ```
///
/// Note we expect the message to only take up the first message.len()-ECC_SIZE
/// bytes, but this can be smaller than BLOCK_SIZE
///
pub fn rs_encode(message: &mut [u8]) {
    assert!(message.len() <= BLOCK_SIZE);
    assert!(message.len() >= ECC_SIZE);
    let data_len = message.len() - ECC_SIZE;

    // create copy for polynomial division
    //
    // note if message is < DATA_SIZE we just treat it as a smaller polynomial,
    // this is equivalent to prepending zeros
    //
    let mut divrem = message.to_vec();
    divrem[data_len..].fill(0);

    // divide by our generator polynomial
    rs_poly_divrem(
        gf256::slice_from_slice_mut(&mut divrem),
        &GENERATOR_POLY
    );

    // return message + remainder, this new message is a polynomial
    // perfectly divisable by our generator polynomial
    message[data_len..].copy_from_slice(&divrem[data_len..]);
}

/// Find syndromes, which should be zero if there are no errors
///
/// ``` text
/// Si = c'(g^i)
/// ```
///
fn rs_find_syndromes(f: &[gf256]) -> Vec<gf256> {
    let mut S = vec![];
    for i in 0..ECC_SIZE {
        S.push(
            rs_poly_eval(f, gf256::GENERATOR.pow(u8::try_from(i).unwrap()))
        );
    }
    S
}

/// Find Forney syndromes, these hide known erasures from the original syndromes
/// so error detection doesn't try (and possibly fail) to find known erasures
///
fn rs_find_forney_syndromes(
    codeword: &[gf256],
    S: &[gf256],
    erasures: &[usize]
) -> Vec<gf256> {
    let mut S = S.to_vec();
    for j in erasures {
        let Xj = gf256::GENERATOR.pow(u8::try_from(codeword.len()-1-j).unwrap());
        for i in 0 .. S.len()-1 {
            S[i] = S[i+1] - S[i]*Xj;
        }
    }

    // trim unnecessary syndromes
    S.drain(S.len()-erasures.len()..);
    S
}

/// Find the error locator polynomial when we know the location of errors
///
/// ``` text
///
/// Λ(x) = ∏ (1 - Xk*x)
///        k
/// ```
///
fn rs_find_erasure_locator(codeword: &[gf256], erasures: &[usize]) -> Vec<gf256> {
    let mut Λ = vec![gf256(0); erasures.len()+1];
    let Λ_len = Λ.len();
    Λ[Λ_len-1] = gf256(1);

    for j in erasures {
        rs_poly_mul(&mut Λ, &[
            -gf256::GENERATOR.pow(u8::try_from(codeword.len()-1-j).unwrap()),
            gf256(1)
        ]);
    }

    Λ
}

/// Iteratively find the error locator polynomial using the
/// Berlekamp-Massey algorithm when we don't know the location of errors
///
fn rs_find_error_locator(S: &[gf256]) -> Vec<gf256> {
    // the current estimate for the error locator polynomial
    let mut Λ = vec![gf256(0); S.len()+1];
    let Λ_len = Λ.len();
    Λ[Λ_len-1] = gf256(1);

    let mut prev_Λ = Λ.clone();
    let mut delta_Λ = Λ.clone();

    // the current estimate for the number of errors
    let mut v = 0;

    for i in 0..S.len() {
        let mut delta = S[i];
        for j in 1..v+1 {
            delta += Λ[Λ.len()-1-j] * S[i-j];
        }

        prev_Λ.rotate_left(1);

        if delta != gf256(0) {
            if 2*v <= i {
                core::mem::swap(&mut Λ, &mut prev_Λ);
                rs_poly_scale(&mut Λ, delta);
                rs_poly_scale(&mut prev_Λ, delta.recip());
                v = i+1-v;
            }

            delta_Λ.copy_from_slice(&prev_Λ);
            rs_poly_scale(&mut delta_Λ, delta);
            rs_poly_add(&mut Λ, &delta_Λ);
        }
    }

    // trim leading zeros
    let zeros = Λ.iter().take_while(|x| **x == gf256(0)).count();
    Λ.drain(0..zeros);

    Λ
}

/// Find roots of the error locator polynomial by brute force
///
/// This just means we evaluate Λ(x) for all x locations in our
/// message, if they equal 0, aka are a root, then we found the
/// error location in our message.
///
fn rs_find_error_locations(codeword: &[gf256], Λ: &[gf256]) -> Vec<usize> {
    let mut error_locations = vec![];
    for j in 0..codeword.len() {
        let Xj = gf256::GENERATOR.pow(u8::try_from(codeword.len()-1-j).unwrap());
        let zero = rs_poly_eval(&Λ, Xj.recip());
        if zero == gf256(0) {
            // found an error location!
            error_locations.push(j);
        }
    }

    error_locations
}

/// Find the error magnitudes using Forney's algorithm
///
/// ``` text
///        Xj*Ω(Xj^-1)
/// Yj = - -----------
///         Λ'(Xj^-1)
/// ```
///
/// Where Ω(x) is the error evaluator polynomial:
///
/// ``` text
/// Ω(x) = S(x)*Λ(x) mod x^2v
/// ```
/// 
/// And S(x) is the partial syndrome polynomial:
/// 
/// ``` text
///       2v
/// S(x) = Σ Si*x^i
///        i
/// ```
///
/// And Λ’(x) is the formal derivative of Λ(x):
///
/// ``` text
///         v
/// Λ'(x) = Σ i*Λi*x^(i-1)
///        i=1
/// ```
///
fn rs_find_error_magnitudes(
    codeword: &[gf256],
    S: &[gf256],
    Λ: &[gf256],
    error_locations: &[usize]
) -> Vec<gf256> {
    // find the erasure evaluator polynomial
    //
    // Ω(x) = S(x)*Λ(x) mod x^2v
    //
    let mut Ω = vec![gf256(0); S.len()+Λ.len()-1];
    let Ω_len = Ω.len();
    Ω[Ω_len-S.len()..].copy_from_slice(&S);
    Ω[Ω_len-S.len()..].reverse();
    rs_poly_mul(&mut Ω, &Λ);
    Ω.drain(..Ω.len()-S.len());

    // find the formal derivative of Λ
    //
    // Λ'(x) = Σ i*Λi*x^(i-1)
    //        i=1
    //
    let mut Λ_prime = vec![gf256(0); Λ.len()-1];
    for i in 1..Λ.len() {
        let mut sum = gf256(0);
        for _ in 0..i {
            sum += Λ[Λ.len()-1-i];
        }
        let Λ_prime_len = Λ_prime.len();
        Λ_prime[Λ_prime_len-1-(i-1)] = sum;
    }

    // find the error magnitudes
    //
    //        Xj*Ω(Xj^-1)
    // Yj = - -----------
    //         Λ'(Xj^-1)
    //
    // we need to be careful to avoid a divide-by-zero here, this can happen
    // in some cases (provided with incorrect erasures?)
    //
    let mut error_magnitudes = vec![];
    for j in error_locations {
        let Xj = gf256::GENERATOR.pow(u8::try_from(codeword.len()-1-j).unwrap());
        let Yj = (-Xj*rs_poly_eval(&Ω, Xj.recip()))
            .checked_div(rs_poly_eval(&Λ_prime, Xj.recip()))
            .unwrap_or(gf256(0));
        error_magnitudes.push(Yj);
    }

    error_magnitudes
}

/// Determine if message is correct
///
/// Note this is quite a bit faster than correcting the errors
///
pub fn rs_is_correct(codeword: &[u8]) -> bool {
    let codeword = gf256::slice_from_slice(codeword);

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = rs_find_syndromes(codeword);
    syndromes.iter().all(|s| *s == gf256(0))
}

/// Correct up to ECC_SIZE erasures at known locations
///
pub fn rs_correct_erasures(
    codeword: &mut [u8],
    erasures: &[usize]
) -> Result<usize, RsError> {
    let codeword = gf256::slice_from_slice_mut(codeword);

    // too many erasures?
    if erasures.len() > ECC_SIZE {
        return Err(RsError::TooManyErrors);
    }

    // find syndromes, syndromes of all zero means there are no errors
    let S = rs_find_syndromes(codeword);
    if S.iter().all(|s| *s == gf256(0)) {
        return Ok(0);
    }

    // find erasure locator polynomial
    let Λ = rs_find_erasure_locator(codeword, &erasures);

    // find erasure magnitudes using Forney's algorithm
    let erasure_magnitudes = rs_find_error_magnitudes(
        codeword,
        &S,
        &Λ,
        &erasures,
    );

    // correct the errors
    for (&Xj, Yj) in erasures.iter().zip(erasure_magnitudes) {
        codeword[Xj] += Yj;
    }

    // re-find the syndromes to check if we were able to find all errors
    let S = rs_find_syndromes(codeword);
    if !S.iter().all(|s| *s == gf256(0)) {
        return Err(RsError::TooManyErrors);
    }

    Ok(erasures.len())
}

/// Correct up to ECC_SIZE/2 errors at unknown locations
///
pub fn rs_correct_errors(codeword: &mut [u8]) -> Result<usize, RsError> {
    let codeword = gf256::slice_from_slice_mut(codeword);

    // find syndromes, syndromes of all zero means there are no errors
    let S = rs_find_syndromes(codeword);
    if S.iter().all(|s| *s == gf256(0)) {
        return Ok(0);
    }

    // find error locator polynomial
    let Λ = rs_find_error_locator(&S);

    // too many errors?
    let error_count = Λ.len() - 1;
    if error_count*2 > ECC_SIZE {
        return Err(RsError::TooManyErrors);
    }

    // find error locations
    let error_locations = rs_find_error_locations(codeword, &Λ);

    // find erasure magnitude using Forney's algorithm
    let error_magnitudes = rs_find_error_magnitudes(
        codeword,
        &S,
        &Λ,
        &error_locations,
    );

    // correct the errors
    for (&Xj, Yj) in error_locations.iter().zip(error_magnitudes) {
        codeword[Xj] += Yj;
    }

    // re-find the syndromes to check if we were able to find all errors
    let S = rs_find_syndromes(codeword);
    if !S.iter().all(|s| *s == gf256(0)) {
        return Err(RsError::TooManyErrors);
    }

    Ok(error_locations.len())
}

/// Correct a mixture of erasures at unknown locations and erasures
/// as known locations, can correct up to 2*errors+erasures <= ECC_SIZE
///
pub fn rs_correct(
    codeword: &mut [u8],
    erasures: &[usize]
) -> Result<usize, RsError> {
    let codeword = gf256::slice_from_slice_mut(codeword);

    // too many erasures?
    if erasures.len() > ECC_SIZE {
        return Err(RsError::TooManyErrors);
    }

    // find syndromes, syndromes of all zero means there are no errors
    let S = rs_find_syndromes(codeword);
    if S.iter().all(|s| *s == gf256(0)) {
        return Ok(0);
    }

    // find Forney syndromes, hiding known erasures from the syndromes
    let forney_S = rs_find_forney_syndromes(codeword, &S, &erasures);

    // find error locator polynomial
    let Λ = rs_find_error_locator(&forney_S);

    // too many errors/erasures?
    let error_count = Λ.len() - 1;
    let erasure_count = erasures.len();
    if error_count*2 + erasure_count > ECC_SIZE {
        return Err(RsError::TooManyErrors);
    }

    // find all error locations
    let mut error_locations = rs_find_error_locations(codeword, &Λ);
    error_locations.extend_from_slice(&erasures);

    // re-find error locator polynomial, this time including both 
    // errors and erasures
    let Λ = rs_find_erasure_locator(codeword, &error_locations);

    // find erasure magnitude using Forney's algorithm
    let error_magnitudes = rs_find_error_magnitudes(
        codeword,
        &S,
        &Λ,
        &error_locations,
    );

    // correct the errors
    for (&Xj, Yj) in error_locations.iter().zip(error_magnitudes) {
        codeword[Xj] += Yj;
    }

    // re-find the syndromes to check if we were able to find all errors
    let S = rs_find_syndromes(codeword);
    if !S.iter().all(|s| *s == gf256(0)) {
        return Err(RsError::TooManyErrors);
    }

    Ok(error_locations.len())
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

    let orig_message = b"Hello World!";
    println!();
    println!("testing rs({:?})", ascii(orig_message));

    println!("dimension = ({},{}), {} errors, {} erasures",
        BLOCK_SIZE,
        DATA_SIZE,
        ECC_SIZE / 2,
        ECC_SIZE
    );

    println!("generator = {}",
        hex(&GENERATOR_POLY.iter()
            .map(|b| u8::from(*b))
            .collect::<Vec<_>>())
    );

    let mut message = vec![0u8; orig_message.len()+ECC_SIZE];
    message[..orig_message.len()].copy_from_slice(&orig_message[..]);
    rs_encode(&mut message);
    println!("{:<19} => {}  {}",
        "rs_encode",
        ascii(&message),
        hex(&message)
    );

    // we can correct up to ECC_SIZE erasures (known location)
    let mut rng = rand::thread_rng();
    let errors = rand::seq::index::sample(&mut rng, message.len(), ECC_SIZE).into_vec();
    for error in errors.iter() {
        message[*error] = b'x';
    }
    println!("{:<19} => {}  {}",
        format!("corrupted ({},{})", ECC_SIZE, 0),
        ascii(&message),
        hex(&message)
    );

    rs_correct_erasures(&mut message, &errors).unwrap();
    println!("{:<19} => {}  {}",
        "rs_correct_erasures",
        ascii(&message),
        hex(&message)
    );
    assert_eq!(
        &message[0..orig_message.len()],
        orig_message
    );

    // we can correct up to ECC_SIZE/2 errors (unknown locations)
    let mut rng = rand::thread_rng();
    let errors = rand::seq::index::sample(&mut rng, message.len(), ECC_SIZE/2).into_vec();
    for error in errors.iter() {
        message[*error] = b'x';
    }
    println!("{:<19} => {}  {}",
        format!("corrupted ({},{})", 0, ECC_SIZE/2),
        ascii(&message),
        hex(&message)
    );

    rs_correct_errors(&mut message).unwrap();
    println!("{:<19} => {}  {}",
        "rs_correct_errors",
        ascii(&message),
        hex(&message)
    );
    assert_eq!(
        &message[0..orig_message.len()],
        orig_message
    );

    // we can correct up to ECC_SIZE = erasures + 2*errors (knowns and unkonwns)
    let mut rng = rand::thread_rng();
    let erasure_count = rng.gen_range(0..ECC_SIZE);
    let errors = rand::seq::index::sample(&mut rng, message.len(),
            erasure_count + (ECC_SIZE-erasure_count)/2
        ).into_vec();
    for error in errors.iter() {
        message[*error] = b'x';
    }
    println!("{:<19} => {}  {}",
        format!("corrupted ({},{})", erasure_count, (ECC_SIZE-erasure_count)/2),
        ascii(&message),
        hex(&message)
    );

    rs_correct(&mut message, &errors[..erasure_count]).unwrap();
    println!("{:<19} => {}  {}",
        "rs_correct",
        ascii(&message),
        hex(&message)
    );
    assert_eq!(
        &message[0..orig_message.len()],
        orig_message
    );

    println!();

    // for fun lets render a corrected image
    fn dots<'a>(width: usize, dots: &'a [u8]) -> impl Iterator<Item=String> + 'a {
        fn todots(d: u8) -> String {
            fn tod(d: u8) -> char {
                match d & 0x3 {
                    0x0 => ' ',
                    0x1 => '\'',
                    0x2 => '.',
                    0x3 => ':',
                    _ => unreachable!(),
                }
            }
            [tod(d >> 6), tod(d >> 4), tod(d >> 2), tod(d >> 0)]
                .iter()
                .collect::<String>()
        }

        (0..dots.len())
            .step_by(width)
            .map(move |d| {
                dots[d..d+width]
                    .iter()
                    .map(|d| todots(*d))
                    .collect::<String>()
            })
    }

    let width = 16;
    let image = [
        0x00, 0x00, 0x00, 0x00, 0x0a, 0xff, 0xff, 0xfe, 0xa0, 0x00, 0x00, 0x00, 0x00, 0xbf, 0xaa, 0xfe,
        0x00, 0x00, 0x00, 0x02, 0xff, 0xff, 0xff, 0xff, 0xfe, 0x80, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x2f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf8, 0x00, 0x00, 0x00, 0xff, 0xff, 0xf4,
        0x00, 0x00, 0x02, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x23, 0xff, 0xff, 0xc0,
        0x00, 0x00, 0x0b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc0, 0x2a, 0x3c, 0xdf, 0xff, 0x40,
        0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfd, 0x4a, 0x1f, 0xc4, 0x00, 0x54, 0x00,
        0x00, 0x00, 0x3f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x5a, 0xff, 0xc4, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x3f, 0xff, 0xff, 0xff, 0xff, 0xff, 0x52, 0xbf, 0xf5, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x3f, 0xff, 0xff, 0xff, 0xff, 0x52, 0xbf, 0xf5, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x2b, 0x3f, 0xff, 0xff, 0xfd, 0x52, 0xbf, 0xf5, 0x2b, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x0a, 0xfd, 0x0f, 0xff, 0xfd, 0x4a, 0xff, 0xf5, 0x2b, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x3f, 0x40, 0x07, 0xf5, 0x2a, 0xff, 0xf5, 0xab, 0xff, 0xff, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xf4, 0x00, 0x00, 0xab, 0xff, 0xd4, 0xaf, 0xff, 0xff, 0xfd, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x2a, 0xbf, 0xff, 0x52, 0xbf, 0xff, 0xff, 0xff, 0xf4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x7f, 0xff, 0xd5, 0x00, 0xff, 0xff, 0xff, 0xff, 0xfd, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x05, 0xff, 0xff, 0xfd, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    // we're using multiple lines per block here for the sole reason of making
    // the result look nice
    let block = 64;
    let ecc_width = ((block+ECC_SIZE) / (block / width)) - width;

    // intersperse the ECC, just so it looks nice
    //
    // though in practice ECC is actually often interspersed to prevent
    // early failure from localized errors
    //
    let stripe = |encoded: &mut [u8], i: usize, slice: &[u8]| {
        for j in 0 .. block/width {
            encoded[i*(width+ecc_width) + j*(width+ecc_width)*image.len()/block
                .. i*(width+ecc_width) + j*(width+ecc_width)*image.len()/block + width]
                    .copy_from_slice(&slice[j*width .. (j+1)*width]);
            encoded[i*(width+ecc_width) + j*(width+ecc_width)*image.len()/block + width
                .. i*(width+ecc_width) + j*(width+ecc_width)*image.len()/block + width+ecc_width]
                    .copy_from_slice(&slice[block + j*ecc_width .. block + (j+1)*ecc_width]);
        }
    };

    let unstripe = |encoded: &[u8], i: usize| -> Vec<u8> {
        let mut slice = vec![0; block+ECC_SIZE];
        for j in 0 .. block/width {
            slice[j*width .. (j+1)*width]
                .copy_from_slice(&encoded[i*(width+ecc_width) + j*(width+ecc_width)*image.len()/block
                    .. i*(width+ecc_width) + j*(width+ecc_width)*image.len()/block + width]);
            slice[block + j*ecc_width .. block + (j+1)*ecc_width]
                .copy_from_slice(&encoded[i*(width+ecc_width) + j*(width+ecc_width)*image.len()/block + width
                    .. i*(width+ecc_width) + j*(width+ecc_width)*image.len()/block + width+ecc_width]);
        }
        slice
    };

    let correct = |encoded: &mut [u8]| -> Result<(), RsError> {
        for i in 0 .. encoded.len()/(block+ECC_SIZE) {
            let mut slice = unstripe(encoded, i);
            rs_correct_errors(&mut slice)?;
            stripe(encoded, i, &slice);
        }
        Ok(())
    };

    // encode our image
    let mut encoded = vec![0; image.len() + (image.len()/block)*ECC_SIZE];
    for i in 0 .. image.len()/block {
        let mut slice = vec![0; block + ECC_SIZE];
        for j in 0 .. block/width {
            slice[j*width .. (j+1)*width]
                .copy_from_slice(&image[i*width + j*width*image.len()/block
                    .. i*width + j*width*image.len()/block + width]);
        }
        rs_encode(&mut slice);
        stripe(&mut encoded, i, &slice);
    }

    // bit-errors, roughly simulating something like radiation damage
    //
    // we push this until we fail, and then show the last recoverable image
    //
    let orig = encoded.clone();
    let mut prev_errored = encoded.clone();
    let mut errors = 0;
    let mut rng = rand::thread_rng();
    loop {
        for _ in 0..errors {
            let error = rng.gen_range(0..8*encoded.len());
            let coord = error / 8;
            let bit = error % 8;
            encoded[coord] ^= 1 << bit;
        }

        let errored = encoded.clone();
        match correct(&mut encoded) {
            Ok(()) => {}
            Err(RsError::TooManyErrors) => {
                break
            }
        }

        prev_errored = errored;
        errors += 1;
    }

    println!("bit corrupted image (errors = {}/{}, {:.2}%):",
        errors,
        8*encoded.len(),
        100.0 * (errors as f64 / ((8*encoded.len()) as f64)));
    println!();
    for line in dots(width+ecc_width, &prev_errored) {
        println!("    {}", line);
    }
    println!();

    // byte-errors, more likely in physical mediums, such as CDs 
    //
    // we push this until we fail, and then show the last recoverable image
    //
    encoded = orig.clone();
    let mut prev_errored = encoded.clone();
    let mut errors = 0;
    let mut rng = rand::thread_rng();
    loop {
        for _ in 0..errors {
            let error = rng.gen_range(0..encoded.len());
            if encoded[error].count_ones() > 4 {
                encoded[error] = 0x00;
            } else {
                encoded[error] = 0xff;
            }
        }

        let errored = encoded.clone();
        match correct(&mut encoded) {
            Ok(()) => {}
            Err(RsError::TooManyErrors) => {
                break
            }
        }

        prev_errored = errored;
        errors += 1;
    }

    println!("byte corrupted image (errors = {}/{}, {:.2}%):",
        errors,
        encoded.len(),
        100.0 * (errors as f64 / (encoded.len() as f64)));
    println!();
    for line in dots(width+ecc_width, &prev_errored) {
        println!("    {}", line);
    }
    println!();

    // show the actual corrected image
    correct(&mut prev_errored).unwrap();

    println!("corrected:");
    println!();
    for line in dots(width+ecc_width, &prev_errored) {
        println!("    {}", line);
    }
    println!();


    // here's another variation, except this time we're also using a
    // parity-bit per byte to help locate the errors
    //
    // note the parity bits themselves can fail!
    //
    let encoded_size = encoded.len();
    let mkparity = |slice: &mut [u8]| {
        let encoded = &slice[..encoded_size];
        let parity = (0..encoded_size/8)
            .map(|i| {
                let mut p = 0;
                for j in 0..8 {
                    let coord = i + j*(encoded_size/8);
                    p |= (encoded[coord].count_ones() as u8 & 1) << j;
                }
                p
            })
            .collect::<Vec<_>>();
        slice[encoded_size..].copy_from_slice(&parity);
    };

    let chparity = |slice: &[u8]| -> Vec<usize> {
        let mut erasures = vec![];
        let encoded = &slice[..encoded_size];
        let parity = &slice[encoded_size..];
        for i in 0..encoded_size/8 {
            for j in 0..8 {
                let coord = i + j*(encoded_size/8);
                if (encoded[coord].count_ones() as u8 & 1)
                    != ((parity[i] >> j) & 1)
                {
                    erasures.push(coord);
                }
            }
        }
        erasures
    };

    let correct = |encoded: &mut [u8]| -> Result<(), RsError> {
        let erasures = chparity(encoded);
        let mut encoded_erasures = vec![0; encoded_size];
        for i in erasures {
            encoded_erasures[i] = 1;
        }

        for i in 0 .. encoded_size/(block+ECC_SIZE) {
            let mut slice = unstripe(&encoded, i);
            let slice_erasures = unstripe(&encoded_erasures, i);
            let erasures = slice_erasures.iter()
                .enumerate()
                .filter(|(_, erasure)| **erasure != 0)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            rs_correct(&mut slice, &erasures)?;
            stripe(encoded, i, &slice);
            mkparity(encoded);
        }
        Ok(())
    };

    encoded = orig.clone();
    encoded.resize(encoded_size + encoded_size/8, 0);
    mkparity(&mut encoded);
    let orig = encoded.clone();

    // bit-errors, roughly simulating something like radiation damage
    //
    // we push this until we fail, and then show the last recoverable image
    //
    let mut prev_errored = encoded.clone();
    let mut errors = 0;
    let mut rng = rand::thread_rng();
    loop {
        for _ in 0..errors {
            let error = rng.gen_range(0..8*encoded.len());
            let coord = error / 8;
            let bit = error % 8;
            encoded[coord] ^= 1 << bit;
        }

        let errored = encoded.clone();
        match correct(&mut encoded) {
            Ok(()) => {}
            Err(RsError::TooManyErrors) => {
                break
            }
        }

        prev_errored = errored;
        errors += 1;
    }

    println!("bit corrupted image (errors = {}/{}, {:.2}%):",
        errors,
        8*encoded.len(),
        100.0 * (errors as f64 / ((8*encoded.len()) as f64)));
    println!();
    for line in dots(width+ecc_width, &prev_errored) {
        println!("    {}", line);
    }
    println!();

    // byte-errors, more likely in physical mediums, such as CDs 
    //
    // we push this until we fail, and then show the last recoverable image
    //
    encoded = orig.clone();
    let mut prev_errored = encoded.clone();
    let mut errors = 0;
    let mut rng = rand::thread_rng();
    loop {
        for _ in 0..errors {
            // in theory our byte+parity-bit would be one word
            let error = rng.gen_range(0..encoded_size);
            if encoded[error].count_ones() > 4 {
                encoded[error] = 0x00;
                encoded[encoded_size + error%(encoded_size/8)]
                    &= !(1 << (error/(encoded_size/8)));
            } else {
                encoded[error] = 0xff;
                encoded[encoded_size + error%(encoded_size/8)]
                    |= 1 << (error/(encoded_size/8));
            }
        }

        let errored = encoded.clone();
        match correct(&mut encoded) {
            Ok(()) => {}
            Err(RsError::TooManyErrors) => {
                break
            }
        }

        prev_errored = errored;
        errors += 1;
    }

    println!("byte corrupted image (errors = {}/{}, {:.2}%):",
        errors,
        (encoded.len()*8/9),
        100.0 * (errors as f64 / ((encoded.len()*8/9) as f64)));
    println!();
    for line in dots(width+ecc_width, &prev_errored) {
        println!("    {}", line);
    }
    println!();

    // show the actual corrected image
    correct(&mut prev_errored).unwrap();

    println!("corrected:");
    println!();
    for line in dots(width+ecc_width, &prev_errored) {
        println!("    {}", line);
    }
    println!();
}
