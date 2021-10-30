//! Reed-Solomon error-correction codes using the Galois-field types
//!
//! The main idea behind error-correct is to create a set of codewords,
//! usually a message + extra redundant bits, where each codeword looks
//! quite different from each other.
//!
//! In the case of Reed-Solomon, codewords are choosen such that, when
//! viewed as a polynomial, every codeword is a multiple of some
//! polynomial G(x).
//!
//! Galois-fields come into play here as a method to represent our message,
//! a sequence of bytes, as a polynomial in gf(256).
//!
//! Note! This example may be a bit confusing for a couple reasons:
//!
//! 1. The math is complicated, with more names than equations, which to
//!    be honest my understanding is rough.
//!
//! 2. We're dealing with two nested systems of polynomials. Reed-Solomon
//!    is built on algebra of polynomials where the _coefficients_ are in
//!    gf(256), which is usually also viewed as an algebra of polynomials.
//!
//!    Try to not worry about the representation of gf(256) here, treat it
//!    a just a set of symbols closed over a conveniently byte-sized
//!    finite-field.
//!
//! Based on description/implementation from:
//! https://en.wikiversity.org/wiki/Reed%E2%80%93Solomon_codes_for_coders
//! https://en.wikipedia.org/wiki/Forney_algorithm
//! https://en.wikipedia.org/wiki/Reed–Solomon_error_correction
//!

use std::convert::TryFrom;
use std::borrow::Cow;
use rand;
use rand::Rng;
use ::gf256::*;


// Constants for Reed-Solomon error correction
//
// Reed-Solomon can correct ECC_SIZE known erasures and ECC_SIZE/2 unknown
// erasures. DATA_SIZE is arbitrary depending on what ratio of error
// correction is required, however the total size is limited to 255 bytes
// in a gf(256) field.
//
pub const DATA_SIZE:  usize = 12;
pub const ECC_SIZE:   usize = 8;
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
pub const GENERATOR_POLY: [gf256; ECC_SIZE+1] = {
    let mut g = [gf256(0); ECC_SIZE+1];
    g[ECC_SIZE] = gf256(1);

    // find G(x) = π (x - g^i)
    let mut i = 0usize;
    while i < ECC_SIZE {
        // H(x) = x - g^i
        let h = [
            gf256(1),
            gf256::GENERATOR.naive_pow(i as u8),
        ];

        // find G(x) = G(x)*H(x) = G(x)*(x - g^i)
        let mut r = [gf256(0); ECC_SIZE+1];
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
pub enum RsError {
    /// Reed-Solomon can fail to decode if:
    /// - errors > ECC_SIZE/2
    /// - erasures > ECC_SIZE
    /// - 2*errors + erasures > ECC_SIZE
    ///
    TooManyErrors,
}


/// Convert slice of u8s to gf256s
fn rs_poly_from_slice(slice: &[u8]) -> &[gf256] {
    // I couldn't find a safe way to do this cheaply+safely 
    unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const gf256,
            slice.len()
        )
    }
}

