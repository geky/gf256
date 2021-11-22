//! Template for Reed-Solomon error-correction functions
//!
//! See examples/rs.rs for a more detailed explanation of
//! where these implementations come from

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
// erasures. DATA_SIZE is arbitrary depending on what ratio of error
// correction is required, however the total size is limited to 255 bytes
// in a gf(256) field.
//
pub const DATA_SIZE:  usize = __data_size;
pub const ECC_SIZE:   usize = __ecc_size;
pub const BLOCK_SIZE: usize = DATA_SIZE + ECC_SIZE;

// The generator polynomial in Reed-Solomon is a polynomial with roots
// at powers of a generator element in the finite-field.
//
// G(x) = (x - g^0)(x - g^1)(x - g^2)...
//
//        ECC_SIZE
// G(x) =    π (x - g^i) 
//          i=0
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
pub const GENERATOR_POLY: [__gf; ECC_SIZE+1] = {
    let mut g = [__gf::new(0); ECC_SIZE+1];
    g[ECC_SIZE] = __gf::new(1);

    // find G(x) = π (x - g^i)
    let mut i = 0usize;
    while i < ECC_SIZE {
        // H(x) = x - g^i
        let h = [
            __gf::new(1),
            __gf::GENERATOR.naive_pow(i as __u),
        ];

        // find G(x) = G(x)*H(x) = G(x)*(x - g^i)
        let mut r = [__gf::new(0); ECC_SIZE+1];
        let mut j = 0usize;
        while j < i+1 {
            let mut k = 0usize;
            while k < h.len() {
                r[r.len()-1-(j+k)] = r[r.len()-1-(j+k)].naive_add(
                    g[g.len()-1-j].naive_mul(h[h.len()-1-k])
                );
                k += 1;
            }
            j += 1;
        }
        g = r;

        i += 1;
    }

    g
};


/// Error codes for Reed-Solomon
#[derive(Debug, Clone)]
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

/// Encode using Reed-Solomon error correction
///
/// Much like in CRC, we want to make the message a multiple of G(x),
/// our generator polynomial. We can do this by appending the remainder
/// of our message after division by G(x).
///
/// M'(x) = M(x) + (M(x) % G(x))
///
/// Note we expect the message to only take up the first message.len()-ECC_SIZE
/// bytes, but this can be smaller than BLOCK_SIZE
///
pub fn encode(message: &mut [__u]) {
    assert!(message.len() <= BLOCK_SIZE);
    assert!(message.len() >= ECC_SIZE);
    let data_len = message.len() - ECC_SIZE;

    // create copy for polynomial division
    //
    // note if message is < DATA_SIZE we treat it as a smaller polynomial,
    // this is equivalent to prepending zeros
    //
    let mut rem = message.to_vec();
    rem[data_len..].fill(0);

    // divide by our generator polynomial
    poly_divrem(
        unsafe { __gf::slice_from_slice_mut_unchecked(&mut rem) },
        &GENERATOR_POLY
    );

    // return message + remainder, this new message is a polynomial
    // perfectly divisable by our generator polynomial
    message[data_len..].copy_from_slice(&rem[data_len..]);
}

/// Find syndromes, which should be zero if there are no errors
///
/// S(x) = M(g^x)
///
fn find_syndromes(f: &[__gf]) -> Vec<__gf> {
    let mut syndromes = vec![];
    for i in (0..ECC_SIZE).rev() {
        syndromes.push(
            poly_eval(f, __gf::GENERATOR.pow(__u::try_from(i).unwrap()))
        );
    }
    syndromes
}

/// Find Forney syndromes, these hide known erasures from the original syndromes
/// so error detection doesn't try (and possibly fail) to find known erasures
///
fn find_forney_syndromes(
    syndromes: &[__gf],
    erasures: &[usize]
) -> Vec<__gf> {
    let mut fs = Vec::from(syndromes);

    for i in erasures {
        let x = __gf::GENERATOR.pow(__u::try_from(BLOCK_SIZE-1-i).unwrap());
        for j in 0..fs.len()-1 {
            let fs_len = fs.len();
            fs[fs_len-1-j] = fs[fs_len-1-j]*x + fs[fs_len-1-(j+1)];
        }
    }

    // trim unnecessary syndromes
    fs.drain(0..erasures.len());
    fs
}

/// Find the erasure locator polynomial
///
/// Λ(x) = π (1 - xg^i)
///
fn find_erasure_locator(erasures: &[usize]) -> Vec<__gf> {
    let mut el = vec![__gf::new(0); erasures.len()+1];
    let el_len = el.len();
    el[el_len-1] = __gf::new(1);

    for i in erasures {
        poly_mul(&mut el, &[
            __gf::GENERATOR.pow(__u::try_from(BLOCK_SIZE-1-i).unwrap()),
            __gf::new(1)
        ]);
    }

    el
}

