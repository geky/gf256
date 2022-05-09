// Template for Reed-Solomon error-correction functions
//
// See examples/rs.rs for a more detailed explanation of
// where these implementations come from

//! Reed-Solomon error-correction functions.
//!
//! ``` rust
//! # use gf256::rs::rs255w223;
//! #
//! // encode
//! let mut buf = b"Hello World!".to_vec();
//! buf.resize(buf.len()+32, 0u8);
//! rs255w223::encode(&mut buf);
//! 
//! // corrupt
//! buf[0..16].fill(b'x');
//! 
//! // correct
//! rs255w223::correct_errors(&mut buf)?;
//! assert_eq!(&buf[0..12], b"Hello World!");
//! # Ok::<(), rs255w223::Error>(())
//! ```
//!
//! See the [module-level documentation](../../rs) for more info.


use __crate::traits::TryFrom;
use core::slice;
use core::fmt;

extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;
use alloc::borrow::Cow;


// Constants for Reed-Solomon error correction
//
// Reed-Solomon can correct ECC_SIZE known erasures and ECC_SIZE/2 unknown
// erasures. DATA_SIZE is arbitrary, however the total size is limited to
// 255 bytes in a GF(256) field.
//

/// Maximum size of the original data in bytes.
pub const DATA_SIZE:  usize = __data_size;

/// Size of the appended error-correction in bytes.
pub const ECC_SIZE:   usize = __ecc_size;

/// Size of the codeword, [`DATA_SIZE`] + [`ECC_SIZE`], in bytes.
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

/// The generator polynomial for this error-correction code.
pub const GENERATOR_POLY: [__gf; ECC_SIZE+1] = {
    let mut g = [__gf::new(0); ECC_SIZE+1];
    g[ECC_SIZE] = __gf::new(1);

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
            __gf::new(1),
            __gf::GENERATOR.naive_pow(i as __u),
        ];

        // G(x)*(x - g^i)
        let mut product = [__gf::new(0); ECC_SIZE+1];
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    /// Reed-Solomon can fail to decode if:
    /// - errors > ECC_SIZE/2
    /// - erasures > ECC_SIZE
    /// - 2*errors + erasures > ECC_SIZE
    ///
    TooManyErrors,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TooManyErrors => write!(f, "Too many errors to correct"),
        }
    }
}


/// Evaluate a polynomial at x using Horner's method
///
/// Note polynomials here are ordered biggest-coefficient first
///
fn poly_eval(f: &[__gf], x: __gf) -> __gf {
    let mut y = __gf::new(0);
    for c in f {
        y = y*x + c;
    }
    y
}

/// Multiply a polynomial by a scalar
fn poly_scale(f: &mut [__gf], c: __gf) {
    for i in 0..f.len() {
        f[i] *= c;
    }
}

/// Add two polynomials together
fn poly_add(f: &mut [__gf], g: &[__gf]) {
    debug_assert!(f.len() >= g.len());

    // note g.len() may be <= f.len()!
    for i in 0..f.len() {
        f[f.len()-1-i] += g[g.len()-1-i];
    }
}

/// Multiply two polynomials together
fn poly_mul(f: &mut [__gf], g: &[__gf]) {
    debug_assert!(f[..g.len()-1].iter().all(|x| *x == __gf::new(0)));

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
        f[f.len()-1-i] = __gf::new(0);

        for j in 0..g.len() {
            f[f.len()-1-(i+j)] += fi * g[g.len()-1-j];
        }
    }
}

/// Divide polynomials via synthetic division
///
/// Note both the quotient and remainder are left in the dividend
///
fn poly_divrem(f: &mut [__gf], g: &[__gf]) {
    debug_assert!(f.len() >= g.len());

    // find leading coeff to normalize g, note you could avoid
    // this if g is already normalized
    let leading_coeff = g[0];

    for i in 0 .. (f.len() - g.len() + 1) {
        if f[i] != __gf::new(0) {
            f[i] /= leading_coeff;

            for j in 1..g.len() {
                f[i+j] -= f[i] * g[j];
            }
        }
    }
}

// Encode using Reed-Solomon error correction
//
// Much like in CRC, we want to make the message a multiple of G(x),
// our generator polynomial. We can do this by appending the remainder
// of our message after division by G(x).
//
// ``` text
// c(x) = m(x) - (m(x) % G(x))
// ```
//
// Note we expect the message to only take up the first message.len()-ECC_SIZE
// bytes, but this can be smaller than BLOCK_SIZE
//