/// Convert mut slice of u8s to gf256s
fn rs_poly_from_slice_mut(slice: &mut [u8]) -> &mut [gf256] {
    // I couldn't find a safe way to do this cheaply+safely 
    unsafe {
        std::slice::from_raw_parts_mut(
            slice.as_mut_ptr() as *mut gf256,
            slice.len()
        )
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
/// M'(x) = M(x) + (M(x) % G(x))
///
pub fn rs_encode(message: &[u8]) -> Vec<u8> {
    assert!(message.len() <= DATA_SIZE);

    // append zeros for polynomial division 
    //
    // note if message is < DATA_SIZE we treat it as a smaller polynomial,
    // this is equivalent to prepending zeros
    //
    let mut encoded = vec![0u8; message.len()+ECC_SIZE];
    encoded[0..message.len()].copy_from_slice(&message);

    // divide by our generator polynomial
    rs_poly_divrem(
        rs_poly_from_slice_mut(&mut encoded),
        &GENERATOR_POLY
    );

    // copy message back over
    encoded[0..message.len()].copy_from_slice(&message);

    // return message + remainder, this new message is a polynomial
    // perfectly divisable by our generator polynomial
    encoded
}

/// Find syndromes, which should be zero if there are no errors
///
/// S(x) = M(g^x)
///
fn rs_find_syndromes(f: &[gf256]) -> Vec<gf256> {
    let mut syndromes = vec![];
    for i in (0..ECC_SIZE).rev() {
        syndromes.push(
            rs_poly_eval(f, gf256::GENERATOR.pow(u8::try_from(i).unwrap()))
        );
    }
    syndromes
}

/// Find Forney syndromes, these hide known erasures from the original syndromes
/// so error detection doesn't try (and possibly fail) to find known erasures
///
fn rs_find_forney_syndromes(
    syndromes: &[gf256],
    erasures: &[usize]
) -> Vec<gf256> {
    let mut fs = Vec::from(syndromes);

    for i in erasures {
        let x = gf256::GENERATOR.pow(u8::try_from(BLOCK_SIZE-1-i).unwrap());
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
fn rs_find_erasure_locator(erasures: &[usize]) -> Vec<gf256> {
    let mut el = vec![gf256(0); erasures.len()+1];
    let el_len = el.len();
    el[el_len-1] = gf256(1);

    for i in erasures {
        rs_poly_mul(&mut el, &[
            gf256::GENERATOR.pow(u8::try_from(BLOCK_SIZE-1-i).unwrap()),
            gf256(1)
        ]);
    }

    el
}

/// Find the erasure evaluator polynomial
///
/// Ω(x) = S(x)*Λ(x)
///
fn rs_find_erasure_evaluator(syndromes: &[gf256], el: &[gf256]) -> Vec<gf256> {
    let mut ee = vec![gf256(0); syndromes.len()+el.len()+2];
    let ee_len = ee.len();
    ee[ee_len-syndromes.len()-1..ee_len-1].copy_from_slice(syndromes);
    rs_poly_mul(&mut ee, el);

    ee.drain(0 .. ee_len-el.len());
    ee
}

/// Iteratively find the error locator polynomial using the
/// Berlekamp-Massey algorithm
///
fn rs_find_error_locator(syndromes: &[gf256]) -> Vec<gf256> {
    let mut old_el = vec![gf256(0); syndromes.len()+1];
    let old_el_len = old_el.len();
    old_el[old_el_len-1] = gf256(1);

    let mut new_el = vec![gf256(0); syndromes.len()+1];
    let new_el_len = new_el.len();
    new_el[new_el_len-1] = gf256(1);

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
        old_el[old_el_len-1] = gf256(0);

        if delta != gf256(0) {
            rs_poly_scale(&mut old_el, delta);
            if 2*l <= i {
                new_el.swap_with_slice(&mut old_el);
                l = i+1-l;
            }
            rs_poly_add(&mut new_el, &old_el);
            rs_poly_scale(&mut old_el, delta.recip());
        }
    }

    // trim leading zeros
    let zeros = new_el.iter().take_while(|x| **x == gf256(0)).count();
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
fn rs_find_errors(error_locator: &[gf256]) -> Vec<usize> {
    let mut errors = vec![];

    for i in 0..BLOCK_SIZE {
        let y = rs_poly_eval(
            error_locator,
            gf256::GENERATOR.pow(u8::try_from(i).unwrap())
        );

        if y == gf256(0) {
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
fn rs_find_erasure_magnitude(
    erasures: &[usize],
    erasure_evaluator: &[gf256]
) -> Vec<gf256>{
    // find erasure roots
    let mut erasure_roots = Vec::with_capacity(erasures.len());
    for i in erasures {
        erasure_roots.push(
            gf256::GENERATOR.pow(u8::try_from(BLOCK_SIZE-1-i).unwrap())
        );
    }

    // find erasure magnitudes using Forney's algorithm
    let mut erasure_magnitude = vec![gf256(0); BLOCK_SIZE];
    for i in 0..erasure_roots.len() {
        let root = erasure_roots[i];
        let root_inv = root.recip();

        let mut derivative = gf256(1);
        for j in 0..erasure_roots.len() {
            if j != i {
                derivative *= gf256(1) - root_inv*erasure_roots[j];
            }
        }

        // derivative should never be zero, though this can happen if there
        // are redundant erasures
        assert!(derivative != gf256(0));

        // evaluate error evaluator
        let y = root * rs_poly_eval(&erasure_evaluator, root_inv);

        // find the actual magnitude
        erasure_magnitude[erasures[i]] = y / derivative;
    }

    erasure_magnitude
}

/// Determine if message is correct
///
/// Note this is quite a bit faster than correcting the errors
///
pub fn rs_is_correct(message: &[u8]) -> bool {
    let message_poly = rs_poly_from_slice(message);

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = rs_find_syndromes(message_poly);
    syndromes.iter().all(|s| *s == gf256(0))
}

/// Correct up to ECC_SIZE erasures at known locations
pub fn rs_correct_erasures(
    message: &mut [u8],
    erasures: &[usize]
) -> Result<usize, RsError> {
    let message_poly = rs_poly_from_slice_mut(message);

    // too many erasures?
    if erasures.len() > ECC_SIZE {
        return Err(RsError::TooManyErrors);
    }

    // adjust erasures for implicitly prepended zeros?
    let mut erasures = Cow::Borrowed(erasures);
    if message_poly.len() < BLOCK_SIZE {
        for erasure in erasures.to_mut().iter_mut() {
            *erasure += BLOCK_SIZE-message_poly.len();
        }
    }

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = rs_find_syndromes(message_poly);
    if syndromes.iter().all(|s| *s == gf256(0)) {
        return Ok(0);
    }

    // find erasure locator polynomial
    let erasure_locator = rs_find_erasure_locator(&erasures);

    // find erasure evaluator polynomial
    let erasure_evaluator = rs_find_erasure_evaluator(&syndromes, &erasure_locator);

    // find erasure magnitude using Forney's algorithm
    let erasure_magnitude = rs_find_erasure_magnitude(
        &erasures,
        &erasure_evaluator
    );

    // correct the errors
    rs_poly_add(
        message_poly,
        &erasure_magnitude[BLOCK_SIZE-message_poly.len()..]
    );
    Ok(erasures.len())
}

/// Correct up to ECC_SIZE/2 errors at unknown locations
pub fn rs_correct_errors(message: &mut [u8]) -> Result<usize, RsError> {
    let message_poly = rs_poly_from_slice_mut(message);

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = rs_find_syndromes(message_poly);
    if syndromes.iter().all(|s| *s == gf256(0)) {
        return Ok(0);
    }

    // find error locator polynomial
    let error_locator = rs_find_error_locator(&syndromes);

    // too many errors?
    let error_count = error_locator.len() - 1;
    if error_count*2 > ECC_SIZE {
        return Err(RsError::TooManyErrors);
    }

    // find error locations
    let errors = rs_find_errors(&error_locator);

    // find erasure locator polynomial
    let erasure_locator = rs_find_erasure_locator(&errors);

    // find erasure evaluator polynomial
    let erasure_evaluator = rs_find_erasure_evaluator(&syndromes, &erasure_locator);

    // find erasure magnitude using Forney's algorithm
    let erasure_magnitude = rs_find_erasure_magnitude(
        &errors,
        &erasure_evaluator
    );

    // correct the errors
    rs_poly_add(
        message_poly,
        &erasure_magnitude[BLOCK_SIZE-message_poly.len()..]
    );
    Ok(errors.len())
}

/// Correct a mixture of erasures at unknown locations and erasures
/// as known locations, can correct up to 2*errors+erasures <= ECC_SIZE
pub fn rs_correct(
    message: &mut [u8],
    erasures: &[usize]
) -> Result<usize, RsError> {
    let message_poly = rs_poly_from_slice_mut(message);

    // adjust erasures for implicitly prepended zeros?
    let mut erasures = Cow::Borrowed(erasures);
    if message_poly.len() < BLOCK_SIZE {
        for erasure in erasures.to_mut().iter_mut() {
            *erasure += BLOCK_SIZE-message_poly.len();
        }
    }

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = rs_find_syndromes(message_poly);
    if syndromes.iter().all(|s| *s == gf256(0)) {
        return Ok(0);
    }

    // find Forney syndromes, hiding known erasures from the syndromes
    let fsyndromes = rs_find_forney_syndromes(&syndromes, &erasures);

    // find error locator polynomial
    let error_locator = rs_find_error_locator(&fsyndromes);

    // too many errors/erasures?
    let error_count = error_locator.len() - 1;
    let erasure_count = erasures.len();
    if error_count*2 +erasure_count > ECC_SIZE {
        return Err(RsError::TooManyErrors);
    }

    // find all error locations
    let mut errors = rs_find_errors(&error_locator);
    errors.extend_from_slice(&erasures);

    // find erasure locator polynomial
    let erasure_locator = rs_find_erasure_locator(&errors);

    // find erasure evaluator polynomial
    let erasure_evaluator = rs_find_erasure_evaluator(&syndromes, &erasure_locator);

    // find erasure magnitude using Forney's algorithm
    let erasure_magnitude = rs_find_erasure_magnitude(
        &errors,
        &erasure_evaluator
    );

    // correct the errors
    rs_poly_add(
        message_poly,
        &erasure_magnitude[BLOCK_SIZE-message_poly.len()..]
    );
    Ok(errors.len())
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

    let mut message = rs_encode(orig_message);
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
}
