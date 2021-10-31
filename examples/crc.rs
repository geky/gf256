//! 32-bit CRC implementations using the polynomial types
//!
//! CRC is defined as the polynomial remainder in gf(2) of
//! the input over a constant primitive polynomial. The constant
//! polynomial used in these examples is 0x104c11db7 (also written
//! 0x04c11db7), which defines the common CRC32 32-bit CRC.
//!
//! Note that the common CRC32 32-bit CRC operates on polynomials
//! in big-endian bit order, requiring that we bit-reverse the
//! input and output to work without our polynomial types.
//!
//! We also bit-invert the CRC before and after calculating the remainder
//! to match CRC32, this is to prevent unchecked leading zeros.
//!

use std::iter;
use std::convert::TryFrom;
use gf256::traits::FromLossy;
use gf256::*;

/// This is a common polynomial for 32-bit CRCs, normally the highest
/// bit of the polynomial is omitted, so this may often be seen as just
/// 0x04c11db7
///
const POLYNOMIAL: p64 = p64(0x104c11db7);

/// A naive CRC implementation using the textbook definition of polynomial
/// remainder, the input is padded with 32-bits of zeros to represent the
/// correct polynomial.
///
/// The bit-invert of the CRC is a bit strange when mapped to the
/// textbook definition as this appears as xoring the input with
/// 32-bits of ones followed by zeros.
///
/// We also have to bit-reverse the input/output in order to match
/// the common CRC32 behavior.
///
pub fn naive_crc(data: &[u8]) -> u32 {
    let mut crc = p64(0);

    for b in
        data.iter().copied()
            // pad with 32-bits
            .chain(iter::repeat(0x00).take(4))
            // invert the first 32-bits
            .zip(iter::repeat(0xff).take(4).chain(iter::repeat(0x00)))
            .map(|(m, b)| m ^ b)
    {
        crc = (crc << 8) | p64::from(b.reverse_bits());
        crc = crc % POLYNOMIAL;
    }

    u32::try_from(crc).unwrap().reverse_bits() ^ 0xffffffff
}

/// A CRC implementation that uses the first common optimization:
/// delaying the addition of the next byte to when overflow can occur
///
pub fn less_naive_crc(data: &[u8]) -> u32 {
    let mut crc = p32(0xffffffff);

    for b in data {
        crc = crc + (p32::from(b.reverse_bits()) << 24);
        crc = p32::try_from((p64::from(crc) << 8) % POLYNOMIAL).unwrap();
    }

    u32::from(crc).reverse_bits() ^ 0xffffffff
}

/// A CRC implementation using the same technique as less_naive_crc but
/// operating on a 32-bit word at a time
///
pub fn word_less_naive_crc(data: &[u8]) -> u32 {
    let mut crc = p32(0xffffffff);

    // iterate over 4-byte words
    let mut words = data.chunks_exact(4);
    for word in &mut words {
        let word = <[u8; 4]>::try_from(word).unwrap();
        crc = crc + p32::from_le_bytes(word).reverse_bits();
        crc = p32::try_from((p64::from(crc) << 32) % POLYNOMIAL).unwrap();
    }

    for b in words.remainder() {
        crc = crc + (p32::from(b.reverse_bits()) << 24);
        crc = p32::try_from((p64::from(crc) << 8) % POLYNOMIAL).unwrap();
    }

    u32::from(crc).reverse_bits() ^ 0xffffffff
}

/// A table-based CRC implementation using precomputed remainders
/// post-addition
///
/// This requires a 4*256 = 1024 byte table (computed at compile-time thanks
/// to Rust's const evaluation), and is the most common CRC implementation
/// thanks to its portability and speed.
///
pub fn table_crc(data: &[u8]) -> u32 {
    const CRC_TABLE: [u32; 256] = {
        let mut table = [0; 256];
        let mut i = 0;
        while i < table.len() {
            let x = (i as u32).reverse_bits();
            let x = p64((x as u64) << 8).naive_rem(POLYNOMIAL).0 as u32;
            table[i] = x.reverse_bits();
            i += 1;
        }
        table
    };

    let mut crc = 0xffffffff;

    for b in data {
        crc = (crc >> 8) ^ CRC_TABLE[usize::from((crc as u8) ^ b)];
    }

    crc ^ 0xffffffff
}