/// Encode a message using Reed-Solomon error-correction.
///
/// This writes [`ECC_SIZE`] bytes of error-correction information to the end
/// of the provided slice, based on the data provided in the first
/// `message.len()-ECC_SIZE` bytes. The entire codeword is limited to at most
/// [`BLOCK_SIZE`] bytes, but can be smaller.
///
/// ``` rust
/// # use gf256::rs::rs255w223;
/// let mut codeword = b"Hello World!".to_vec();
/// codeword.resize(codeword.len()+32, 0u8);
/// rs255w223::encode(&mut codeword);
/// assert_eq!(&codeword, b"Hello World!\
///     \x85\xa6\xad\xf8\xbd\x15\x94\x6e\x5f\xb6\x07\x12\x4b\xbd\x11\xd3\
///     \x34\x14\xa7\x06\xd6\x25\xfd\x84\xc2\x61\x81\xa7\x8a\x15\xc9\x35");
/// ```
///
pub fn encode(message: &mut [__u]) {
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
    poly_divrem(
        unsafe { __gf::slice_from_slice_mut_unchecked(&mut divrem) },
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
fn find_syndromes(f: &[__gf]) -> Vec<__gf> {
    let mut S = vec![];
    for i in 0..ECC_SIZE {
        S.push(
            poly_eval(f, __gf::GENERATOR.pow(__u::try_from(i).unwrap()))
        );
    }
    S
}

/// Find Forney syndromes, these hide known erasures from the original syndromes
/// so error detection doesn't try (and possibly fail) to find known erasures
///
fn find_forney_syndromes(
    codeword: &[__gf],
    S: &[__gf],
    erasures: &[usize]
) -> Vec<__gf> {
    let mut S = S.to_vec();
    for j in erasures {
        let Xj = __gf::GENERATOR.pow(__u::try_from(codeword.len()-1-j).unwrap());
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
fn find_erasure_locator(codeword: &[__gf], erasures: &[usize]) -> Vec<__gf> {
    let mut Λ = vec![__gf::new(0); erasures.len()+1];
    let Λ_len = Λ.len();
    Λ[Λ_len-1] = __gf::new(1);

    for j in erasures {
        poly_mul(&mut Λ, &[
            -__gf::GENERATOR.pow(__u::try_from(codeword.len()-1-j).unwrap()),
            __gf::new(1)
        ]);
    }

    Λ
}

/// Iteratively find the error locator polynomial using the
/// Berlekamp-Massey algorithm when we don't know the location of errors
///
fn find_error_locator(S: &[__gf]) -> Vec<__gf> {
    // the current estimate for the error locator polynomial
    let mut Λ = vec![__gf::new(0); S.len()+1];
    let Λ_len = Λ.len();
    Λ[Λ_len-1] = __gf::new(1);

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

        if delta != __gf::new(0) {
            if 2*v <= i {
                core::mem::swap(&mut Λ, &mut prev_Λ);
                poly_scale(&mut Λ, delta);
                poly_scale(&mut prev_Λ, delta.recip());
                v = i+1-v;
            }

            delta_Λ.copy_from_slice(&prev_Λ);
            poly_scale(&mut delta_Λ, delta);
            poly_add(&mut Λ, &delta_Λ);
        }
    }

    // trim leading zeros
    let zeros = Λ.iter().take_while(|x| **x == __gf::new(0)).count();
    Λ.drain(0..zeros);

    Λ
}

/// Find roots of the error locator polynomial by brute force
///
/// This just means we evaluate Λ(x) for all x locations in our
/// message, if they equal 0, aka are a root, then we found the
/// error location in our message.
///
fn find_error_locations(codeword: &[__gf], Λ: &[__gf]) -> Vec<usize> {
    let mut error_locations = vec![];
    for j in 0..codeword.len() {
        let Xj = __gf::GENERATOR.pow(__u::try_from(codeword.len()-1-j).unwrap());
        let zero = poly_eval(&Λ, Xj.recip());
        if zero == __gf::new(0) {
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
fn find_error_magnitudes(
    codeword: &[__gf],
    S: &[__gf],
    Λ: &[__gf],
    error_locations: &[usize]
) -> Vec<__gf> {
    // find the erasure evaluator polynomial
    //
    // Ω(x) = S(x)*Λ(x) mod x^2v
    //
    let mut Ω = vec![__gf::new(0); S.len()+Λ.len()-1];
    let Ω_len = Ω.len();
    Ω[Ω_len-S.len()..].copy_from_slice(&S);
    Ω[Ω_len-S.len()..].reverse();
    poly_mul(&mut Ω, &Λ);
    Ω.drain(..Ω.len()-S.len());

    // find the formal derivative of Λ
    //
    // Λ'(x) = Σ i*Λi*x^(i-1)
    //        i=1
    //
    let mut Λ_prime = vec![__gf::new(0); Λ.len()-1];
    for i in 1..Λ.len() {
        let mut sum = __gf::new(0);
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
        let Xj = __gf::GENERATOR.pow(__u::try_from(codeword.len()-1-j).unwrap());
        let Yj = (-Xj*poly_eval(&Ω, Xj.recip()))
            .checked_div(poly_eval(&Λ_prime, Xj.recip()))
            .unwrap_or(__gf::new(0));
        error_magnitudes.push(Yj);
    }

    error_magnitudes
}

/// Determine if codeword is correct and has no errors/erasures.
///
/// This is quite a bit faster than actually finding the errors/erasures.
///
/// ``` rust
/// # use gf256::rs::rs255w223;
/// let codeword = b"Hello World!\
///     \x85\xa6\xad\xf8\xbd\x15\x94\x6e\x5f\xb6\x07\x12\x4b\xbd\x11\xd3\
///     \x34\x14\xa7\x06\xd6\x25\xfd\x84\xc2\x61\x81\xa7\x8a\x15\xc9\x35".to_vec();
/// assert!(rs255w223::is_correct(&codeword));
/// ```
///
pub fn is_correct(codeword: &[__u]) -> bool {
    let codeword = unsafe { __gf::slice_from_slice_unchecked(codeword) };

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = find_syndromes(codeword);
    syndromes.iter().all(|s| *s == __gf::new(0))
}

/// Correct up to [`ECC_SIZE`] erasures at known locations.
///
/// Returns the number of erasures, or [`Error::TooManyErrors`] if the codeword
/// can not be corrected.
///
/// ``` rust
/// # use gf256::rs::rs255w223;
/// let mut codeword = b"xxxxxxxxxxxx\
///     xxxxxxxxxxxxxxxx\
///     xxxx\xd6\x25\xfd\x84\xc2\x61\x81\xa7\x8a\x15\xc9\x35".to_vec();
///
/// let erasures = (0..32).collect::<Vec<_>>();
/// assert_eq!(rs255w223::correct_erasures(&mut codeword, &erasures), Ok(32));
/// assert_eq!(&codeword, b"Hello World!\
///     \x85\xa6\xad\xf8\xbd\x15\x94\x6e\x5f\xb6\x07\x12\x4b\xbd\x11\xd3\
///     \x34\x14\xa7\x06\xd6\x25\xfd\x84\xc2\x61\x81\xa7\x8a\x15\xc9\x35");
/// ```
///
pub fn correct_erasures(
    codeword: &mut [__u],
    erasures: &[usize]
) -> Result<usize, Error> {
    let codeword = unsafe { __gf::slice_from_slice_mut_unchecked(codeword) };

    // too many erasures?
    if erasures.len() > ECC_SIZE {
        return Err(Error::TooManyErrors);
    }

    // find syndromes, syndromes of all zero means there are no errors
    let S = find_syndromes(codeword);
    if S.iter().all(|s| *s == __gf::new(0)) {
        return Ok(0);
    }

    // find erasure locator polynomial
    let Λ = find_erasure_locator(codeword, &erasures);

    // find erasure magnitudes using Forney's algorithm
    let erasure_magnitudes = find_error_magnitudes(
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
    let S = find_syndromes(codeword);
    if !S.iter().all(|s| *s == __gf::new(0)) {
        return Err(Error::TooManyErrors);
    }

    Ok(erasures.len())
}

/// Correct up to [`ECC_SIZE/2`](ECC_SIZE) errors at unknown locations.
///
/// Returns the number of errors, or [`Error::TooManyErrors`] if the codeword
/// can not be corrected.
///
/// ``` rust
/// # use gf256::rs::rs255w223;
/// let mut codeword = b"xexlx xoxlx!\
///     x\xa6x\xf8x\x15x\x6ex\xb6x\x12x\xbdx\xd3\
///     x\x14x\x06\xd6\x25\xfd\x84\xc2\x61\x81\xa7\x8a\x15\xc9\x35".to_vec();
///
/// assert_eq!(rs255w223::correct_errors(&mut codeword), Ok(16));
/// assert_eq!(&codeword, b"Hello World!\
///     \x85\xa6\xad\xf8\xbd\x15\x94\x6e\x5f\xb6\x07\x12\x4b\xbd\x11\xd3\
///     \x34\x14\xa7\x06\xd6\x25\xfd\x84\xc2\x61\x81\xa7\x8a\x15\xc9\x35");
/// ```
///
pub fn correct_errors(codeword: &mut [__u]) -> Result<usize, Error> {
    let codeword = unsafe { __gf::slice_from_slice_mut_unchecked(codeword) };

    // find syndromes, syndromes of all zero means there are no errors
    let S = find_syndromes(codeword);
    if S.iter().all(|s| *s == __gf::new(0)) {
        return Ok(0);
    }

    // find error locator polynomial
    let Λ = find_error_locator(&S);

    // too many errors?
    let error_count = Λ.len() - 1;
    if error_count*2 > ECC_SIZE {
        return Err(Error::TooManyErrors);
    }

    // find error locations
    let error_locations = find_error_locations(codeword, &Λ);

    // find erasure magnitude using Forney's algorithm
    let error_magnitudes = find_error_magnitudes(
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
    let S = find_syndromes(codeword);
    if !S.iter().all(|s| *s == __gf::new(0)) {
        return Err(Error::TooManyErrors);
    }

    Ok(error_locations.len())
}

/// Correct a mixture of errors and erasures, up to `2*errors+erasures <= ECC_SIZE`.
///
/// Where erasures are at known locations and errors are at unknown locations.
/// Errors must be <= [`ECC_SIZE`], erasures must be <= [`ECC_SIZE/2`](ECC_SIZE),
/// and `2*errors+erasures` must be <= [`ECC_SIZE`].
///
/// Returns the number of errors and erasures, or [`Error::TooManyErrors`] if the
/// codeword can not be corrected.
///
/// ``` rust
/// # use gf256::rs::rs255w223;
/// let mut codeword = b"xxxxxxxxxxxx\
///     xxxx\xbd\x15\x94\x6e\x5f\xb6\x07\x12\x4b\xbd\x11\xd3\
///     \x34x\xa7x\xd6x\xfdx\xc2x\x81x\x8ax\xc9x".to_vec();
///
/// let erasures = (0..16).collect::<Vec<_>>();
/// assert_eq!(rs255w223::correct(&mut codeword, &erasures), Ok(24));
/// assert_eq!(&codeword, b"Hello World!\
///     \x85\xa6\xad\xf8\xbd\x15\x94\x6e\x5f\xb6\x07\x12\x4b\xbd\x11\xd3\
///     \x34\x14\xa7\x06\xd6\x25\xfd\x84\xc2\x61\x81\xa7\x8a\x15\xc9\x35");
/// ```
///
pub fn correct(
    codeword: &mut [__u],
    erasures: &[usize]
) -> Result<usize, Error> {
    let codeword = unsafe { __gf::slice_from_slice_mut_unchecked(codeword) };

    // too many erasures?
    if erasures.len() > ECC_SIZE {
        return Err(Error::TooManyErrors);
    }

    // find syndromes, syndromes of all zero means there are no errors
    let S = find_syndromes(codeword);
    if S.iter().all(|s| *s == __gf::new(0)) {
        return Ok(0);
    }

    // find Forney syndromes, hiding known erasures from the syndromes
    let forney_S = find_forney_syndromes(codeword, &S, &erasures);

    // find error locator polynomial
    let Λ = find_error_locator(&forney_S);

    // too many errors/erasures?
    let error_count = Λ.len() - 1;
    let erasure_count = erasures.len();
    if error_count*2 + erasure_count > ECC_SIZE {
        return Err(Error::TooManyErrors);
    }

    // find all error locations
    let mut error_locations = find_error_locations(codeword, &Λ);
    error_locations.extend_from_slice(&erasures);

    // re-find error locator polynomial, this time including both 
    // errors and erasures
    let Λ = find_erasure_locator(codeword, &error_locations);

    // find erasure magnitude using Forney's algorithm
    let error_magnitudes = find_error_magnitudes(
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
    let S = find_syndromes(codeword);
    if !S.iter().all(|s| *s == __gf::new(0)) {
        return Err(Error::TooManyErrors);
    }

    Ok(error_locations.len())
}

