## gf256

A Rust library containing Galois-field types and utilities, leveraging hardware
instructions (mostly carry-less multiplication) when available.

This project started as a learning project to learn more about these
"Galois-field thingies" after seeing them pop up in way too many subjects, so
this crate may be more educational than practical. PRs welcome.

``` rust
use ::gf256::*;

let a = gf256(0xfd);
let b = gf256(0xfe);
let c = gf256(0xff);
assert_eq!(a*(b+c), a*b + a*c);
```

## What are Galois-fields?

Galois-fields, also called Finite-fields, are a finite set of "numbers" (for
some definition of number), that you can do "math" on (for some definition
of math).

More specifically, Galois-fields support addition, subtraction, multiplication,
and division, which follow a set of rules called "field axioms":

1. Subtraction is the inverse of addition, and division is the inverse of
   multiplication:
 
   ``` rust
   # use ::gf256::*;
   #
   # let a = gf256(1);
   # let b = gf256(2);
   assert_eq!((a+b)-b, a);
   assert_eq!((a*b)/b, a);
   ```
 
   Except for 0, over which division is undefined:
 
   ``` rust
   # use ::gf256::*;
   #
   # let a = gf256(1);
   assert_eq!(a.checked_div(gf256(0)), None);
   ```
 
1. There exists an element 0 that is the identity of addition, and an element
   1 that is the identity of multiplication:
 
   ``` rust
   # use ::gf256::*;
   #
   # let a = gf256(1);
   assert_eq!(a + gf256(0), a);
   assert_eq!(a * gf256(1), a);
   ```
 
1. Addition and multiplication are associative:
 
   ``` rust
   # use ::gf256::*;
   #
   # let a = gf256(1);
   # let b = gf256(2);
   # let c = gf256(3);
   assert_eq!(a+(b+c), (a+b)+c);
   assert_eq!(a*(b*c), (a*b)*c);
   ```
 
1. Addition and multiplication are commutative:
 
   ``` rust
   # use ::gf256::*;
   #
   # let a = gf256(1);
   # let b = gf256(2);
   assert_eq!(a+b, b+a);
   assert_eq!(a*b, b*a);
   ```
 
1. Multiplication is distributive over addition:
 
   ``` rust
   # use ::gf256::*;
   #
   # let a = gf256(1);
   # let b = gf256(2);
   # let c = gf256(3);
   assert_eq!(a*(b+c), a*b + a*c);
   ```

Keep in mind these aren't your normal integer operations! The operations
defined in a Galois-field types satisfy the above rules, but they may have
unintuitive results:

``` rust
# use ::gf256::*;
#
assert_eq!(gf256(1) + gf256(1), gf256(0));
```

This also means not all of math works in a Galois-field:

``` rust
# use ::gf256::*;
#
# let a = gf256(1);
assert_ne!(a + a, gf256(2)*a);
```

Unsurprisingly, finite-fields can be very useful for applying high-level math
onto machine words, since machine words (u8, u16, u32, etc) are inherently
finite, we just normally try to ignore this limitation.

In Rust this has the fun feature that the Galois-field types can not overflow,
so Galois-field types don't need the set of overflowing operations normally
found in other Rust types:

``` rust
# use ::gf256::*;
#
let a = (u8::MAX).checked_add(1);  // overflows
let b = gf256(u8::MAX) + gf256(1); // does not overflow
```

## Galois-field construction

There are several methods for constructing finite-fields.

On common method is to perform all operations modulo a prime number:

``` text
a + b = (a + b) % p
a * b = (a * b) % p
```

This works, but only when the size of the finite-field is prime. And
unfortunately for us, the size of our machine words are very not prime!

Instead, we use what's called a "binary-extension field".

Consider a binary number:

``` text
a = 0b11011011
```

Normally we would view this as the binary representation of the number 219.

Instead, lets view it as a polynomial for some made-up variable "x", where
each coefficient is a binary 1 or 0:

