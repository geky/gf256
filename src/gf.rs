//! ## Galois-field types
//!
//! Types representing elements of a binary-extension finite-field.
//!
//! ``` rust
//! use ::gf256::*;
//! 
//! let a = gf256(0xfd);
//! let b = gf256(0xfe);
//! let c = gf256(0xff);
//! assert_eq!(a*(b+c), a*b + a*c);
//! ```
//!
//! ## What are Galois-fields?
//! 
//! [Galois-fields][finite-field], also called finite-fields, are a finite set of
//! "numbers" (for some definition of number), that you can do "math" on (for some
//! definition of math).
//! 
//! More specifically, Galois-fields support addition, subtraction, multiplication,
//! and division, which follow a set of rules called "[field axioms][field-axioms]":
//! 
//! 1. Subtraction is the inverse of addition, and division is the inverse of
//!    multiplication:
//!  
//!    ``` rust
//!    # use ::gf256::*;
//!    #
//!    # let a = gf256(1);
//!    # let b = gf256(2);
//!    assert_eq!((a+b)-b, a);
//!    assert_eq!((a*b)/b, a);
//!    ```
//!  
//!    Except for `0`, over which division is undefined:
//!  
//!    ``` rust
//!    # use ::gf256::*;
//!    #
//!    # let a = gf256(1);
//!    assert_eq!(a.checked_div(gf256(0)), None);
//!    ```
//!  
//! 1. There exists an element `0` that is the identity of addition, and an element
//!    `1` that is the identity of multiplication:
//!  
//!    ``` rust
//!    # use ::gf256::*;
//!    #
//!    # let a = gf256(1);
//!    assert_eq!(a + gf256(0), a);
//!    assert_eq!(a * gf256(1), a);
//!    ```
//!  
//! 1. Addition and multiplication are associative:
//!  
//!    ``` rust
//!    # use ::gf256::*;
//!    #
//!    # let a = gf256(1);
//!    # let b = gf256(2);
//!    # let c = gf256(3);
//!    assert_eq!(a+(b+c), (a+b)+c);
//!    assert_eq!(a*(b*c), (a*b)*c);
//!    ```
//!  
//! 1. Addition and multiplication are commutative:
//!  
//!    ``` rust
//!    # use ::gf256::*;
//!    #
//!    # let a = gf256(1);
//!    # let b = gf256(2);
//!    assert_eq!(a+b, b+a);
//!    assert_eq!(a*b, b*a);
//!    ```
//!  
//! 1. Multiplication is distributive over addition:
//!  
//!    ``` rust
//!    # use ::gf256::*;
//!    #
//!    # let a = gf256(1);
//!    # let b = gf256(2);
//!    # let c = gf256(3);
//!    assert_eq!(a*(b+c), a*b + a*c);
//!    ```
//! 
//! Keep in mind these aren't your normal integer operations! The operations
//! defined in a Galois-field types satisfy the above rules, but they may have
//! unintuitive results:
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! assert_eq!(gf256(1) + gf256(1), gf256(0));
//! ```
//! 
//! This also means not all of math works in a Galois-field:
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! # let a = gf256(1);
//! assert_ne!(a + a, gf256(2)*a);
//! ```
//! 
//! Finite-fields can be very useful for applying high-level math onto machine
//! words, since machine words (`u8`, `u16`, `u32`, etc) are inherently finite.
//! Normally we just ignore this until an integer overflow occurs and then we just
//! waive our hands around wailing that math has failed us.
//! 
//! In Rust this has the fun side-effect that the Galois-field types are incapable
//! of overflowing, so Galois-field types don't need the set of overflowing
//! operations normally found in other Rust types:
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! let a = (u8::MAX).checked_add(1);  // overflows          :(
//! let b = gf256(u8::MAX) + gf256(1); // does not overflow  :)
//! ```
//! 
//! ## Galois-field construction
//! 
//! There are several methods for constructing finite-fields.
//! 
//! On common method is to perform all operations modulo a prime number:
//! 
//! ``` text
//! a + b = (a + b) % p
//! a * b = (a * b) % p
//! ```
//! 
//! This works, but only when the size of the finite-field is prime. And
//! unfortunately for us, the size of our machine words are very not prime!
//! 
//! Instead, we use what's called a "binary-extension field".
//! 
//! Consider a binary number:
//! 
//! ``` text
//! a = 0b1011
//! ```
//! 
//! Normally we would view this as the binary representation of the decimal
//! number 11.
//! 
//! Instead, lets view it as a polynomial for some made-up variable `x`, where
//! each coefficient is a binary `1` or `0`:
//! 
//! ``` text
//! a = 0b1011 = 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
//! ```
//! 
//! We can add polynomials together, as long as we mod each coefficient by `2`
//! so they remain binary:
//! 
//! ``` text
//! a   = 0b1011 = 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
//! b   = 0b1101 = 0*x^3 + 1*x^2 + 0*x^1 + 1*x^0
//! 
//! a+b = ((1+1)%2)*x^3 + ((0+1)%2)*x^2 + ((1+0)%2)*x^1 + ((1+1)%2)*x^0
//!     = 0*x^3 + 1*x^2 + 1*x^1 + 0*x^0
//!     = 0b0110
//! ```
//! 
//! You may recognize that this is actually xor!
//! 
//! But there's more, we can also multiply polynomials together:
//! 
//! ``` text
//! a   = 0b1011 = 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
//! b   = 0b1101 = 1*x^3 + 1*x^2 + 0*x^1 + 1*x^0
//! 
//! a*b = 1*x^6 + 0*x^5 + 1*x^4 + 1*x^3
//!             + 1*x^5 + 0*x^4 + 1*x^3 + 1*x^2
//!                             + 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
//!     = 1*x^6 + ((0+1)%2)*x^5 + ((1+0)%2)*x^4 + ((1+1+1)%2)*x^3 + ((1+0)%2)*x^2 + 1*x^1 + 1*x^0
//!     = 1*x^6 + 1*x^5 + 1*x^4 + 1*x^3 + 1*x^2 + 1*x^1 + 1*x^0
//!     = 0b1111111
//! ```
//! 
//! It's worth emphasizing that the `x` in these polynomials is a variable that
//! we never actually evaluate. We just use it to create a view of the underlying
//! binary numbers that we can do polynomial operations on.
//! 
//! gf256 comes with a set of polynomial types to perform these operations
//! directly, and there is more info on these types [`p`'s module-level
//! documentation](mod@p):
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! assert_eq!(p8(0b1011) + p8(0b1101), p8(0b0110));
//! assert_eq!(p8(0b1011) * p8(0b1101), p8(0b1111111));
//! ```
//! 
//! But you may notice we have a problem. Polynomial addition (xor!) does not
//! change the size of our polynomial, but polynomial multiplication does. If we
//! tried to create a finite-field using these operations as they are right now,
//! multiplication would escape the field!
//! 
//! So we need a trick to keep multiplication in our field, and the answer, like
//! any good math problem, is more prime numbers!
//! 
//! In order to keep multiplication in our field, we define multiplication as
//! polynomial multiplication modulo a "prime number". Except we aren't dealing
//! with normal numbers, we're dealing with polynomials, so we call it an
//! "irreducible polynomial". An "irreducible polynomial" being a polynomial that
//! can't be divided evenly by any other polynomial.
//! 
//! I'm going to use the polynomial `0b10011` in this example, but it can be any
//! polynomial as long as it's 1. irreducible and 2. has n+1 bits where n is the
//! number of bits in our field. How we find these polynomials is explained in
//! more detail in the [Finding irreducible polynomials and generators
//! ](#finding-irreducible-polynomials-and-generators) section.
//! 
//! 1. Why does the polynomial need to be irreducible?
//!  
//!    It's for the same reason we have to use a prime number for prime fields. If
//!    we use a polynomial with factors, we actually end up with multiple smaller
//!    fields that don't intersect. Numbers in field constructable by one factor
//!    never end up in the field constructed by the other number and vice-versa.
//!  
//! 1. Why does the polynomial need n+1 bits?
//!  
//!    This is so the resulting polynomial after finding the remainder has n-bits.
//!  
//!    Much like how the prime number that defines prime fields is outside that
//!    field, the irreducible polynomial that defines a binary-extension field is
//!    outside of the field.
//! 
//! Multiplication modulo our irreducible polynomial now always stays in our field:
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! assert_eq!((p8(0b1011) * p8(0b1101)) % p8(0b10011), p8(0b0110));
//! assert_eq!((p8(0b1111) * p8(0b1111)) % p8(0b10011), p8(0b1010));
//! assert_eq!((p8(0b1100) * p8(0b1001)) % p8(0b10011), p8(0b0110));
//! ```
//! 
//! That's great! Now we have an addition and multiplication that stays in our field
//! while following all of our field axioms:
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! // a*(b+c) == a*b + a*c
//! let a = p8(0b0001);
//! let b = p8(0b0010);
//! let c = p8(0b0011);
//! assert_eq!((a * (b+c)) % p8(0b10011), p8(0b0001));
//! assert_eq!((a*b) % p8(0b10011) + (a*c) % p8(0b10011), p8(0b0001));
//! ```
//! 
//! But to satisfy the field-axioms, we need the inverses: subtraction and division.
//! 
//! Fortunately subtraction is easy to define, it's polynomial subtraction with
//! each coefficient modulo 2 in order to remain binary, which is... the exact
//! same as addition. But that's perfectly fine! Polynomial addition (xor!) is its
//! own inverse:
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! // (a+b)-b == a
//! let a = p8(0b0001);
//! let b = p8(0b0010);
//! assert_eq!(a + b, p8(0b0011));
//! assert_eq!(p8(0b0011) - b, p8(0b0001));
//! ```
//! 
//! Division is a bit more complicated. In this case we need to look at some of the
//! more quirky properties of finite-fields.
//! 
//! If we take any number in our finite-field and repeatedly multiply it to itself,
//! aka exponentiation, we will eventually end up with our original number since
//! there are only a finite set of numbers. This is called a "multiplicative cycle",
//! and it turns out that the length of every multiplicative cycle is a factor of
//! the number of non-zero number in our field.
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! fn multiplicative_cycle(a: p8) -> usize {
//!     let mut x = a;
//!     let mut i = 0;
//!     loop {
//!         x = (x * a) % p8(0b10011);
//!         i += 1;
//! 
//!         if x == a {
//!             return i;
//!         }
//!     }
//! }
//! 
//! assert_eq!(multiplicative_cycle(p8(0b0001)), 1);
//! assert_eq!(multiplicative_cycle(p8(0b0110)), 3);
//! assert_eq!(multiplicative_cycle(p8(0b1000)), 5);
//! ```
//! 
//! In fact, there are several numbers in every finite-field whose multiplicative
//! cycle actually includes every non-zero number in the field. These are called
//! "generators" or "primitive elements" and are very useful when designing
//! algorithms.
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! # fn multiplicative_cycle(a: p8) -> usize {
//! #     let mut x = a;
//! #     let mut i = 0;
//! #     loop {
//! #         x = (x * a) % p8(0b10011);
//! #         i += 1;
//! #
//! #         if x == a {
//! #             return i;
//! #         }
//! #     }
//! # }
//! #
//! assert_eq!(multiplicative_cycle(p8(0b0010)), 15);
//! ```
//! 
//! This has some very interesting implications.
//! 
//! If we raise any non-zero number in our 16-element field to the power of `15`
//! (the number of non-zero elements), we will end up with our original number:
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! fn pow(a: p8, exp: usize) -> p8 {
//!     let mut x = a;
//!     for _ in 0..exp {
//!         x = (x * a) % p8(0b10011);
//!     }
//!     x
//! }
//! 
//! // a^15 = a
//! assert_eq!(pow(p8(0b0001), 15), p8(0b0001));
//! assert_eq!(pow(p8(0b0110), 15), p8(0b0110));
//! assert_eq!(pow(p8(0b1000), 15), p8(0b1000));
//! assert_eq!(pow(p8(0b0010), 15), p8(0b0010));
//! ```
//! 
//! If we raise any non-zero number in our field to the power of `15-1` (the
//! number of non-zero elements - 1), we will end up with the number divided by
//! itself, aka the identity of multiplication, aka `1`.
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! # fn pow(a: p8, exp: usize) -> p8 {
//! #     let mut x = a;
//! #     for _ in 0..exp {
//! #         x = (x * a) % p8(0b10011);
//! #     }
//! #     x
//! # }
//! # 
//! // a^15-1 = 1
//! assert_eq!(pow(p8(0b0001), 15-1), p8(0b0001));
//! assert_eq!(pow(p8(0b0110), 15-1), p8(0b0001));
//! assert_eq!(pow(p8(0b1000), 15-1), p8(0b0001));
//! assert_eq!(pow(p8(0b0010), 15-1), p8(0b0001));
//! ```
//! 
//! And, fascinatingly, if we raise any non-zero number of our field to the power
//! of `15-2` (the number of non-zero elements - 2), we will end up with the number
//! divided by itself twice, which is the multiplicative inverse of the original
//! number. Isn't that neat!
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! # fn pow(a: p8, exp: usize) -> p8 {
//! #     let mut x = a;
//! #     for _ in 0..exp {
//! #         x = (x * a) % p8(0b10011);
//! #     }
//! #     x
//! # }
//! # 
//! // a^15-2 = a^-1
//! assert_eq!(pow(p8(0b0001), 15-2), p8(0b0001));
//! assert_eq!(pow(p8(0b0110), 15-2), p8(0b0111));
//! assert_eq!(pow(p8(0b1000), 15-2), p8(0b1111));
//! assert_eq!(pow(p8(0b0010), 15-2), p8(0b1001));
//! 
//! // a*a^-1 = 1
//! assert_eq!((p8(0b0001) * p8(0b0001)) % p8(0b10011), p8(0b0001));
//! assert_eq!((p8(0b0110) * p8(0b0111)) % p8(0b10011), p8(0b0001));
//! assert_eq!((p8(0b1000) * p8(0b1111)) % p8(0b10011), p8(0b0001));
//! assert_eq!((p8(0b0010) * p8(0b1001)) % p8(0b10011), p8(0b0001));
//! ```
//! 
//! This means we can define division in terms of repeated multiplication, which
//! gives us the last of our field operations:
//! 
//! ``` text
//! a + b = a + b
//!
//! a - b = a - b
//!
//! a * b = a * b mod p
//!         (2^n)-1-2
//! a / b = a * ∏ b mod p
//! ```
//! 
//! Where `a` and `b` are viewed as polynomials, `p` is an irreducible polynomial
//! with `n+1` bits, and `n` is the bit-width of our field.
//! 
//! These operations follow our field axioms:
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! fn add(a: p8, b: p8) -> p8 {
//!     a + b
//! }
//! 
//! fn sub(a: p8, b: p8) -> p8 {
//!     a - b
//! }
//! 
//! fn mul(a: p8, b: p8) -> p8 {
//!     (a * b) % p8(0b10011)
//! }
//! 
//! fn div(a: p8, b: p8) -> p8 {
//!     let mut x = b;
//!     for _ in 0..(15-2) {
//!         x = mul(x, b);
//!     }
//!     mul(a, x)
//! }
//! 
//! # let a = p8(0b0001);
//! # let b = p8(0b0010);
//! # let c = p8(0b0011);
//! #
//! // inverse
//! assert_eq!(sub(add(a, b), b), a);
//! assert_eq!(div(mul(a, b), b), a);
//! 
//! // identity
//! assert_eq!(add(a, p8(0)), a);
//! assert_eq!(mul(a, p8(1)), a);
//! 
//! // associativity
//! assert_eq!(add(a, add(b, c)), add(add(a, b), c));
//! assert_eq!(mul(a, mul(b, c)), mul(mul(a, b), c));
//! 
//! // commutativity
//! assert_eq!(add(a, b), add(b, a));
//! assert_eq!(mul(a, b), mul(b, a));
//! 
//! // distribution
//! assert_eq!(mul(a, add(b, c)), add(mul(a, b), mul(a, c)));
//! ```
//! 
//! And this is how our Galois-field types are defined. gf256 uses several
//! different techniques to optimize these operations, but they are built on the
//! same underlying theory.
//! 
//! We generally name these fields `GF(n)`, where `n` is the number of elements in
//! the field, in honor of Évariste Galois the mathematician who created this branch
//! of mathematics. So since this field is defined for 4-bits, we can call this
//! field `GF(16)`.
//! 
//! We can create this exact field using gf256:
//! 
//! ``` rust
//! # use ::gf256::*;
//! use ::gf256::gf::gf;
//! 
//! #[gf(polynomial=0b10011, generator=0b0010)]
//! type gf16;
//!
//! # fn main() {
//! assert_eq!(gf16::new(0b1011) * gf16::new(0b1101), gf16::new(0b0110));
//! # }
//! ```
//!
//! ## Finding irreducible polynomials and generators
//!
//! So far we've mentioned that the representation of Galois-fields in gf256
//! require "irreducible polynomials", and that every Galois-field contains
//! "generators", sometimes called "primitive elements".
//!
//! - irreducible polynomial - a polynomial that can't be evenly divided by any
//!   other non-trivial polynomial
//!
//! - generator - an element of a finite-field whose multiplicative cycle
//!   contains all non-zero elements of the field.
//!
//! But we haven't actually explained how to find these important polynomials.
//!
//! Unfortunately, much like traditional prime numbers, there just isn't a
//! straightforward formula for finding irreducible polynomials or generators.
//! Our best option is a brute force search.
//!
//! To help with this, gf256 contains the tool/example [find-p][find-p], which
//! can search for irreducible polynomials and their generators of a given
//! bit-width. Note you need a 5-bit irreducible polynomial for a 4-bit field:
//!
//! ``` bash
//! $ RUSTFLAGS="-Ctarget-cpu=native" cargo run --release --example find-p -- --width=5 -m=1
//! polynomial=0x13, generator=0x2
//! polynomial=0x19, generator=0x2
//! polynomial=0x1f, generator=0x3
//! ```
//!
//! So far this has been efficient enough to find irreducible polynomials and
//! their generators up to 65-bits:
//!
//! ``` bash
//! $ RUSTFLAGS="-Ctarget-cpu=native" cargo --release --example find-p -- --width=65 -m=1 -n=1
//! polynomial=0x1000000000000001b, generator=0x2
//! ```
//!
//! ## Optimizations
//!
//! There are a number of optimizations we can do to make Galois-fields more
//! efficient.
//!
//! For example, we can use [exponentiation by squaring][exp-by-squaring] to
//! reduce the overhead of exponentiation to `O(log exp)`.
//!
//! This naive implementation of Galois-fields is available with the `naive` mode,
//! though more complex optimization are available, mostly around the expensive
//! polynomial remainder we need to compute during multiplication and division:
//!
//! - In `table` mode, Galois-field types use [precomputed log and anti-log tables][log-tables].
//!
//!   This is the most common implementation of Galois-field types due to its
//!   performance, but this technique really only works for small Galois-fields since
//!   the log and anti-log tables require a number of elements equal to the size
//!   of the finite-field.
//!
//! - In `rem_table` mode, Galois-field types use a precomputed remainder table to
//!   compute the remainder a byte at a time.
//!
//!   This uses the same technique as the precomputed remainder tables in CRC
//!   calculation, since a CRC is just a polynomial remainder. The [`crc`](../crc)
//!   module-level documentation has more info on this.
//!
//!   Surprisingly this technique is less powerful here, perhaps because of the
//!   smaller input size and issues keeping the lookup table in the cache.
//!
//! - In `small_rem_table` mode, the same strategy as `rem_table` mode is used, but
//!   with a 16 element remainder table computing the remainder a nibble at a time.
//!
//! - In `barret` mode,  Galois-field types use [Barret-reduction][barret-reduction]
//!   to efficiently compute the remainder using only multiplication by precomputed
//!   constants.
//!
//!   This mode is especially effective when hardware carry-less multiplication
//!   instructions are available.
//!
//! Galois-fields with <=8 bits default to the `table` mode, which is the fastest,
//! but requires two tables the size of the number of elements in the field.
//! Galois-fields >8 bits default to `barret` mode, which, perhaps surprisingly,
//! is the fastest even when hardware carry-less multiplication is not available.
//!
//! If the features `small-tables` or `no-tables` are enabled, `barret` mode is used
//! for all Galois-field types.
//!
//! Though note the default mode is susceptible to change.
//!
//! See also [BENCHMARKS.md][benchmarks]
//!
//! ## `const fn` support
//!
//! Due to the use of traits and intrinsics, it's not possible to use the
//! Galois-field operators in [`const fns`][const-fn].
//!
//! As an alternative, the Galois-field types preovide a set of "naive"
//! functions, which provide less efficient, well, naive, implementations,
//! that can be used in const fns.
//! 
//! These are very useful for calculating complex constants at compile-time:
//! 
//! ``` rust
//! # use ::gf256::*;
//! #
//! const ECC_SIZE: usize = 32;
//! const GENERATOR_POLY: [gf256; ECC_SIZE+1] = {
//!     let mut g = [gf256(0); ECC_SIZE+1];
//!     g[ECC_SIZE] = gf256(1);
//! 
//!     // find G(x)
//!     //
//!     //     ECC_SIZE
//!     // G(x) = ∏  (x - g^i)
//!     //        i
//!     //
//!     let mut i = 0usize;
//!     while i < ECC_SIZE {
//!         // x - g^i
//!         let root = [
//!             gf256(1),
//!             gf256::GENERATOR.naive_pow(i as u8),
//!         ];
//! 
//!         // G(x)*(x - g^i)
//!         let mut product = [gf256(0); ECC_SIZE+1];
//!         let mut j = 0usize;
//!         while j < i+1 {
//!             let mut k = 0usize;
//!             while k < root.len() {
//!                 product[product.len()-1-(j+k)] = product[product.len()-1-(j+k)].naive_add(
//!                     g[g.len()-1-j].naive_mul(root[root.len()-1-k])
//!                 );
//!                 k += 1;
//!             }
//!             j += 1;
//!         }
//!         g = product;
//! 
//!         i += 1;
//!     }
//! 
//!     g
//! };
//! ```
//!
//! ## Constant-time
//!
//! gf256 provides "best-effort" constant-time implementations for certain
//! useful operations.
//!
//! Galois-field types in `barret` mode rely only on carry-less multiplication
//! and xors, and should always execute in constant time.
//!
//! This includes Galois-field addition (xor), subtraction (xor),
//! multiplication, and division.
//!
//! Note that exponentiation is NOT constant-time with regards to the exponent.
//!
//! And the other Galois-field implementations are NOT constant-time due to the use
//! of lookup tables, which may be susceptible to cache-timing attacks. Note that
//! the default Galois-field types likely use a table-based implementation.
//!
//! You will need to declare a custom Galois-field type using `barret` mode if you
//! want constant-time finite-field operations:
//!
//! ``` rust
//! # use ::gf256::*;
//! use gf256::gf::gf;
//! 
//! #[gf(polynomial=0x11b, generator=0x3, barret)]
//! type gf256_rijndael;
//!
//! # fn main() {}
//! ```
//!
//!
//! [finite-field]: https://en.wikipedia.org/wiki/Finite_field
//! [field-axioms]: https://en.wikipedia.org/wiki/Field_(mathematics)
//! [exp-by-squaring]: https://en.wikipedia.org/wiki/Exponentiation_by_squaring
//! [log-tables]: https://en.wikipedia.org/wiki/Finite_field_arithmetic#Generator_based_tables
//! [barret-reduction]: https://en.wikipedia.org/wiki/Barrett_reduction
//! [const-fn]: https://doc.rust-lang.org/reference/const_eval.html
//! [find-p]: https://github.com/geky/gf256/blob/master/examples/find-p.rs
//! [benchmarks]: https://github.com/geky/gf256/blob/master/BENCHMARKS.md


/// A macro for generating custom Galois-field types.
///
/// ``` rust
/// # use ::gf256::*;
/// # use ::gf256::gf::gf;
/// #[gf(polynomial=0x11d, generator=0x2)]
/// pub type my_gf256;
///
/// # fn main() {
/// let a = my_gf256(0xfd);
/// let b = my_gf256(0xfe);
/// let c = my_gf256(0xff);
/// assert_eq!(a*(b+c), a*b + a*c);
/// # }
/// ```
///
/// The `gf` macro accepts a number of configuration options:
///
/// - `polynomial` - The irreducible polynomial that defines the field.
/// - `generator` - A generator, aka primitive element, of the field.
/// - `usize` - Indicate if the width is dependent on the usize width,
///   defaults to true if the `u` type is `usize`.
/// - `u` - The underlying unsigned type, defaults to the minimum sized unsigned
///   type that fits the field.
/// - `u2` - An unsigned type with twice the width, used as an intermediary type
///   for computations, defaults to the correct type based on `u`.
/// - `p` - The polynomial type used for computation, defaults to the
///   polynomial version of `u`.
/// - `p2` - A polynomial type with twice the width, used as an intermediary type
///   for computations, defaults to the correct type based on `p`.
/// - `naive` - Use a naive bitwise implementation.
/// - `table` - Use precomputed log and anti-log tables. This is the default for
///   types <= 8-bits.
/// - `rem_table` - Use a precomputed remainder table.
/// - `small_rem_table` - Use a small, 16-element remainder table.
/// - `barret` - Use Barret-reduction with polynomial multiplication. This is the
///   default for types > 8-bits.
///
/// ``` rust
/// # use ::gf256::*;
/// # use ::gf256::gf::gf;
/// #[gf(
///     polynomial=0x11d,
///     generator=0x2,
///     usize=false,
///     u=u8,
///     u2=u16,
///     p=p8,
///     p2=p16,
///     // naive,
///     // table,
///     // rem_table,
///     // small_rem_table,
///     // barret,
/// )]
/// type my_gf256;
///
/// # fn main() {
/// let a = my_gf256(0xfd);
/// let b = my_gf256(0xfe);
/// let c = my_gf256(0xff);
/// assert_eq!(a*(b+c), a*b + a*c);
/// # }
/// ```
///
pub use gf256_macros::gf;


// An 8-bit binary-extension finite-field
#[gf(polynomial=0x11d, generator=0x2)]
pub type gf256;

// A 16-bit binary-extension finite-field
#[gf(polynomial=0x1002d, generator=0x2)]
pub type gf2p16;

// A 32-bit binary-extension finite-field
#[gf(polynomial=0x1000000af, generator=0x2)]
pub type gf2p32;

// A 64-bit binary-extension finite-field
#[gf(polynomial=0x1000000000000001b, generator=0x2)]
pub type gf2p64;


#[cfg(test)]
mod test {
    use super::*;
    use crate::p::*;

    // Create a custom gf type here (Rijndael's finite field) to test a
    // different polynomial
    #[gf(polynomial=0x11b, generator=0x3)]
    type gf256_rijndael;

    // Test both table-based and Barret reduction implementations
    #[gf(polynomial=0x11d, generator=0x2, table)]
    type gf256_table;
    #[gf(polynomial=0x11d, generator=0x2, rem_table)]
    type gf256_rem_table;
    #[gf(polynomial=0x11d, generator=0x2, small_rem_table)]
    type gf256_small_rem_table;
    #[gf(polynomial=0x11d, generator=0x2, barret)]
    type gf256_barret;

    #[test]
    fn add() {
        assert_eq!(gf256(0x12).naive_add(gf256(0x34)), gf256(0x26));
        assert_eq!(gf256_rijndael(0x12).naive_add(gf256_rijndael(0x34)), gf256_rijndael(0x26));

        assert_eq!(gf256(0x12) + gf256(0x34), gf256(0x26));
        assert_eq!(gf256_rijndael(0x12) + gf256_rijndael(0x34), gf256_rijndael(0x26));

        assert_eq!(gf256_table(0x12).naive_add(gf256_table(0x34)), gf256_table(0x26));
        assert_eq!(gf256_rem_table(0x12).naive_add(gf256_rem_table(0x34)), gf256_rem_table(0x26));
        assert_eq!(gf256_small_rem_table(0x12).naive_add(gf256_small_rem_table(0x34)), gf256_small_rem_table(0x26));
        assert_eq!(gf256_barret(0x12).naive_add(gf256_barret(0x34)), gf256_barret(0x26));

        assert_eq!(gf256_table(0x12) + gf256_table(0x34), gf256_table(0x26));
        assert_eq!(gf256_rem_table(0x12) + gf256_rem_table(0x34), gf256_rem_table(0x26));
        assert_eq!(gf256_small_rem_table(0x12) + gf256_small_rem_table(0x34), gf256_small_rem_table(0x26));
        assert_eq!(gf256_barret(0x12) + gf256_barret(0x34), gf256_barret(0x26));
    }

    #[test]
    fn sub() {
        assert_eq!(gf256(0x12).naive_sub(gf256(0x34)), gf256(0x26));
        assert_eq!(gf256_rijndael(0x12).naive_sub(gf256_rijndael(0x34)), gf256_rijndael(0x26));

        assert_eq!(gf256(0x12) - gf256(0x34), gf256(0x26));
        assert_eq!(gf256_rijndael(0x12) - gf256_rijndael(0x34), gf256_rijndael(0x26));

        assert_eq!(gf256_table(0x12).naive_sub(gf256_table(0x34)), gf256_table(0x26));
        assert_eq!(gf256_rem_table(0x12).naive_sub(gf256_rem_table(0x34)), gf256_rem_table(0x26));
        assert_eq!(gf256_small_rem_table(0x12).naive_sub(gf256_small_rem_table(0x34)), gf256_small_rem_table(0x26));
        assert_eq!(gf256_barret(0x12).naive_sub(gf256_barret(0x34)), gf256_barret(0x26));

        assert_eq!(gf256_table(0x12) - gf256_table(0x34), gf256_table(0x26));
        assert_eq!(gf256_rem_table(0x12) - gf256_rem_table(0x34), gf256_rem_table(0x26));
        assert_eq!(gf256_small_rem_table(0x12) - gf256_small_rem_table(0x34), gf256_small_rem_table(0x26));
        assert_eq!(gf256_barret(0x12) - gf256_barret(0x34), gf256_barret(0x26));
    }

    #[test]
    fn mul() {
        assert_eq!(gf256(0x12).naive_mul(gf256(0x34)), gf256(0x0f));
        assert_eq!(gf256_rijndael(0x12).naive_mul(gf256_rijndael(0x34)), gf256_rijndael(0x05));

        assert_eq!(gf256(0x12) * gf256(0x34), gf256(0x0f));
        assert_eq!(gf256_rijndael(0x12) * gf256_rijndael(0x34), gf256_rijndael(0x05));

        assert_eq!(gf256_table(0x12).naive_mul(gf256_table(0x34)), gf256_table(0x0f));
        assert_eq!(gf256_rem_table(0x12).naive_mul(gf256_rem_table(0x34)), gf256_rem_table(0x0f));
        assert_eq!(gf256_small_rem_table(0x12).naive_mul(gf256_small_rem_table(0x34)), gf256_small_rem_table(0x0f));
        assert_eq!(gf256_barret(0x12).naive_mul(gf256_barret(0x34)), gf256_barret(0x0f));

        assert_eq!(gf256_table(0x12) * gf256_table(0x34), gf256_table(0x0f));
        assert_eq!(gf256_rem_table(0x12) * gf256_rem_table(0x34), gf256_rem_table(0x0f));
        assert_eq!(gf256_small_rem_table(0x12) * gf256_small_rem_table(0x34), gf256_small_rem_table(0x0f));
        assert_eq!(gf256_barret(0x12) * gf256_barret(0x34), gf256_barret(0x0f));
    }

    #[test]
    fn div() {
        assert_eq!(gf256(0x12).naive_div(gf256(0x34)), gf256(0xc7));
        assert_eq!(gf256_rijndael(0x12).naive_div(gf256_rijndael(0x34)), gf256_rijndael(0x54));

        assert_eq!(gf256(0x12) / gf256(0x34), gf256(0xc7));
        assert_eq!(gf256_rijndael(0x12) / gf256_rijndael(0x34), gf256_rijndael(0x54));

        assert_eq!(gf256_table(0x12).naive_div(gf256_table(0x34)), gf256_table(0xc7));
        assert_eq!(gf256_rem_table(0x12).naive_div(gf256_rem_table(0x34)), gf256_rem_table(0xc7));
        assert_eq!(gf256_small_rem_table(0x12).naive_div(gf256_small_rem_table(0x34)), gf256_small_rem_table(0xc7));
        assert_eq!(gf256_barret(0x12).naive_div(gf256_barret(0x34)), gf256_barret(0xc7));

        assert_eq!(gf256_table(0x12) / gf256_table(0x34), gf256_table(0xc7));
        assert_eq!(gf256_rem_table(0x12) / gf256_rem_table(0x34), gf256_rem_table(0xc7));
        assert_eq!(gf256_small_rem_table(0x12) / gf256_small_rem_table(0x34), gf256_small_rem_table(0xc7));
        assert_eq!(gf256_barret(0x12) / gf256_barret(0x34), gf256_barret(0xc7));
    }

    #[test]
    fn all_mul() {
        // test all multiplications
        for a in 0..=255 {
            for b in 0..=255 {
                let x = gf256(a).naive_mul(gf256(b));
                let y = gf256(a) * gf256(b);
                let z = gf256_barret(a) * gf256_barret(b);
                let w = gf256_table(a) * gf256_table(b);
                assert_eq!(u8::from(x), u8::from(y));
                assert_eq!(u8::from(x), u8::from(z));
                assert_eq!(u8::from(x), u8::from(w));
            }
        }
    }

    #[test]
    fn all_div() {
        // test all divisions
        for a in 0..=255 {
            for b in 1..=255 {
                let x = gf256(a).naive_div(gf256(b));
                let y = gf256(a) / gf256(b);
                let z = gf256_barret(a) / gf256_barret(b);
                let w = gf256_table(a) / gf256_table(b);
                assert_eq!(u8::from(x), u8::from(y));
                assert_eq!(u8::from(x), u8::from(z));
                assert_eq!(u8::from(x), u8::from(w));
            }
        }
    }

    #[test]
    fn all_recip() {
        // test all reciprocals
        for a in (1..=255).map(gf256) {
            let x = a.naive_recip();
            let y = a.recip();
            let z = a.naive_pow(254);
            let w = a.pow(254);
            let v = gf256(1).naive_div(a);
            let u = gf256(1) / a;
            assert_eq!(x, y);
            assert_eq!(x, z);
            assert_eq!(x, w);
            assert_eq!(x, v);
            assert_eq!(x, u);
        }
    }

    #[test]
    fn mul_div() {
        // test that div is the inverse of mul
        for a in (1..=255).map(gf256) {
            for b in (1..=255).map(gf256) {
                let c = a * b;
                assert_eq!(c / b, a);
                assert_eq!(c / a, b);
            }
        }
    }

    #[test]
    fn pow() {
        // gf256::naive_pow just uses gf256::naive_mul, we want
        // to test with a truely naive pow
        fn naive_pow(a: gf256, exp: u8) -> gf256 {
            let mut x = gf256(1);
            for _ in 0..exp {
                x *= a;
            }
            x
        }

        for a in (0..=255).map(gf256) {
            for b in 0..=255 {
                assert_eq!(a.pow(b), naive_pow(a, b));
            }
        }
    }

    // Test higher/lower order fields
    //
    // These polynomials/generators were all found using the find-p
    // program in the examples in the examples
    //
    #[gf(polynomial=0x13, generator=0x2)]
    type gf16;
    #[gf(polynomial=0x1053, generator=0x2)]
    type gf4096;
    #[gf(polynomial=0x800021, generator=0x2)]
    type gf2p23;

    macro_rules! test_axioms {
        ($name:ident; $gf:ty; $nz:expr; $x:expr) => {
            #[test]
            fn $name() {
                assert_eq!(<$gf>::NONZEROS, $nz);

                let xs = [
                    <$gf>::new(1*$x),
                    <$gf>::new(2*$x),
                    <$gf>::new(3*$x),
                    <$gf>::new(4*$x),
                ];

                for x in xs {
                    for y in xs {
                        for z in xs {
                            // 0 is the identity of addition
                            assert_eq!(x + <$gf>::new(0), x);
                            // 1 is the identity of multiplication
                            assert_eq!(x * <$gf>::new(1), x);
                            // addition and subtraction are inverses
                            assert_eq!((x + y) - y, x);
                            // multiplication and division are inverses
                            assert_eq!((x * y) / y, x);
                            // addition is distributive over multiplication
                            assert_eq!(x*(y + z), x*y + x*z);
                            // haha math
                            assert_eq!((x+y).pow(2), x.pow(2) + y.pow(2));
                        }
                    }
                }
            }
        }
    }

    test_axioms! { gf16_axioms;    gf16;   15;  0x1 }
    test_axioms! { gf256_axioms;   gf256;  255; 0x11 }
    test_axioms! { gf4096_axioms;  gf4096; 4095; 0x111 }
    test_axioms! { gf2p16_axioms;  gf2p16; 65535; 0x1111 }
    test_axioms! { gf2p23_axioms;  gf2p23; 8388607; 0x111111 }
    test_axioms! { gf2p32_axioms;  gf2p32; 4294967295; 0x11111111 }
    test_axioms! { gf2p64_axioms;  gf2p64; 18446744073709551615; 0x1111111111111111 }

    // Test with explicit implementations
    //
    // This introduces a lot of things to compile, but is important to cover
    // niche implementations that are very prone to bugs
    //

    #[gf(polynomial=0x13, generator=0x2, table)]
    type gf16_table;

    test_axioms! { gf16_table_axioms;    gf16_table; 15;  0x1 }
    test_axioms! { gf256_table_axioms;   gf256_table; 255; 0x11 }

    #[gf(polynomial=0x13, generator=0x2, rem_table)]
    type gf16_rem_table;
    #[gf(polynomial=0x1053, generator=0x2, rem_table)]
    type gf4096_rem_table;
    #[gf(polynomial=0x1002d, generator=0x2, rem_table)]
    type gf2p16_rem_table;
    #[gf(polynomial=0x800021, generator=0x2, rem_table)]
    type gf2p23_rem_table;
    #[gf(polynomial=0x1000000af, generator=0x2, rem_table)]
    type gf2p32_rem_table;
    #[gf(polynomial=0x1000000000000001b, generator=0x2, rem_table)]
    type gf2p64_rem_table;

    test_axioms! { gf16_rem_table_axioms;    gf16_rem_table;   15;  0x1 }
    test_axioms! { gf256_rem_table_axioms;   gf256_rem_table;  255; 0x11 }
    test_axioms! { gf4096_rem_table_axioms;  gf4096_rem_table; 4095; 0x111 }
    test_axioms! { gf2p16_rem_table_axioms;  gf2p16_rem_table; 65535; 0x1111 }
    test_axioms! { gf2p23_rem_table_axioms;  gf2p23_rem_table; 8388607; 0x111111 }
    test_axioms! { gf2p32_rem_table_axioms;  gf2p32_rem_table; 4294967295; 0x11111111 }
    test_axioms! { gf2p64_rem_table_axioms;  gf2p64_rem_table; 18446744073709551615; 0x1111111111111111 }

    #[gf(polynomial=0x13, generator=0x2, small_rem_table)]
    type gf16_small_rem_table;
    #[gf(polynomial=0x1053, generator=0x2, small_rem_table)]
    type gf4096_small_rem_table;
    #[gf(polynomial=0x1002d, generator=0x2, small_rem_table)]
    type gf2p16_small_rem_table;
    #[gf(polynomial=0x800021, generator=0x2, small_rem_table)]
    type gf2p23_small_rem_table;
    #[gf(polynomial=0x1000000af, generator=0x2, small_rem_table)]
    type gf2p32_small_rem_table;
    #[gf(polynomial=0x1000000000000001b, generator=0x2, small_rem_table)]
    type gf2p64_small_rem_table;

    test_axioms! { gf16_small_rem_table_axioms;    gf16_small_rem_table;   15;  0x1 }
    test_axioms! { gf256_small_rem_table_axioms;   gf256_small_rem_table;  255; 0x11 }
    test_axioms! { gf4096_small_rem_table_axioms;  gf4096_small_rem_table; 4095; 0x111 }
    test_axioms! { gf2p16_small_rem_table_axioms;  gf2p16_small_rem_table; 65535; 0x1111 }
    test_axioms! { gf2p23_small_rem_table_axioms;  gf2p23_small_rem_table; 8388607; 0x111111 }
    test_axioms! { gf2p32_small_rem_table_axioms;  gf2p32_small_rem_table; 4294967295; 0x11111111 }
    test_axioms! { gf2p64_small_rem_table_axioms;  gf2p64_small_rem_table; 18446744073709551615; 0x1111111111111111 }

    #[gf(polynomial=0x13, generator=0x2, barret)]
    type gf16_barret;
    #[gf(polynomial=0x1053, generator=0x2, barret)]
    type gf4096_barret;
    #[gf(polynomial=0x1002d, generator=0x2, barret)]
    type gf2p16_barret;
    #[gf(polynomial=0x800021, generator=0x2, barret)]
    type gf2p23_barret;
    #[gf(polynomial=0x1000000af, generator=0x2, barret)]
    type gf2p32_barret;
    #[gf(polynomial=0x1000000000000001b, generator=0x2, barret)]
    type gf2p64_barret;

    test_axioms! { gf16_barret_axioms;    gf16_barret;   15;  0x1 }
    test_axioms! { gf256_barret_axioms;   gf256_barret;  255; 0x11 }
    test_axioms! { gf4096_barret_axioms;  gf4096_barret; 4095; 0x111 }
    test_axioms! { gf2p16_barret_axioms;  gf2p16_barret; 65535; 0x1111 }
    test_axioms! { gf2p23_barret_axioms;  gf2p23_barret; 8388607; 0x111111 }
    test_axioms! { gf2p32_barret_axioms;  gf2p32_barret; 4294967295; 0x11111111 }
    test_axioms! { gf2p64_barret_axioms;  gf2p64_barret; 18446744073709551615; 0x1111111111111111 }

    // all Galois-field params
    #[gf(
        polynomial=0x11d,
        generator=0x2,
        usize=false,
        u=u8,
        u2=u16,
        p=p8,
        p2=p16,
    )]
    type gf256_all_params;

    test_axioms! { gf_all_params; gf256_all_params; 255; 0x11 }
}