/// Find the erasure evaluator polynomial
///
/// Ω(x) = S(x)*Λ(x)
///
fn find_erasure_evaluator(syndromes: &[__gf], el: &[__gf]) -> Vec<__gf> {
    let mut ee = vec![__gf::new(0); syndromes.len()+el.len()+2];
    let ee_len = ee.len();
    ee[ee_len-syndromes.len()-1..ee_len-1].copy_from_slice(syndromes);
    poly_mul(&mut ee, el);

    ee.drain(0 .. ee_len-el.len());
    ee
}

/// Iteratively find the error locator polynomial using the
/// Berlekamp-Massey algorithm
///
fn find_error_locator(syndromes: &[__gf]) -> Vec<__gf> {
    let mut old_el = vec![__gf::new(0); syndromes.len()+1];
    let old_el_len = old_el.len();
    old_el[old_el_len-1] = __gf::new(1);

    let mut new_el = vec![__gf::new(0); syndromes.len()+1];
    let new_el_len = new_el.len();
    new_el[new_el_len-1] = __gf::new(1);

    // l is the current estimate on number of errors
    let mut l = 0;

    for i in 0..syndromes.len() {
        let mut delta = syndromes[syndromes.len()-1-i];
        for j in 1..l+1 {
            delta += new_el[new_el.len()-1-j]
                * syndromes[syndromes.len()-1-(i-j)];
        }

        // shift polynomial
        old_el.copy_within(1.., 0);
        let old_el_len = old_el.len();
        old_el[old_el_len-1] = __gf::new(0);

        if delta != __gf::new(0) {
            poly_scale(&mut old_el, delta);
            if 2*l <= i {
                new_el.swap_with_slice(&mut old_el);
                l = i+1-l;
            }
            poly_add(&mut new_el, &old_el);
            poly_scale(&mut old_el, delta.recip());
        }
    }

    // trim leading zeros
    let zeros = new_el.iter().take_while(|x| **x == __gf::new(0)).count();
    new_el.drain(0 .. zeros);

    new_el.reverse();
    new_el
}

/// Find roots of the error locator polynomial by brute force
///
/// This just means we evaluate Λ(x) for all x locations in our
/// message, if they equal 0, aka are a root, then we found the
/// error location in our message.
///
fn find_errors(error_locator: &[__gf]) -> Vec<usize> {
    let mut errors = vec![];

    for i in 0..BLOCK_SIZE {
        let y = poly_eval(
            error_locator,
            __gf::GENERATOR.pow(__u::try_from(i).unwrap())
        );

        if y == __gf::new(0) {
            // found a root!
            errors.push(BLOCK_SIZE-1-i);
        }
    }

    debug_assert_eq!(errors.len(), error_locator.len()-1);
    errors
}

/// Find the error magnitude polynomial using Forney's algorithm
///
///          Ω(g^-i)
/// e_i = - ---------
///          Λ'(g^-i)
///
/// Where Λ'(x) is the formal derivative of of Λ(x), aka the "totally not
/// derivative" derivative of Λ(x), because lim x->∞ doesn't make sense in
/// a finite-field, because... it's finite.
///
///
/// if   Λ(x)  = Σ (a_i*x^i)
///              i
///
///                 i
/// then Λ'(x) = Σ (Σ (a_i*x^i-1))
///              i  j
///
fn find_erasure_magnitude(
    erasures: &[usize],
    erasure_evaluator: &[__gf]
) -> Vec<__gf>{
    // find erasure roots
    let mut erasure_roots = Vec::with_capacity(erasures.len());
    for i in erasures {
        erasure_roots.push(
            __gf::GENERATOR.pow(__u::try_from(BLOCK_SIZE-1-i).unwrap())
        );
    }

    // find erasure magnitudes using Forney's algorithm
    let mut erasure_magnitude = vec![__gf::new(0); BLOCK_SIZE];
    for i in 0..erasure_roots.len() {
        let root = erasure_roots[i];
        let root_inv = root.recip();

        let mut derivative = __gf::new(1);
        for j in 0..erasure_roots.len() {
            if j != i {
                derivative *= __gf::new(1) - root_inv*erasure_roots[j];
            }
        }

        // derivative should never be zero, though this can happen if there
        // are redundant erasures
        assert!(derivative != __gf::new(0));

        // evaluate error evaluator
        let y = root * poly_eval(&erasure_evaluator, root_inv);

        // find the actual magnitude
        erasure_magnitude[erasures[i]] = y / derivative;
    }

    erasure_magnitude
}