``` text
a = 0b1011 = 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
```

We can add polynomials together, as long as we mod each coefficient by 2 so
they remain binary:

``` text
a   = 0b1011 = 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
b   = 0b1101 = 0*x^3 + 1*x^2 + 0*x^1 + 1*x^0

a+b = ((1+1)%2)*x^3 + ((0+1)%2)*x^2 + ((1+0)%2)*x^1 + ((1+1)%2)*x^0
    = 0*x^3 + 1*x^2 + 1*x^1 + 0*x^0
    = 0b0110
```

You may recognize that this is actually xor! One fun way of viewing
binary-extension fields is a generalization of xor.

But there's more, we can also multiply polynomials together:

``` text
a   = 0b1011 = 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
b   = 0b1101 = 1*x^3 + 1*x^2 + 0*x^1 + 1*x^0

a*b = 1*x^6 + 0*x^5 + 1*x^4 + 1*x^3
            + 1*x^5 + 0*x^4 + 1*x^3 + 1*x^2
                            + 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
    = 1*x^6 + ((0+1)%2)*x^5 + ((1+0)%2)*x^4 + ((1+1+1)%2)*x^3 + ((1+0)%2)*x^2 + 1*x^1 + 1*x^0
    = 1*x^6 + 1*x^5 + 1*x^4 + 1*x^3 + 1*x^2 + 1*x^1 + 1*x^0
    = 0b1111111
```

It's worth emphasizing that the "x" in these polynomials is a variable that we
never actually evaluate. We just use it to create a view of the underlying
binary numbers that we can do polynomial operations on.

gf256 actually comes with a set of polynomial types to perform these
operations directly:

``` rust
# use ::gf256::*;
#
assert_eq!(p8(0b1011) + p8(0b1101), p8(0b0110));
assert_eq!(p8(0b1011) * p8(0b1101), p8(0b1111111));
```

But you may notice we have a problem. Polynomial addition (xor!) does not
change the size of our polynomial, but polynomial multiplication does. If we
tried to create a finite-field using these operations as they are right now,
multiplication would escape the field!

So we need a trick to keep multiplication in our field, and the answer, like
any good math problem, is more prime numbers!

In order to keep multiplication in our field, we define multiplication as
polynomial multiplication modulo a "prime number". Except we aren't dealing
with normal numbers, we're dealing with polynomials, so we call it an
"irreducible polynomial". An "irreducible polynomial" being a polynomial that
can't be divided evenly by any other polynomial.

I'm going to use the polynomial `0b10011` in this example, but it can be any
polynomial as long as it's 1. irreducible and 2. has n+1 bits where n is the
number of bits in our field.

1. Why does the polynomial need to be irreducible?
 
   It's for the same reason we have to use a prime number for prime fields. If
   we use a polynomial with factors, we actually end up with multiple smaller
   fields that don't intersect. Numbers in field constructable by one factor
   never end up in the field constructed by the other number and vice-versa.
 
1. Why does the polynomial need n+1 bits?
 
   This is so the resulting polynomial after finding the remainder has n-bits.
 
   Much like how the prime number that defines prime fields is outside that
   field, the irreducible polynomial that defines a binary-extension field is
   outside of the field.

TODO show tool here

Multiplication modulo our irreducible polynomial now always stays in our field:

``` rust
# use ::gf256::*;
#
assert_eq!((p8(0b1011) * p8(0b1101)) % p8(0b10011), p8(0b0110));
assert_eq!((p8(0b1111) * p8(0b1111)) % p8(0b10011), p8(0b1010));
assert_eq!((p8(0b1100) * p8(0b1001)) % p8(0b10011), p8(0b0110));
```

That's great! Now we have an addition and multiplication that stays in our field
while following all of our field axioms:

``` rust
# use ::gf256::*;
#
// a*(b+c) == a*b + a*c
let a = p8(0b0001);
let b = p8(0b0010);
let c = p8(0b0011);
assert_eq!((a * (b+c)) % p8(0b10011), p8(0b0001));
assert_eq!((a*b) % p8(0b10011) + (a*c) % p8(0b10011), p8(0b0001));
```

But we need the inverses: subtraction and division.

Fortunately subtraction is easy to define, it's polynomial subtraction with
each coefficient modulo 2 in order to remain binary, which is... the exact same
as addition. But that's perfectly fine! Polynomial addition (xor!) is its own
inverse:

``` rust
# use ::gf256::*;
#
// (a+b)-b == a
let a = p8(0b0001);
let b = p8(0b0010);
assert_eq!(a + b, p8(0b0011));
assert_eq!(p8(0b0011) - b, p8(0b0001));
```

Division is a bit more complicated. In this case we need to look at some of the
more quirky properties of finite-fields.

If we take any number in our finite-field and repeatedly multiply it to itself,
aka exponentiation, we will eventually end up with our original number since
there are only a finite set of numbers. This is called a "multiplicative cycle",
and it turns out that the length of every multiplicative cycle is a factor of
the number of non-zero number in our field.

``` rust
# use ::gf256::*;
#
fn multiplicative_cycle(a: p8) -> usize {
    let mut x = a;
    let mut i = 0;
    loop {
        x = (x * a) % p8(0b10011);
        i += 1;

        if x == a {
            return i;
        }
    }
}

assert_eq!(multiplicative_cycle(p8(0b0001)), 1);
assert_eq!(multiplicative_cycle(p8(0b0110)), 3);
assert_eq!(multiplicative_cycle(p8(0b1000)), 5);
```

In fact, there are several numbers in every finite-field whose multiplicative
cycle actually includes every non-zero number in the field. These are called
"generators" or "primitive elements" and are very useful for several
algorithms.

``` rust
# use ::gf256::*;
#
# fn multiplicative_cycle(a: p8) -> usize {
#     let mut x = a;
#     let mut i = 0;
#     loop {
#         x = (x * a) % p8(0b10011);
#         i += 1;
#
#         if x == a {
#             return i;
#         }
#     }
# }
#
assert_eq!(multiplicative_cycle(p8(0b0010)), 15);
```

This has some very interesting implications.

If we raise any non-zero number in our field to the power of 15 (the number of
non-zero elements), we will end up with our original number:

``` rust
# use ::gf256::*;
#
fn pow(a: p8, exp: usize) -> p8 {
    let mut x = a;
    for _ in 0..exp {
        x = (x * a) % p8(0b10011);
    }
    x
}

// a^15 = a
assert_eq!(pow(p8(0b0001), 15), p8(0b0001));
assert_eq!(pow(p8(0b0110), 15), p8(0b0110));
assert_eq!(pow(p8(0b1000), 15), p8(0b1000));
assert_eq!(pow(p8(0b0010), 15), p8(0b0010));
```

If we raise any non-zero number in our field to the power of 15-1 (the number
of non-zero elements - 1), we will end up with the number divided by itself,
aka the identity of multiplication, aka 1.

``` rust
# use ::gf256::*;
#
# fn pow(a: p8, exp: usize) -> p8 {
#     let mut x = a;
#     for _ in 0..exp {
#         x = (x * a) % p8(0b10011);
#     }
#     x
# }
# 
// a^15-1 = 1
assert_eq!(pow(p8(0b0001), 15-1), p8(0b0001));
assert_eq!(pow(p8(0b0110), 15-1), p8(0b0001));
assert_eq!(pow(p8(0b1000), 15-1), p8(0b0001));
assert_eq!(pow(p8(0b0010), 15-1), p8(0b0001));
```

And, fascinatingly, if we raise any non-zero number of our field to the power
of 15-2 (the number of non-zero elements - 2), we will end up with the number
divided by itself twice, which is the multiplicative inverse of the original
number. Isn't that neat!

