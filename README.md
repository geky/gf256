## gf256

A Rust library containing Galois-field types and utilities, leveraging hardware
instructions when available.

This project started as a learning project to learn more about these
"Galois-field thingies" after seeing them pop up in far too many subjects.
So this crate may be more educational than practical.

``` rust
use ::gf256::*;

let a = gf256(0xfd);
let b = gf256(0xfe);
let c = gf256(0xff);
assert_eq!(a*(b+c), a*b + a*c);
```

If you, like me, are interested in learning more about the fascinating utility
of Galois-fields, take a look at the documentation of gf256's modules. I've
tried to comprehensively capture what I've learned, hopefully provided a decent
entry point into learning more about this useful field of math.

I also want to point out that the Rust examples in each module are completely
functional and tested in CI thanks to Rust's [doctest runner][doctest-runner].
Feel free to copy and tweak them to see what happens.

- [p - Polynomial types][p]
- [gf - Galois-field types][gf]
- [lfsr - LFSR structs][lfsr]
- [crc - CRC functions][crc]
- [shamir - Shamir secret-sharing functions][shamir]
- [raid - RAID-parity functions][raid]
- [rs - Reed-Solomon error-correction functions][rs]

## Reed-Solomon error-correction using gf256

``` text
                      ..:::::::::::...                  .:::....:::.:.:.'':.'.:.'::: :   :' ''  '''
                   .:::::::::::::::::::..               ::::::::::::  .:....'.. '.:...  :' :'':': .'
                 .::::::::::::::::::::::::.             ::::::::::' . :..:..:' : '.':::  :.':'' '  :
               .:::::::::::::::::::::::::::.         . ::::::::::   :: '::' .: '' ''. : :: .:..::. '
              .::::::::::::::::::::::::::::::    ... :: :'::::::'   ::':. ':' .: '.:'::'::: ': '...:
              :::::::::::::::::::::::::::::'' .. '::: '     '''     ''..:.:'.''::.   .:.' .'': '. .:
             :::::::::::::::::::::::::::''..::::: '                  : .: .'. :  :::.'.:':.:':: .. :
             :::::::::::::::::::::::'' ..:::::''                    ' :... .': ::.''':' .''. . '  ..
             :::::::::::::::::::'' ..:::::'' .                       : :..':::.:. : .:' :.   .':'.':
         ..: ::::::::::::::''' ..:::::'' ..:::                      :' . :.' .'.'::. ' ::' ':  .:.
      ..:::'  :::::::::'' ..::::::'' ..:::::::                      :' .  '.'::'.:  : .:'' .:.'.:::'
     :::'     ':::'' ...::::::''...::::::::::                        '' ' .'::...' :':...':.. . .' :
    ::'         ...::::::'' ..:::::::::::::'                        . .. ' .:::.'':::.  .':''':::...
         ....:::::::'' ..:::::::::::::::::'                         : '.'  .': :. .:.': . .'  .  ::'
    '::::::::'''    :::::::::::::::::::''                            '.. .: ::: ': ::'::. ' '.': : '
                      '':::::::::::'''                              .:.':.. '''.  : : ':'':.': '.:':
```

## What are Galois-fields?

[Galois-fields][finite-field], also called finite-fields, are a finite set of
"numbers" (for some definition of number), that you can do "math" on (for some
definition of math).

More specifically, Galois-fields support addition, subtraction, multiplication,
and division, which follow a set of rules called "[field axioms][field-axioms]":

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
 
   Except for `0`, over which division is undefined:
 
   ``` rust
   # use ::gf256::*;
   #
   # let a = gf256(1);
   assert_eq!(a.checked_div(gf256(0)), None);
   ```
 
1. There exists an element `0` that is the identity of addition, and an element
   `1` that is the identity of multiplication:
 
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

Finite-fields can be very useful for applying high-level math onto machine
words, since machine words (`u8`, `u16`, `u32`, etc) are inherently finite.
Normally we just ignore this until an integer overflow occurs and then we just
wave our hands around wailing that math has failed us.

In Rust this has the fun side-effect that the Galois-field types are incapable
of overflowing, so Galois-field types don't need the set of overflowing
operations normally found in other Rust types:

``` rust
# use ::gf256::*;
#
let a = (u8::MAX).checked_add(1);  // overflows          :(
let b = gf256(u8::MAX) + gf256(1); // does not overflow  :)
```