/// Determine if message is correct
///
/// Note this is quite a bit faster than correcting the errors
///
pub fn is_correct(message: &[__u]) -> bool {
    let message_poly = unsafe { __gf::slice_from_slice_unchecked(message) };

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = find_syndromes(message_poly);
    syndromes.iter().all(|s| *s == __gf::new(0))
}

/// Correct up to ECC_SIZE erasures at known locations
pub fn correct_erasures(
    message: &mut [__u],
    erasures: &[usize]
) -> Result<usize, Error> {
    let message_poly = unsafe { __gf::slice_from_slice_mut_unchecked(message) };

    // too many erasures?
    if erasures.len() > ECC_SIZE {
        return Err(Error::TooManyErrors);
    }

    // adjust erasures for implicitly prepended zeros?
    let mut erasures = Cow::Borrowed(erasures);
    if message_poly.len() < BLOCK_SIZE {
        for erasure in erasures.to_mut().iter_mut() {
            *erasure += BLOCK_SIZE-message_poly.len();
        }
    }

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = find_syndromes(message_poly);
    if syndromes.iter().all(|s| *s == __gf::new(0)) {
        return Ok(0);
    }

    // find erasure locator polynomial
    let erasure_locator = find_erasure_locator(&erasures);

    // find erasure evaluator polynomial
    let erasure_evaluator = find_erasure_evaluator(&syndromes, &erasure_locator);

    // find erasure magnitude using Forney's algorithm
    let erasure_magnitude = find_erasure_magnitude(
        &erasures,
        &erasure_evaluator
    );

    // correct the errors
    poly_add(
        message_poly,
        &erasure_magnitude[BLOCK_SIZE-message_poly.len()..]
    );
    Ok(erasures.len())
}

/// Correct up to ECC_SIZE/2 errors at unknown locations
pub fn correct_errors(message: &mut [__u]) -> Result<usize, Error> {
    let message_poly = unsafe { __gf::slice_from_slice_mut_unchecked(message) };

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = find_syndromes(message_poly);
    if syndromes.iter().all(|s| *s == __gf::new(0)) {
        return Ok(0);
    }

    // find error locator polynomial
    let error_locator = find_error_locator(&syndromes);

    // too many errors?
    let error_count = error_locator.len() - 1;
    if error_count*2 > ECC_SIZE {
        return Err(Error::TooManyErrors);
    }

    // find error locations
    let errors = find_errors(&error_locator);

    // find erasure locator polynomial
    let erasure_locator = find_erasure_locator(&errors);

    // find erasure evaluator polynomial
    let erasure_evaluator = find_erasure_evaluator(&syndromes, &erasure_locator);

    // find erasure magnitude using Forney's algorithm
    let erasure_magnitude = find_erasure_magnitude(
        &errors,
        &erasure_evaluator
    );

    // correct the errors
    poly_add(
        message_poly,
        &erasure_magnitude[BLOCK_SIZE-message_poly.len()..]
    );
    Ok(errors.len())
}

/// Correct a mixture of erasures at unknown locations and erasures
/// as known locations, can correct up to 2*errors+erasures <= ECC_SIZE
pub fn correct(
    message: &mut [__u],
    erasures: &[usize]
) -> Result<usize, Error> {
    let message_poly = unsafe { __gf::slice_from_slice_mut_unchecked(message) };

    // adjust erasures for implicitly prepended zeros?
    let mut erasures = Cow::Borrowed(erasures);
    if message_poly.len() < BLOCK_SIZE {
        for erasure in erasures.to_mut().iter_mut() {
            *erasure += BLOCK_SIZE-message_poly.len();
        }
    }

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = find_syndromes(message_poly);
    if syndromes.iter().all(|s| *s == __gf::new(0)) {
        return Ok(0);
    }

    // find Forney syndromes, hiding known erasures from the syndromes
    let fsyndromes = find_forney_syndromes(&syndromes, &erasures);

    // find error locator polynomial
    let error_locator = find_error_locator(&fsyndromes);

    // too many errors/erasures?
    let error_count = error_locator.len() - 1;
    let erasure_count = erasures.len();
    if error_count*2 +erasure_count > ECC_SIZE {
        return Err(Error::TooManyErrors);
    }

    // find all error locations
    let mut errors = find_errors(&error_locator);
    errors.extend_from_slice(&erasures);

    // find erasure locator polynomial
    let erasure_locator = find_erasure_locator(&errors);

    // find erasure evaluator polynomial
    let erasure_evaluator = find_erasure_evaluator(&syndromes, &erasure_locator);

    // find erasure magnitude using Forney's algorithm
    let erasure_magnitude = find_erasure_magnitude(
        &errors,
        &erasure_evaluator
    );

    // correct the errors
    poly_add(
        message_poly,
        &erasure_magnitude[BLOCK_SIZE-message_poly.len()..]
    );
    Ok(errors.len())
}