``` rust
# use ::gf256::*;
#
# fn pow(a: p8, exp: usize) -> p8 {
#     let mut x = a;
#     for _ in 0..exp {
#         x = (x * a) % p8(0b10011);
#     }
#     x
# }
# 
// a^15-2 = a^-1
assert_eq!(pow(p8(0b0001), 15-2), p8(0b0001));
assert_eq!(pow(p8(0b0110), 15-2), p8(0b0111));
assert_eq!(pow(p8(0b1000), 15-2), p8(0b1111));
assert_eq!(pow(p8(0b0010), 15-2), p8(0b1001));

// a*a^-1 = 1
assert_eq!((p8(0b0001) * p8(0b0001)) % p8(0b10011), p8(0b0001));
assert_eq!((p8(0b0110) * p8(0b0111)) % p8(0b10011), p8(0b0001));
assert_eq!((p8(0b1000) * p8(0b1111)) % p8(0b10011), p8(0b0001));
assert_eq!((p8(0b0010) * p8(0b1001)) % p8(0b10011), p8(0b0001));
```

This means we can define division in terms of repeated multiplication, which
gives us the last of our field operations!

``` text
a + b = a + b
a - b = a - b
a * b = (a * b) % p
         (2^n)-1-2
a / b = b * (π a % p)

where a and b are viewed as polynomials, p is an irreducible polynomial with
n+1 bits, and n is the number of bits in our field.
```

These operations follow our field axioms:

``` rust
# use ::gf256::*;
#
fn add(a: p8, b: p8) -> p8 {
    a + b
}

fn sub(a: p8, b: p8) -> p8 {
    a - b
}

fn mul(a: p8, b: p8) -> p8 {
    (a * b) % p8(0b10011)
}

fn div(a: p8, b: p8) -> p8 {
    let mut x = b;
    for _ in 0..(15-2) {
        x = mul(x, b);
    }
    mul(a, x)
}

# let a = p8(0b0001);
# let b = p8(0b0010);
# let c = p8(0b0011);
#
// inverse
assert_eq!(sub(add(a, b), b), a);
assert_eq!(div(mul(a, b), b), a);

// identity
assert_eq!(add(a, p8(0)), a);
assert_eq!(mul(a, p8(1)), a);

// associativity
assert_eq!(add(a, add(b, c)), add(add(a, b), c));
assert_eq!(mul(a, mul(b, c)), mul(mul(a, b), c));

// commutativity
assert_eq!(add(a, b), add(b, a));
assert_eq!(mul(a, b), mul(b, a));

// distribution
assert_eq!(mul(a, add(b, c)), add(mul(a, b), mul(a, c)));
```

And this is how our Galois-field types are defined! gf256 uses several
different techniques to optimize these operations, but they are built on the
same underlying theory.

We generally name these fields GF(n), where n is the number of elements in the
field, in honor of Évariste Galois the mathematician who created this branch
of mathematics. So since this field is defined for 4-bits, we can call this
field GF(16).

We can create this exact field using gf256:

``` rust ignore
use ::gf256::macros::gf;

##[gf(polynomial=0b10011, generator=0b0010)]
type gf16;

assert_eq!(gf16::new(0b1011) * gf16::new(0b1101), gf16::new(0b0110));
```

## Included in gf256

gf256 contains a bit more than the Galois-field types. It also contains a
number of other utilities that are based on the math around finite-fields:

- **Polynomial types**

  ``` rust
  use ::gf256::*;
  
  let a = p32(0x1234);
  let b = p32(0x5678);
  assert_eq!(a*b, p32(0x05c58160));
  ```

- **Galois-field types**

  ``` rust
  use ::gf256::*;
  
  let a = gf256(0xfd);
  let b = gf256(0xfe);
  let c = gf256(0xff);
  assert_eq!(a*(b+c), a*b + a*c);
  ```