/// A smaller table-based CRC implementation using 4-bit precomputed
/// remainders post-addition
///
/// This requires a 4*16 = 64 byte table (computed at compile-time thanks
/// to Rust's const evaluation), significantly reducing the code-size
/// at the cost of 2x the number of operations. This CRC implementation
/// is common on embedded systems.
///
pub fn small_table_crc(data: &[u8]) -> u32 {
    const CRC_SMALL_TABLE: [u32; 16] = {
        let mut table = [0; 16];
        let mut i = 0;
        while i < table.len() {
            let x = (i as u32).reverse_bits();
            let x = p64((x as u64) << 4).naive_rem(POLYNOMIAL).0 as u32;
            table[i] = x.reverse_bits();
            i += 1;
        }
        table
    };

    let mut crc = 0xffffffff;

    for b in data {
        crc = (crc >> 4) ^ CRC_SMALL_TABLE[usize::from(((crc as u8) ^ (b >> 0)) & 0xf)];
        crc = (crc >> 4) ^ CRC_SMALL_TABLE[usize::from(((crc as u8) ^ (b >> 4)) & 0xf)];
    }

    crc ^ 0xffffffff
}

/// A hardware-accelerated CRC implementation using Barret reduction
///
/// This leverages polynomial multiplication instructions (pclmulqdq,
/// pmul, etc) to provide an efficient CRC implementation without the need
/// of a lookup table.
///
/// You may notice that polynomial multiplication is not the polynomial
/// remainder operation needed for CRC, and that is where Barret reduction
/// comes in. Barret reduction allows you to turn division/remainder
/// by a constant into a cheaper multiply by a different constant.
///
/// Fortunately Rust makes it easy to precompute this constant at
/// compile-time.
///
pub fn barret_crc(data: &[u8]) -> u32 {
    const BARRET_CONSTANT: p64 = {
        p64(p128(0x10000000000000000)
            .naive_div(p128(POLYNOMIAL.0 as u128)).0 as u64)
    };

    let mut crc = p32(0xffffffff);

    for b in data {
        crc = crc ^ (p32::from(b.reverse_bits()) << 24);
        let q = (p64::from(crc >> 24)*BARRET_CONSTANT) >> 32;
        crc = p32::from_lossy(q*POLYNOMIAL) + (crc << 8);
    }

    u32::from(crc).reverse_bits() ^ 0xffffffff
}

/// A hardware-accelerated CRC implementation using the same technique as
/// barret_crc, but operating on a 32-bit word at a time
///
pub fn word_barret_crc(data: &[u8]) -> u32 {
    const BARRET_CONSTANT: p64 = {
        p64(p128(0x10000000000000000)
            .naive_div(p128(POLYNOMIAL.0 as u128)).0 as u64)
    };

    let mut crc = p32(0xffffffff);

    // iterate over 4-byte words
    let mut words = data.chunks_exact(4);
    for word in &mut words {
        let word = <[u8; 4]>::try_from(word).unwrap();
        crc = crc ^ p32::from_le_bytes(word).reverse_bits();
        let q = (p64::from(crc)*BARRET_CONSTANT) >> 32;
        crc = p32::from_lossy(q*POLYNOMIAL);
    }

    for b in words.remainder() {
        crc = crc ^ (p32::from(b.reverse_bits()) << 24);
        let q = (p64::from(crc >> 24)*BARRET_CONSTANT) >> 32;
        crc = p32::from_lossy(q*POLYNOMIAL) + (crc << 8);
    }

    u32::from(crc).reverse_bits() ^ 0xffffffff
}


fn main() {
    let input = b"Hello World!";
    let expected = 0x1c291ca3;
    println!();
    println!("testing crc({:?})", String::from_utf8_lossy(input));

    let output = naive_crc(input);
    println!("{:<19} => 0x{:08x}", "naive_crc", output);
    assert_eq!(output, expected);

    let output = naive_crc(input);
    println!("{:<19} => 0x{:08x}", "less_naive_crc", output);
    assert_eq!(output, expected);

    let output = naive_crc(input);
    println!("{:<19} => 0x{:08x}", "word_less_naive_crc", output);
    assert_eq!(output, expected);

    let output = naive_crc(input);
    println!("{:<19} => 0x{:08x}", "table_crc", output);
    assert_eq!(output, expected);

    let output = naive_crc(input);
    println!("{:<19} => 0x{:08x}", "small_table_crc", output);
    assert_eq!(output, expected);

    let output = naive_crc(input);
    println!("{:<19} => 0x{:08x}", "barret_crc", output);
    assert_eq!(output, expected);

    let output = naive_crc(input);
    println!("{:<19} => 0x{:08x}", "word_barret_crc", output);
    assert_eq!(output, expected);

    println!();
}