For more information on Galois-fields and how we construct them, see the
relevant documentation in [`gf`'s module-level documentation][gf].

## Included in gf256

gf256 contains a bit more than the Galois-field types. It also contains a
number of other utilities that rely on the math around finite-fields:

- [**Polynomial types**][p]

  ``` rust
  use ::gf256::*;
  
  let a = p32(0x1234);
  let b = p32(0x5678);
  assert_eq!(a+b, p32(0x444c));
  assert_eq!(a*b, p32(0x05c58160));
  ```

- [**Galois-field types**][gf]

  ``` rust
  use ::gf256::*;
  
  let a = gf256(0xfd);
  let b = gf256(0xfe);
  let c = gf256(0xff);
  assert_eq!(a*(b+c), a*b + a*c);
  ```

- [**LFSR structs**][lfsr] (requires feature `lfsr`)

  ``` rust
  use gf256::lfsr::Lfsr16;

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

- [**CRC functions**][crc] (requires feature `crc`)

  ``` rust
  use gf256::crc::crc32c;

  assert_eq!(crc32c(b"Hello World!", 0), 0xfe6cf1dc);
  ```

- [**Shamir secret-sharing functions**][shamir] (requires features `shamir` and `thread-rng`)

  ``` rust
  use gf256::shamir::shamir;

  // generate shares
  let shares = shamir::generate(b"secret secret secret!", 5, 4);

  // <4 can't reconstruct secret
  assert_ne!(shamir::reconstruct(&shares[..1]), b"secret secret secret!");
  assert_ne!(shamir::reconstruct(&shares[..2]), b"secret secret secret!");
  assert_ne!(shamir::reconstruct(&shares[..3]), b"secret secret secret!");

  // >=4 can reconstruct secret
  assert_eq!(shamir::reconstruct(&shares[..4]), b"secret secret secret!");
  assert_eq!(shamir::reconstruct(&shares[..5]), b"secret secret secret!");
  ```

- [**RAID-parity functions**][raid] (requires feature `raid`)

  ``` rust
  use gf256::raid::raid7;

  // format
  let mut buf = b"Hello World!".to_vec();
  let mut parity1 = vec![0u8; 4];
  let mut parity2 = vec![0u8; 4];
  let mut parity3 = vec![0u8; 4];
  let slices = buf.chunks(4).collect::<Vec<_>>();
  raid7::format(&slices, &mut parity1, &mut parity2, &mut parity3);

  // corrupt
  buf[0..8].fill(b'x');

  // repair
  let mut slices = buf.chunks_mut(4).collect::<Vec<_>>();
  raid7::repair(&mut slices, &mut parity1, &mut parity2, &mut parity3, &[0, 1]);
  assert_eq!(&buf, b"Hello World!");
  ```

- [**Reed-Solomon error-correction functions**][rs] (requires feature `rs`)

  ``` rust
  use gf256::rs::rs255w223;

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

Since this math depends on some rather arbitrary constants, each of these
utilities is available as both a normal Rust API, defined using reasonable
defaults, and as a highly configurable [`proc_macro`][proc-macros]:

``` rust
# pub use ::gf256::*;
use gf256::gf::gf;

#[gf(polynomial=0x11b, generator=0x3)]
type gf256_rijndael;

# fn main() {
let a = gf256_rijndael(0xfd);
let b = gf256_rijndael(0xfe);
let c = gf256_rijndael(0xff);
assert_eq!(a*(b+c), a*b + a*c);
# }
```

## Hardware support

Most modern 64-bit hardware contains instructions for accelerating this sort
of math. This usually comes in the form of a [carry-less multiplication][xmul]
instruction.

Carry-less multiplication, also called polynomial multiplication and xor
multiplication, is the multiplication analog for xor. Where traditional
multiplication can be implemented as a series of shifts and adds, carry-less
multiplication can be implemented as a series of shifts and xors:

``` text
Multiplication:

1011 * 1101 =  1011
            +   1011
            +     1011
            ----------
            = 10001111

Carry-less multiplication:

1011 * 1101 =  1011
            ^   1011
            ^     1011
            ----------
            = 01111111
```

64-bit carry-less multiplication is available on x86_64 with the
[`pclmulqdq`][pclmulqdq], and on aarch64 with the slightly less wordy
[`pmull`][pmull] instruction.

gf256 takes advantage of these instructions when possible. However, at the time
of writing, `pmull` support in Rust is only available on [nightly][nightly].

``` rust
# use ::gf256::*;
#
// uses carry-less multiplication instructions if available
let a = p32(0b1011);
let b = p32(0b1101);
assert_eq!(a * b, p32(0b01111111));
```

gf256 also exposes the flag [`HAS_XMUL`], which can be used to choose
algorithms based on whether or not hardware accelerated carry-less
multiplication is available:

``` rust
# use gf256::p::p32;
#
let a = p32(0b1011);
let b = if gf256::HAS_XMUL {
    a * p32(0b11)
} else {
    (a << 1) ^ a
};
```

gf256 also leverages the hardware accelerated [carry-less addition][xor]
instructions, sometimes called polynomial addition, or simply xor. But this
is much less notable.

## `const fn` support

Due to the use of traits and intrinsics, it's not possible to use the
polynomial/Galois-field operators in [`const fns`][const-fn].

As an alternative, gf256 provides a set of "naive" functions, which
provide less efficient, well, naive, implementations that can be used
in const fns.

These are very useful for calculating complex constants at compile-time,
which is common in these sort of algorithms:

``` rust
# use ::gf256::*;
#
const POLYNOMIAL: p64 = p64(0x104c11db7);
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
```

## `no_std` support

gf256 is just a pile of math, so it is mostly [`no_std`][no-std] compatible.

The exceptions are the extra utilities `rs` and `shamir`, which
currently require `alloc`.

## Constant-time

gf256 provides "best-effort" constant-time implementations for certain
useful operations. Though it should be emphasized this was primarily an
educational project, so the constant-time properties should be externally
evaluated before use, and you use this library at your own risk.

- Polynomial multiplication

  Polynomial multiplication in gf256 should always be constant-time.

  The assumption is that any hardware accelerated carry-less multiplication
  instructions complete in a fixed number of cycles, which is generally true.

  If carry-less multiplication instructions are not available, a branch-less
  loop implementation of carry-less multiplication is used.

- Galois-field operations

  Galois-field types in `barret` mode rely only on carry-less multiplication
  and xors, and should always execute in constant time.

  The other Galois-field implementations are NOT constant-time due to the use
  of lookup tables, which may be susceptible to cache-timing attacks. Note that
  the default Galois-field types likely use a table-based implementation.

  You will need to declare a custom Galois-field type using `barret` mode if you
  want constant-time finite-field operations:

  ``` rust
  # pub use ::gf256::*;
  use gf256::gf::gf;

  #[gf(polynomial=0x11b, generator=0x3, barret)]
  type gf256_rijndael;
  #
  # fn main() {}
  ```

- Shamir secret-sharing

  The default Shamir secret-sharing implementation internally uses a custom
  Galois-field type in `barret` mode and should (keyword _should_) be
  constant-time.

## Features

- `no-xmul` - Disables carry-less multiplication instructions, forcing the use
  of naive bitwise implementations

  This is mostly available for testing/benchmarking purposes.

- `no-tables` - Disables lookup tables, relying only on hardware instructions
  or naive implementations

  This may be useful on memory constrained devices

- `small-tables` - Limits lookup tables to "small tables", tables with <=16
  elements

  This provides a compromise between full 256-byte tables and no-tables,
  which may be useful on memory constrained devices

- `thread-rng` - Enables features that depend on ThreadRng

  Note this requires `std`

  This is used to provide a default Rng implementation for Shamir's
  secret-sharing implementations

- `lfsr` - Makes LFSR structs and macros available

- `crc` - Makes CRC functions and macros available

- `shamir` - Makes Shamir secret-sharing functions and macros available

  Note this requires `alloc` and `rand`

  You may also want to enable the `thread-rng` feature, which is required for
  a default rng

- `raid` - Makes RAID-parity functions and macros available

- `rs` - Makes Reed-Solomon functions and macros available

  Note this requires `alloc`

## Testing

gf256 comes with a number of tests implemented in Rust's [test runner][test-runner],
these can be run with make:

``` bash
make test
```

Additionally all of the code samples in these docs can be ran with Rust's
[doctest runner][doctest-runner]:

``` bash
make docs
```

## Benchmarking

gf256 also has a number of benchmarks implemented in [Criterion][criterion].
These were used to determine the best default implementations, and can be ran
with make:

``` bash
make bench
```

A full summary of the benchmark results can be found in [BENCHMARKS.md][benchmarks].


[p]: https://docs.rs/gf256/latest/gf256/p
[gf]: https://docs.rs/gf256/latest/gf256/gf
[lfsr]: https://docs.rs/gf256/latest/gf256/lfsr
[crc]: https://docs.rs/gf256/latest/gf256/crc
[shamir]: https://docs.rs/gf256/latest/gf256/shamir
[raid]: https://docs.rs/gf256/latest/gf256/raid
[rs]: https://docs.rs/gf256/latest/gf256/rs
[finite-field]: https://en.wikipedia.org/wiki/Finite_field
[field-axioms]: https://en.wikipedia.org/wiki/Field_(mathematics)
[proc-macros]: https://doc.rust-lang.org/reference/procedural-macros.html
[xmul]: https://en.wikipedia.org/wiki/Carry-less_product
[xor]: https://en.wikipedia.org/wiki/Bitwise_operation#XOR
[pclmulqdq]: https://www.felixcloutier.com/x86/pclmulqdq
[pmull]: https://developer.arm.com/documentation/ddi0596/2021-06/SIMD-FP-Instructions/PMULL--PMULL2--Polynomial-Multiply-Long-
[nightly]: https://doc.rust-lang.org/book/appendix-07-nightly-rust.html
[no-std]: https://docs.rust-embedded.org/book/intro/no-std.html
[const-fn]: https://doc.rust-lang.org/reference/const_eval.html
[test-runner]: https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
[doctest-runner]: https://doc.rust-lang.org/rustdoc/documentation-tests.html
[criterion]: https://docs.rs/criterion/latest/criterion
[benchmarks]: https://github.com/geky/gf256/blob/master/BENCHMARKS.md