- **CRC functions** (requires feature `crc`)

  ``` rust
  use ::gf256::crc::crc32c;

  assert_eq!(crc32c(b"Hello World!"), 0xfe6cf1dc);
  ```

- **LFSR structs** (requires feature `lfsr`)

  ``` rust
  # use std::iter;
  use ::gf256::lfsr::Lfsr16;

  let mut lfsr = Lfsr16::new(1);
  assert_eq!(lfsr.next(16), 0x0001);
  assert_eq!(lfsr.next(16), 0x002d);
  assert_eq!(lfsr.next(16), 0x0451);
  assert_eq!(lfsr.next(16), 0xbdad);
  assert_eq!(lfsr.prev(16), 0xbdad);
  assert_eq!(lfsr.prev(16), 0x0451);
  assert_eq!(lfsr.prev(16), 0x002d);
  assert_eq!(lfsr.prev(16), 0x0001);
  ```

- **RAID-parity functions** (requires feature `raid`)

  ``` rust
  use ::gf256::raid::raid6;

  // format
  let mut buf = b"Hello World!".to_vec();
  let mut parity1 = vec![0u8; 4];
  let mut parity2 = vec![0u8; 4];
  let slices = buf.chunks(4).collect::<Vec<_>>();
  raid6::format(&slices, &mut parity1, &mut parity2);

  // corrupt
  buf[0..8].fill(b'x');

  // repair
  let mut slices = buf.chunks_mut(4).collect::<Vec<_>>();
  raid6::repair(&mut slices, &mut parity1, &mut parity2, &[0, 1]);
  assert_eq!(&buf, b"Hello World!");
  ```

- **Reed-Solomon error-correction functions** (requires feature `rs`)

  ``` rust
  use ::gf256::rs::rs255w223;

  // encode
  let mut buf = b"Hello World!".to_vec();
  buf.resize(buf.len()+32, 0u8);
  rs255w223::encode(&mut buf);

  // corrupt
  buf[0..16].fill(b'x');

  // correct
  rs255w223::correct_errors(&mut buf)?;
  assert_eq!(&buf[0..12], b"Hello World!");
  # Ok::<(), rs255w223::Error>(())
  ```

- **Shamir secret-sharing functions** (requires features `shamir` and `thread-rng`)

  ``` rust
  use ::gf256::shamir::shamir;

  // generate shares
  let shares = shamir::generate(b"Hello World!", 5, 4);

  // <4 can't reconstruct secret
  assert_ne!(shamir::reconstruct(&shares[..1]), b"Hello World!");
  assert_ne!(shamir::reconstruct(&shares[..2]), b"Hello World!");
  assert_ne!(shamir::reconstruct(&shares[..3]), b"Hello World!");

  // >=4 can reconstruct secret
  assert_eq!(shamir::reconstruct(&shares[..4]), b"Hello World!");
  assert_eq!(shamir::reconstruct(&shares[..5]), b"Hello World!");
  ```

Since this math depends on some rather arbitrary constants, each of these
utilities is available as both a normal Rust API, defined using reasonable
defaults, and as a highly configurable proc_macro:

``` rust ignore
use ::gf256::macros::gf;

#[gf(polynomial=0x11b, generator=0x3)]
type gf256_rijndael;

let a = gf256_rijndael(0xfd);
let b = gf256_rijndael(0xfe);
let c = gf256_rijndael(0xff);
assert_eq!(a*(b+c), a*b + a*c);
```

## Hardware instructions

## `const fns`

## `no_std` support

gf256 is just a pile of math, so it is mostly `no_std` compatible.

The exception is the extra utilities `rs` and `shamir`, which require `alloc`.

## Constant-time

## Features

## Testing

## Benchmarking

## License




https://en.wikipedia.org/wiki/Field_(mathematics)
https://en.wikipedia.org/wiki/Finite_field
https://en.wikipedia.org/wiki/Finite_field_arithmetic
