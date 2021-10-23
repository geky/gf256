//! Reed-Solomon error correction encoding/decoding using our
//! Galois-field types
//!
//! TODO doc
//!

use std::convert::TryFrom;
use std::mem;
use std::cmp;
use rand;
use rand::Rng;
use ::gf256::*;


const DATA_SIZE:  usize = 12;
const ECC_SIZE:   usize = 8;
const BLOCK_SIZE: usize = DATA_SIZE + ECC_SIZE;

const GENERATOR_POLY: [gf256; ECC_SIZE+1] = {
    let mut g = [gf256(0); ECC_SIZE+1];
    g[ECC_SIZE] = gf256(1);

    // find G(x) = prod{x - g^i}
    let mut i = 0usize;
    while i < ECC_SIZE {
        // H(x) = x - g^i
        let h = [
            gf256(1),
            gf256::GENERATOR.naive_pow(i as u32),
        ];

        // G(x) = G(x)*H(x) = G(x)*(x - g^i)
        let mut r = [gf256(0); ECC_SIZE+1];
        let mut j = 0usize;
        while j < i+1 {
            let mut k = 0usize;
            while k < h.len() {
                r[r.len()-1-(j+k)] = r[r.len()-1-(j+k)].add(
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


/// Convert slice of u8s to gf256s
///
/// I couldn't find a safe way to do this cheaply
///
#[allow(unused)]
fn rs_poly_from_slice(slice: &[u8]) -> &[gf256] {
    unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const gf256,
            slice.len()
        )
    }
}

/// Convert mut slice of u8s to gf256s
///
/// I couldn't find a safe way to do this cheaply
///
fn rs_poly_from_slice_mut(slice: &mut [u8]) -> &mut [gf256] {
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

    // note g.len() my not equal f.len()
    for i in 0..f.len() {
        f[f.len()-1-i] += g[g.len()-1-i];
    }
}

/// Multiply two polynomials together
fn rs_poly_mul(f: &mut [gf256], g: &[gf256]) {
    debug_assert!(f[..g.len()-1].iter().all(|x| *x == gf256(0)));

    // TODO make this in-place
    let mut r = vec![gf256(0); f.len()];
    for i in 0..f.len()-g.len()+1 {
        for j in 0..g.len() {
            let r_len = r.len();
            r[r_len-1-(i+j)] += f[f.len()-1-i] * g[g.len()-1-j];
        }
    }

    f.copy_from_slice(&r);

// NOTE THIS DOES NOT WORK
//    // we can multiply in place by iterating over f backwards
//    for i in (0..f.len()).rev() {
//        let fi = f[f.len()-1-i];
//        for j in 0..g.len() {
//            f[f.len()-1-(i+j)] += fi * g[g.len()-1-j];
//        }
//    }
}

/// Divide polynomials via synthetic division
///
/// Note the quotient and remainder are left in the dividend
///
fn rs_poly_divrem(f: &mut [gf256], g: &[gf256]) {
    debug_assert!(f.len() >= g.len());

    // TODO do we need to normalize?
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


pub fn rs_encode(message: &[u8]) -> Vec<u8> {
    debug_assert!(message.len() <= DATA_SIZE);

    // pad with zeros to form a full block
    let mut encoded = vec![0u8; BLOCK_SIZE];
    encoded[0..message.len()].copy_from_slice(&message);

    // divide by our generator polynomial
    rs_poly_divrem(
        rs_poly_from_slice_mut(&mut encoded),
        &GENERATOR_POLY
    );

    // copy message back over
    encoded[0..message.len()].copy_from_slice(&message);

    // compress if the full block is unused
    if message.len() < DATA_SIZE {
        encoded.drain(message.len() .. DATA_SIZE);
    }

    // return message + remainder
    encoded
}

// find syndromes, should be zero if no errors
fn rs_find_syndromes(f: &[gf256]) -> Vec<gf256> {
    let mut syndromes = vec![];
    for i in (0..ECC_SIZE).rev() {
        syndromes.push(
            rs_poly_eval(f, gf256::GENERATOR.pow(u32::try_from(i).unwrap()))
        );
    }
    syndromes
}

// find Forney syndromes to hide known erasures from the original syndrome
fn rs_find_forney_syndromes(
    syndromes: &[gf256],
    erasure_coeffs: &[usize]
) -> Vec<gf256> {
    let mut fs = Vec::from(syndromes);

    for coeff in erasure_coeffs {
        let x = gf256::GENERATOR.pow(u32::try_from(*coeff).unwrap());
        for j in 0..fs.len()-1 {
            let fs_len = fs.len();
            fs[fs_len-1-j] = fs[fs_len-1-j]*x + fs[fs_len-1-(j+1)];
        }
    }

    // trim unnecessary syndromes
    fs.drain(0..erasure_coeffs.len());
    fs
}

fn rs_find_erasure_locator(erasure_coeffs: &[usize]) -> Vec<gf256> {
    // TODO ??? what's going on here
    let mut el = vec![gf256(0); erasure_coeffs.len()+1];
    let el_len = el.len();
    el[el_len-1] = gf256(1);

    for coeff in erasure_coeffs {
        rs_poly_mul(&mut el, &[
            gf256::GENERATOR.pow(u32::try_from(*coeff).unwrap()),
            gf256(1)
        ]);
    }

    el
}

fn rs_find_erasure_evaluator(syndromes: &[gf256], el: &[gf256]) -> Vec<gf256> {
    // TODO ??? what's going on here
    // note we omit the g^0 term in the syndromes until here
    let mut ee = vec![gf256(0); syndromes.len()+el.len()+2];
    let ee_len = ee.len();
    ee[ee_len-syndromes.len()-1..ee_len-1].copy_from_slice(syndromes);
    rs_poly_mul(&mut ee, el);

    ee.drain(0 .. ee_len-el.len());
    ee
}

fn rs_find_error_locator(syndromes: &[gf256]) -> Vec<gf256> {
    let mut old_el = vec![gf256(0); syndromes.len()+1];
    let old_el_len = old_el.len();
    old_el[old_el_len-1] = gf256(1);
    let mut old_el_degree = 1;

    let mut new_el = vec![gf256(0); syndromes.len()+1];
    let new_el_len = new_el.len();
    new_el[new_el_len-1] = gf256(1);
    let mut new_el_degree = 1;

    // iteratively find error locator using Berlekamp-Massey
    for i in 0..syndromes.len() {
        let mut delta = syndromes[syndromes.len()-1-i];
        for j in 1..new_el_degree {
            delta += new_el[new_el.len()-1-j]
                * syndromes[syndromes.len()-1-(i-j)];
        }

        // shift polynomial
        let old_el_len = old_el.len();
        old_el.copy_within(old_el_len-old_el_degree.., old_el_len-old_el_degree-1);
        old_el[old_el_len-1] = gf256(0);
        old_el_degree += 1;

        if delta != gf256(0) {
            // TODO doesn't this always happen?
            if old_el_degree > new_el_degree {
                new_el.swap_with_slice(&mut old_el);
                mem::swap(&mut new_el_degree, &mut old_el_degree);
                rs_poly_scale(&mut new_el, delta);
                rs_poly_scale(&mut old_el, delta.recip());
            }

            rs_poly_scale(&mut old_el, delta);
            rs_poly_add(&mut new_el, &old_el);
            rs_poly_scale(&mut old_el, delta.recip());
            new_el_degree = cmp::max(new_el_degree, old_el_degree);
        }
    }

    // trim leading zeros
    let zeros = new_el.iter().take_while(|x| **x == gf256(0)).count();
    new_el.drain(0 .. zeros);

    new_el.reverse();
    new_el
}

// find roots of error_locator by brute force
fn rs_find_errors(error_locator: &[gf256]) -> Vec<usize> {
    let mut errors = vec![];

    for i in 0..BLOCK_SIZE {
        let y = rs_poly_eval(
            error_locator,
            gf256::GENERATOR.pow(u32::try_from(i).unwrap())
        );

        if y == gf256(0) {
            // found a root!
            errors.push(BLOCK_SIZE-1-i);
        }
    }

    debug_assert_eq!(errors.len(), error_locator.len()-1);
    errors
}

// TODO is this really Chien's search?
// find error locations with Chien search
fn rs_find_erasure_roots(erasure_coeffs: &[usize]) -> Vec<gf256> {
    let mut erasure_roots = Vec::with_capacity(erasure_coeffs.len());
    for coeff in erasure_coeffs {
        erasure_roots.push(
            gf256::GENERATOR.pow(u32::try_from(*coeff).unwrap())
        );
    }
    erasure_roots
}

// find error magnitude using Forney's algorithm
fn rs_find_erasure_magnitude(
    erasure_evaluator: &[gf256],
    erasure_roots: &[gf256],
    erasures: &[usize]
) -> Vec<gf256>{
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

        assert!(derivative != gf256(0));

        // evaluate error evaluator
        let y = root * rs_poly_eval(&erasure_evaluator, root_inv);

        // find the actual magnitude
        erasure_magnitude[erasures[i]] = y / derivative;
    }
    erasure_magnitude
}

/// Correct up to ECC_SIZE erasures at known locations
// TODO rename
// TODO insert padding?
pub fn rs_correct_erasures(message: &mut [u8], erasures: &[usize]) {
    let message_poly = rs_poly_from_slice_mut(message);

    // convert locations to coefficients (reversed order)
    // TODO we can avoid allocating this by making everything in terms of indices
    let erasure_coeffs = erasures.iter()
        .map(|i| message_poly.len()-1-i)
        .collect::<Vec<_>>();

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = rs_find_syndromes(message_poly);
    if syndromes.iter().all(|s| *s == gf256(0)) {
        return;
    }

    // find error locator polynomial
    let erasure_locator = rs_find_erasure_locator(&erasure_coeffs);

    // find error evaluator polynomial
    let erasure_evaluator = rs_find_erasure_evaluator(&syndromes, &erasure_locator);

    // TODO are these error locations? is this Chien search?
    // find error locations with Chien search
    let erasure_roots = rs_find_erasure_roots(&erasure_coeffs);

    // find error magnitude using Forney's algorithm
    let erasure_magnitude = rs_find_erasure_magnitude(
        &erasure_evaluator,
        &erasure_roots,
        erasures
    );

    // correct the errors
    rs_poly_add(message_poly, &erasure_magnitude);
}

/// Correct up to ECC_SIZE/2 errors at unknown locations
pub fn rs_correct_errors(message: &mut [u8]) {
    let message_poly = rs_poly_from_slice_mut(message);

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = rs_find_syndromes(message_poly);
    if syndromes.iter().all(|s| *s == gf256(0)) {
        return;
    }

    // TODO need erasure_count here?
    let error_locator = rs_find_error_locator(&syndromes);

    // TODO error here
    let error_count = error_locator.len() - 1;
    assert!(error_count*2 <= ECC_SIZE);

    let errors = rs_find_errors(&error_locator);

    rs_correct_erasures(message, &errors);
}

/// Correct a mixture of erasures at unknown locations and erasures
/// as known locations, can correct up to 2*errors+erasures <= ECC_SIZE
pub fn rs_correct(message: &mut [u8], erasures: &[usize]) {
    let message_poly = rs_poly_from_slice_mut(message);

    // convert locations to coefficients (reversed order)
    let erasure_coeffs = erasures.iter()
        .map(|i| message_poly.len()-1-i)
        .collect::<Vec<_>>();

    // find syndromes, syndromes of all zero means there are no errors
    let syndromes = rs_find_syndromes(message_poly);
    if syndromes.iter().all(|s| *s == gf256(0)) {
        return;
    }

    // find Forney syndromes, truncate to number of errors
    let fsyndromes = rs_find_forney_syndromes(&syndromes, &erasure_coeffs);

    // TODO need erasure_count here?
    let error_locator = rs_find_error_locator(&fsyndromes);

    // TODO error here
    let error_count = error_locator.len() - 1;
    let erasure_count = erasure_coeffs.len();
    assert!(error_count*2 + erasure_count <= ECC_SIZE);

    let mut errors = rs_find_errors(&error_locator);

    errors.extend_from_slice(erasures);
    rs_correct_erasures(message, &errors);
}



fn main() {
    fn hex(xs: &[u8]) -> String {
        xs.iter()
            .map(|x| format!("{:02x}", x))
            .collect()
    }

    let orig_message = b"Hello World!";

    let test = rs_find_error_locator(rs_poly_from_slice(&[0x9b, 0xf4, 0xb2, 0xc6, 0xc9, 0x9e]));
    println!("=> {:?}", test);
    assert!(test.len() <= 4);

    println!();
    println!("testing rs({:?})", String::from_utf8_lossy(orig_message));

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
    println!("{:<19} => {:<31} {}",
        "rs_encode",
        format!("{:?}", String::from_utf8_lossy(&message)),
        hex(&message)
    );

    // we can correct up to ECC_SIZE erasures (known location)
    let mut rng = rand::thread_rng();
    let errors = rand::seq::index::sample(&mut rng, message.len(), ECC_SIZE).into_vec();
    for error in errors.iter() {
        message[*error] = b'x';
    }
    println!("{:<19} => {:<31} {}",
        format!("corrupted ({},{})", ECC_SIZE, 0),
        format!("{:?}", String::from_utf8_lossy(&message)),
        hex(&message)
    );

    rs_correct_erasures(&mut message, &errors);
    println!("{:<19} => {:<31} {}",
        "rs_correct_erasures",
        format!("{:?}", String::from_utf8_lossy(&message)),
        hex(&message)
    );
    assert_eq!(
        &message[0..12],
        orig_message
    );

    // we can correct up to ECC_SIZE/2 errors (unknown locations)
    let mut rng = rand::thread_rng();
    let errors = rand::seq::index::sample(&mut rng, message.len(), ECC_SIZE/2).into_vec();
    for error in errors.iter() {
        message[*error] = b'x';
    }
    println!("{:<19} => {:<31} {}",
        format!("corrupted ({},{})", 0, ECC_SIZE/2),
        format!("{:?}", String::from_utf8_lossy(&message)),
        hex(&message)
    );

    rs_correct_errors(&mut message);
    println!("{:<19} => {:<31} {}",
        "rs_correct_errors",
        format!("{:?}", String::from_utf8_lossy(&message)),
        hex(&message)
    );
    assert_eq!(
        &message[0..12],
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
    println!("{:<19} => {:<31} {}",
        format!("corrupted ({},{})", erasure_count, (ECC_SIZE-erasure_count)/2),
        format!("{:?}", String::from_utf8_lossy(&message)),
        hex(&message)
    );

    rs_correct(&mut message, &errors[..erasure_count]);
    println!("{:<19} => {:<31} {}",
        "rs_correct",
        format!("{:?}", String::from_utf8_lossy(&message)),
        hex(&message)
    );
    assert_eq!(
        &message[0..12],
        orig_message
    );

    println!();
}
