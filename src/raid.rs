//! ## RAID-parity functions and macros
//!
//! [RAID](raid), short for a "Redundant Array of Independent Disks", is a set
//! of schemes commonly found in storage systems, with the purpose of using an
//! array of multiple disks to providing data redundancy and/or performance
//! improvements.
//!
//! The most interesting part for us is the higher-numbered RAID levels, which
//! use extra disks to store parity information, capable of reconstructing any
//! one (for RAID 5), two (for RAID 6), and even three (for RAID 7, though this
//! name is not standardized) failed disks.
//!
//! TODO example
//!
//! These parity-schemes aren't limited to physical disks. They can be applied
//! to any array of blocks as long as there is some mechanism to detect failures,
//! such as CRCs or other checksums.
//!
//! ## How does RAID-parity work?
//!
//! The simplest parity-scheme to understand is RAID 5, aka single-parity. In this
//! scheme the parity block, "p", is simply the xor of all of the data blocks:
//!
//! ``` text
//! p = d0 ^ d1 ^ d2 ^ ...
//! ```
//!
//! If any single data block goes bad, we can reconstruct its original value by
//! xoring the parity block with all of the other data blocks:
//!
//! ``` text
//! d1 = p ^ d0 ^ d2 ...
//! ```
//!
//! Here's some Rust code that shows this working:
//!
//! ``` rust
//! // given some data blocks
//! let mut data = vec![0u8; 10];
//! for i in 0..data.len() {
//!     data[i] = i as u8;
//! }
//!
//! // calculate parity block p
//! let mut p = 0;
//! for i in 0..data.len() {
//!    p ^= data[i];
//! }
//!
//! // oh no! a block is corrupted!
//! data[1] = 0xff;
//!
//! // reconstruct using our parity block p
//! let mut fixed = p;
//! for i in 0..data.len() {
//!     if i != 1 {
//!         fixed ^= data[i];
//!     }
//! }
//! data[1] = fixed;
//!
//! for i in 0..data.len() {
//!     assert_eq!(data[i], i as u8);
//! }
//! ```
//!
//! Let's think about what this means mathematically.
//!
//! xor is equivalent to addition in a finite-field, so, as long as we say we're
//! in a finite-field, we can rewrite the first equation like so:
//!
//! ``` text
//! p = d0 + d1 + d2 + ...
//!
//! Or:
//!
//! p = Σ di
//!     i
//! ```
//!
//! If a block goes bad, we've introduced an unknown into our equation:
//!
//! ``` text
//! p = d0 + ? + d2 + ...
//! ```
//!
//! But we can solve for it! Let's call the unknown "dx":
//!
//! ``` text
//! p = d0 + dx + d2 + ...
//!
//! dx = p - (d0 + d2 + ...)
//!
//! Or:
//!
//! dx = p - Σ di
//!         i!=x
//! ```
//!
//! And since subtraction is also xor in a finite-field, this is the equivalent 
//! to xoring the parity block and data blocks together.
//!
//! ---
//!
//! But what if two blocks go bad?
//!
//! ``` text
//! p = ? + ? + d2 + ...
//! ```
//!
//! If two blocks go bad, we have two unknowns, but only one equation. We just
//! can't find a unique solution.
//!
//! But we can if we have two equations! In RAID-parity schemes, we treat the blocks
//! as a linear system of equations. And if you recall grade-school algebra, you can
//! find a unique solution to a linear system of equations if you have more equations
//! than unknowns, and the equations are _linearly independent_ (this becomes
//! important later).
//!
//! Say, for our second parity block, "q", we came up with a different equation by
//! using some arbitrary constants, we'll call them "c0, c1, c2..." for now:
//!
//! ``` text
//! p = d0 + d1 + d2 + ...
//!
//! q = d0*c0 + d1*c1 + d2*c2 + ...
//!
//! Or:
//!
//! p = Σ di
//!     i
//!
//! q = Σ di*ci
//!     i
//! ```
//!
//! Now, if any two data blocks go bad, we have two unknowns:
//!
//! ``` text
//! p = ? + ? + d2 + ...
//!
//! q = ?*c0 + ?*c1 + d2*c2 + ...
//! ```
//!
//! But since we have two equations, we can still solve it! Let's call the
//! unknowns dx and dy:
//!
//! ``` text
//! p = dx + dy + d2 + ...
//!
//! q = dx*c0 + dy*c1 + d2*c2 + ...
//!
//! dx + dy = p - (d2 + ...)
//!
//! dx*c0 + dy*c1 = q - (d2*c2 + ...)
//!
//! Or:
//!
//! dx + dy = p - Σ di
//!             i!=x,y
//!
//! dx*cx + dy*cy = q - Σ di*ci
//!                   i!=x,y
//!
//! Solving for dy:
//!
//! dx + dy = p - Σ di
//!             i!=x,y
//! 
//! dy = p - Σ di - dx
//!        i!=x,y
//!
//! Substitute dy:
//!
//! dx*cx + (p - Σ di - dx)*cy = q - Σ di*ci
//!            i!=x,y              i!=x,y
//!
//! dx*cx + (p - Σ di)*cy - dx*cy = q - Σ di*ci
//!            i!=x,y                 i!=x,y
//!
//! dx*(cx - cy) + (p - Σ di)*cy = q - Σ di*ci
//!                   i!=x,y         i!=x,y
//!
//! dx*(cx - cy) = (q - Σ di*ci) - (p - Σ di)*cy
//!                   i!=x,y          i!=x,y
//!
//!      (q - Σ di*ci) - (p - Σ di)*cy
//!         i!=x,y          i!=x,y
//! dx = -----------------------------
//!                 cx - cy
//!
//! So:
//!
//!      (q - Σ di*ci) - (p - Σ di)*cy
//!         i!=x,y          i!=x,y
//! dx = -----------------------------
//!                 cx - cy
//!
//! dy = p - Σ di - dx
//!        i!=x,y
//! ```
//!
//! This gives us two "simple" (at least for a computer) equations to find
//! dx and dy.
//!
//! We can see that we can always find a solution as long as cx != cy, otherwise
//! we end up dividing by zero.
//!
//! We can use any set of unique constants for this, but it's convenient to use
//! powers of a generator in our field, "g". Recall that powers of a generator,
//! sometimes called a primitive element, generate all non-zero elements of a
//! given field before looping. So g^i is both non-zero and unique for any
//! i < the number of non-zero elements in the field, 255 for GF(256), which
//! means our equation will always be solvable as long as we don't have more
//! than 255 disks! TODO how do we find generators? something to link to here?
//!
//! We can substitute powers of our generator "g" back into our solutions to give
//! us our final equations:
//!
//! Given parity blocks p and q:
//!
//! ``` text
//! p = d0 + d1 + d2 + ...
//!
//! q = d0*g^0 + d1*g^1 + d2*g^2 + ...
//!
//! Or:
//!
//! p = Σ di
//!     i
//!
//! q = Σ di*g^i
//!     i
//! ```
//!
//! We can solve for any two bad blocks, dx and dy:
//!
//! ``` text
//! dx + dy = p - Σ di
//!             i!=x,y
//!
//! dx*g^x + dy*g^y = q - Σ di*g^i
//!                     i!=x,y
//!
//! Therefore:
//!
//!      (q - Σ di*g^i) - (p - Σ di)*g^y
//!         i!=x,y           i!=x,y
//! dx = -------------------------------
//!                 g^x - g^y
//!
//! dy = p - Σ di - dx
//!        i!=x,y
//! ```
//!
//! Let's see this in action!
//!
//! ``` rust
//! # use ::gf256::*;
//! # use ::gf256::traits::FromLossy;
//! #
//! // given some data blocks
//! let mut data = vec![gf256(0); 10];
//! for i in 0..data.len() {
//!     data[i] = gf256::from_lossy(i);
//! }
//!
//! // calculate parity blocks p and q
//! //
//! // p = Σ di
//! //     i
//! //
//! // q = Σ di*g^i
//! //     i
//! //
//! let mut p = gf256(0);
//! let mut q = gf256(0);
//! for i in 0..data.len() {
//!     p += data[i];
//!     q += data[i]*gf256::GENERATOR.pow(i as u8);
//! }
//!
//! // oh no! TWO blocks are corrupted!
//! data[1] = gf256(0xff);
//! data[2] = gf256(0xff);
//!
//! // reconstruct using our parity blocks p and q
//! //
//! //      (q - Σ di*g^i) - (p - Σ di)*g^y
//! //         i!=x,y           i!=x,y
//! // dx = -------------------------------
//! //                 g^x - g^y
//! //
//! // dy = p - Σ di - dx
//! //        i!=x,y
//! //
//! let mut pdelta = p;
//! let mut qdelta = q;
//! for i in 0..data.len() {
//!     if i != 1 && i != 2 {
//!         pdelta -= data[i];
//!         qdelta -= data[i]*gf256::GENERATOR.pow(i as u8);
//!     }
//! }
//! let gx = gf256::GENERATOR.pow(1);
//! let gy = gf256::GENERATOR.pow(2);
//! data[1] = (qdelta - pdelta*gy) / (gx - gy);
//! data[2] = pdelta - data[1];
//!
//! for i in 0..data.len() {
//!     assert_eq!(data[i], gf256::from_lossy(i));
//! }
//! ```
//!
//! ---
//!
//! Can we push this further?
//!
//! We can kind of fudge our original equations so they're in a similar form:
//!
//! ``` text
//! p = Σ di*g^0i      dx*g^0x + dy*g^0y = p - Σ di*g^0i
//!     i                                    i!=x,y
//!                =>  
//! q = Σ di*g^1i      dx*g^1x + dy*g^1y = q - Σ di*g^1i
//!     i                                    i!=x,y    
//! ```
//!
//! Which raises an interesting question, can we keep going?
//!
//! ``` text
//! p = Σ di*g^0i      dx*g^0x + dy*g^0y + dz*g^0z + dw*g^0w + ... = p - Σ di*g^0i
//!     i                                                            i!=x,y,z,w,...
//!                
//! q = Σ di*g^1i      dx*g^1x + dy*g^1y + dz*g^1z + dw*g^1w + ... = q - Σ di*g^1i
//!     i                                                            i!=x,y,z,w,...
//!                =>  
//! r = Σ di*g^2i      dx*g^2x + dy*g^2y + dz*g^2z + dw*g^2w + ... = r - Σ di*g^2i
//!     i                                                            i!=x,y,z,w,...
//!
//! s = Σ di*g^3i      dx*g^3x + dy*g^3y + dz*g^3z + dw*g^3w + ... = s - Σ di*g^3i
//!     i                                                            i!=x,y,z,w,...
//!
//! ...                ...
//! ```
//!
//! Perhaps surprisingly, it turns out the answer is no! Kinda. At least not
//! generally.
//!
//! Rather than trying to reason about this big mess of equations, let us just
//! consider pairs of equations. We at least need to always be able to solve
//! these if the rest of the parity blocks go bad:
//!
//! ``` text
//! dx*g^(j*x) + dy*g^(j*y) = q - Σ di*g^(j*i)
//!                             i!=x,y
//! 
//! dx*g^(k*x) + dy*g^(k*y) = r - Σ di*g^(k*i)
//!                             i!=x,y    
//! ```
//!
//! We can solve a system of linear equations if they are "linearly independent",
//! that is, no equation is a scalar-multiple of another linear equation.
//!
//! Or an more mathy terms, is there a constant c such that c*f(...) = g(...)?
//!
//! What's interesting is we can actually allow one coefficient to be a multiple,
//! but not both. So we can phrase the question a bit differently: Is there a
//! unique c such that c*g^(j*x) = g^(k*x), assuming j!=k and x < 255?
//!
//! We can reduce this a bit:
//!
//! ``` text
//! c*g^(j*x) = g^(k*x)
//!
//! c = g^(k*x - j*x)
//!
//! c = g^((k-j)*x)
//! ```
//! And since our constant is arbitrary, we can substitute it for, say, the
//! log base g of c.
//!
//! Except this gets a bit tricky, recall that the powers of g form a multiplicative
//! cycle equal to the number of non-zero elements in our field, 255 for GF(256).
//! So when we take the logarithm, we actually end up with an equation under mod 255,
//! because the powers of g loops.
//!
//! We're actually dealing with two number systems here, our finite-field and the
//! infinite integers. There's probably a better way to notate this mathematically:
//!
//! ``` text
//! c = g^((k-j)*x)
//!
//! log_g(c) = log_g(g^((k-j)*x))
//!
//! c' = (k-j)*x mod 255
//! ```
//!
//! So! Is (k-j)*x mod 255 unique for any k!=j, x < 255?
//!
//! Unfortunately, while x is less than 255, (k-j)*x may not be.
//!
//! But there's a fun property of modular multiplication we can leverage. It turns
//! out modular multiplication by a constant will iterate all elements of the group
//! if the constant and the modulo are coprime.
//!
//! For example, say we were dealing with mod 9. 3, which is not coprime with 9,
//! gets stuck in a smaller loop:
//!
//! ``` rust
//! assert_eq!((3*1) % 9,  3);
//! assert_eq!((3*2) % 9,  6);
//! assert_eq!((3*3) % 9,  0);
//! assert_eq!((3*4) % 9,  3);
//! assert_eq!((3*5) % 9,  6);
//! assert_eq!((3*6) % 9,  0);
//! assert_eq!((3*7) % 9,  3);
//! assert_eq!((3*8) % 9,  6);
//! assert_eq!((3*9) % 9,  0);
//! assert_eq!((3*10) % 9, 3);
//! // ...
//! ```
//!
//! But 7, which is coprime with 9, iterates through all elements before looping:
//!
//! ``` rust
//! assert_eq!((7*1) % 9,  7);
//! assert_eq!((7*2) % 9,  5);
//! assert_eq!((7*3) % 9,  3);
//! assert_eq!((7*4) % 9,  1);
//! assert_eq!((7*5) % 9,  8);
//! assert_eq!((7*6) % 9,  6);
//! assert_eq!((7*7) % 9,  4);
//! assert_eq!((7*8) % 9,  2);
//! assert_eq!((7*9) % 9,  0);
//! assert_eq!((7*10) % 9, 7);
//! // ...
//! ```
//!
//! So (k-j)*x mod 255 actually _is_ unique for any x < 255, as long as k!=j
//! and k-j is coprime with 255.
//!
//! So there you go!
//!
//! We can extend our RAID-parity scheme to any number of parity blocks as long
//! as we have a set of unique constants where for ANY of the two constants,
//! k and j, k-j is coprime with 255.
//!
//! So how many parity blocks does that give us?
//!
//! Three.
//!
//! That's right, only three. It turns out the above constraint is actually quite
//! limiting.
//!
//! Note that 255, while mostly prime, is not actually prime. And, as Miracle Max
//! would say, mostly prime just means slightly composite:
//!
//! ``` text
//! 255 = 3 * 5 * 17
//! ```
//!
//! So if any k-j is a multiple of 3, it fails to be coprime with 255.
//!
//! We can choose some constants:
//!
//! ``` text
//! j = 3n+0 for any n
//! k = 3n+1 for any n
//! l = 3n+2 for any n
//! ```
//!
//! But the moment try to choose some constant m = 3n+3, well, m-j = 3n, some
//! multiple of 3, which can't be coprime with 255.
//!
//! And this is true for ANY word-sized field. Any word-sized field GF(2^8),
//! GF(2^16), GF(2^32), GF(2^(2^i)) has a multiplicative cycle of
//! length 2^(2^i)-1 ([A051179][A051179]), which is always divisible by 3.
//!
//! The good news is that this does gives us a set of three valid constants,
//! which give us three linearly independent generators:
//!
//! ``` text
//! g^(3n+0) for any n
//! g^(3n+1) for any n
//! g^(3n+2) for any n
//! ```
//!
//! And we can just set n=0 for all three constants:
//!
//! ``` text
//! g^0 = 1
//! g^1 = g
//! g^2 = g^2
//! ```
//!
//! It's worth noting that g^2 is also a generator of the field. For similar
//! reasons to modular multiplication, for any generator g, g^k is also a
//! generator, iff k is coprime with the size of the multiplicative cycle. And
//! since any 2^i-1 (not divisible by 2) is always coprime with 2^j (only
//! prime factor is 2), g^(2^k) is also a generator for any k is also a
//! generator.
//!
//! This isn't actually useful here, but it's interesting to know!
//!
//! ---
//!
//! With this, we can construct a triple-parity scheme!
//!
//! We can create three parity blocks, "p", "q", "r", with three linearly
//! independent equations, by using two generators, "g" and "h", where h = g^2:
//!
//! ``` text
//! p = d0 + d1 + d2 + ...
//!
//! q = d0*g^0 + d1*g^1 + d2*g^2 + ...
//!
//! r = d0*h^0 + d1*h^1 + d2*h^2 + ...
//!
//! Or:
//!
//! p = Σ di
//!     i
//!
//! q = Σ di*g^i
//!     i
//!
//! r = Σ di*h^i
//!     i
//! ```
//!
//! If three blocks go bad, we have three equations and three unknowns, "dx",
//! "dy", "dz":
//!
//! ``` text
//! dx + dy + dz = p - Σ di
//!                 i!=x,y,z
//!
//! dx*g^x + dy*g^y + dz*g^z = q - Σ di*g^i
//!                             i!=x,y,z
//!
//! dx*h^x + dy*h^y + dz*h^z = r - Σ di*h^i
//!                             i!=x,y,z
//! ```
//!
//! Ready for some big equations?
//!
//! ``` text
//! Solving for dz:
//!
//! dx + dy + dz = p - Σ di
//!                 i!=x,y,z
//!
//! dz = p - Σ di - dx - dy
//!       i!=x,y,z
//!
//! Substitute dz:
//!
//! dx*g^x + dy*g^y + (p - Σ di - dx - dy)*g^z = q - Σ di*g^i
//!                     i!=x,y,z                  i!=x,y,z
//!
//! dx*g^x + dy*g^y + (p - Σ di)*g^z - dx*g^z - dy*g^z = q - Σ di*g^i
//!                     i!=x,y,z                          i!=x,y,z
//!
//! dx*(g^x - g^z) + dy*(g^y - g^z) = (q - Σ di*g^i) - (p - Σ di)*g^z
//!                                     i!=x,y,z         i!=x,y,z
//!
//! dy*(g^y - g^z) = (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
//!                    i!=x,y,z         i!=x,y,z
//!
//!      (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
//!        i!=x,y,z         i!=x,y,z
//! dy = ------------------------------------------------
//!                         g^y - g^z
//!
//! Substitute dy:
//!
//!                  ( (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z) )
//!                  (   i!=x,y,z         i!=x,y,z                      )
//! dx*(h^x - h^z) + ( ------------------------------------------------ )*(h^y - h^z) + (p - Σ di)*h^z = r - Σ di*h^i
//!                  (                    g^y - g^z                     )                 i!=x,y,z        i!=x,y,z
//!
//!
//!                  ( (q - Σ di*g^i) - (p - Σ di)*g^z )
//!                  (   i!=x,y,z         i!=x,y,z     )               ( dx*(g^x - g^z) )
//! dx*(h^x - h^z) + ( ------------------------------- )*(h^y - h^z) - ( -------------- )*(h^y - h^z) + (p - Σ di)*h^z = r - Σ di*h^i
//!                  (             g^y - g^z           )               (   g^y - g^z    )                 i!=x,y,z        i!=x,y,z
//!
//!                                                                                     ( (q - Σ di*g^i) - (p - Σ di)*g^z )
//!                  ( dx*(g^x - g^z) )                                                 (   i!=x,y,z         i!=x,y,z     )
//! dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = (r - Σ di*h^i) - (p - Σ di)*h^z - ( ------------------------------- )*(h^y - h^z)
//!                  (   g^y - g^z    )                 i!=x,y,z         i!=x,y,z       (             g^y - g^z           )
//!
//!                                                   (r - Σ di*h^i)*(g^y - g^z) - (p - Σ di)*h^z*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) + (p - Σ di)*g^z*(h^y - h^z)
//!                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//! dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = -----------------------------------------------------------------------------------------------------------------
//!                  (   g^y - g^z    )                                                              g^y - g^z
//!
//!                                                   (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(h^z*(g^y - g^z) - g^z*(h^y - h^z))
//!                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//! dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = --------------------------------------------------------------------------------------------------------
//!                  (   g^y - g^z    )                                                              g^y - g^z
//!
//!                                                   (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^z - g^z*h^y + g^z*h^z)
//!                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//! dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = ------------------------------------------------------------------------------------------------------------
//!                  (   g^y - g^z    )                                                              g^y - g^z
//!
//!                                                   (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//!                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//! dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = ----------------------------------------------------------------------------------------
//!                  (   g^y - g^z    )                                                         g^y - g^z
//!
//!                                                           (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//! dx*(h^x - h^z)*(g^y - g^z) - dx*(g^x - g^z)*(h^y - h^z)     i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//! ------------------------------------------------------- = ----------------------------------------------------------------------------------------
//!                        g^y - g^z                                                                    g^y - g^z
//!
//! dx*(h^x - h^z)*(g^y - g^z) - dx*(g^x - g^z)*(h^y - h^z) = (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//!                                                             i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//!
//! dx*((h^x - h^z)*(g^y - g^z) - (h^y - h^z)*(g^x - g^z)) = (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//!                                                            i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//!
//!      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//!        i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//! dx = ----------------------------------------------------------------------------------------
//!                       (h^x - h^z)*(g^y - g^z) - (h^y - h^z)*(g^x - g^z)
//!
//! So:
//!
//!      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//!        i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//! dx = ----------------------------------------------------------------------------------------
//!                       (h^x - h^z)*(g^y - g^z) - (h^y - h^z)*(g^x - g^z)
//!
//!      (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
//!        i!=x,y,z         i!=x,y,z
//! dy = ------------------------------------------------
//!                         g^y - g^z
//!
//! dz = p - Σ di - dx - dy
//!       i!=x,y,z
//! ```
//!
//! If we use the property that h = g^2, we can simplify a little bit further:
//!
//! ``` text
//!      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//!        i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//! dx = ----------------------------------------------------------------------------------------
//!                       (h^x - h^z)*(g^y - g^z) - (h^y - h^z)*(g^x - g^z)
//!
//!
//!      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(g^2y - g^2z) - (p - Σ di)*(g^y*g^2z - g^z*g^2y)
//!         i!=x,y,z                     i!=x,y,z                       i!=x,y,z
//! dx = --------------------------------------------------------------------------------------------
//!                         (g^2x - g^2z)*(g^y - g^z) - (g^2y - g^2z)*(g^x - g^z)
//!
//!      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(g^2y - g^2z) - (p - Σ di)*(g^y*g^2z - g^z*g^2y)
//!         i!=x,y,z                     i!=x,y,z                       i!=x,y,z
//! dx = --------------------------------------------------------------------------------------------
//!                               (g^x - g^y)*(g^x - g^z)*(g^y - g^z)
//!
//!      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(g^y - g^z)^2 - (p - Σ di)*g^y*g^z*(g^z - g^y)
//!         i!=x,y,z                     i!=x,y,z                       i!=x,y,z
//! dx = --------------------------------------------------------------------------------------------
//!                               (g^x - g^y)*(g^x - g^z)*(g^y - g^z)
//!
//!      (r - Σ di*h^i) - (q - Σ di*g^i)*(g^y - g^z) - (p - Σ di)*g^y*g^z
//!         i!=x,y,z         i!=x,y,z                     i!=x,y,z
//! dx = ----------------------------------------------------------------
//!                      (g^x - g^y)*(g^x - g^z)
//!
//! So:
//!
//!      (r - Σ di*h^i) - (q - Σ di*g^i)*(g^y - g^z) - (p - Σ di)*g^y*g^z
//!         i!=x,y,z         i!=x,y,z                     i!=x,y,z
//! dx = ----------------------------------------------------------------
//!                      (g^x - g^y)*(g^x - g^z)
//!
//!      (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
//!        i!=x,y,z         i!=x,y,z
//! dy = ------------------------------------------------
//!                         g^y - g^z
//!
//! dz = p - Σ di - dx - dy
//!       i!=x,y,z
//! ```
//!
//! So, to summarize, given parity blocks p, q, r:
//!
//! ``` text
//! p = d0 + d1 + d2 + ...
//!
//! q = d0*g^0 + d1*g^1 + d2*g^2 + ...
//!
//! r = d0*h^0 + d1*h^1 + d2*h^2 + ...
//!
//! Or:
//!
//! p = Σ di
//!     i
//!
//! q = Σ di*g^i
//!     i
//!
//! r = Σ di*h^i
//!     i
//! ```
//!
//! We can solve for any three bad blocks, dx, dy, dz:
//!
//! ``` text
//! dx + dy + dz = p - Σ di
//!                 i!=x,y,z
//!
//! dx*g^x + dy*g^y + dz*g^z = q - Σ di*g^i
//!                             i!=x,y,z
//!
//! dx*h^x + dy*h^y + dz*h^z = r - Σ di*h^i
//!                             i!=x,y,z
//!
//! Therefore:
//!
//!      (r - Σ di*h^i) - (q - Σ di*g^i)*(g^y - g^z) - (p - Σ di)*g^y*g^z
//!         i!=x,y,z         i!=x,y,z                     i!=x,y,z
//! dx = ----------------------------------------------------------------
//!                      (g^x - g^y)*(g^x - g^z)
//!
//!      (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
//!        i!=x,y,z         i!=x,y,z
//! dy = ------------------------------------------------
//!                         g^y - g^z
//!
//! dz = p - Σ di - dx - dy
//!       i!=x,y,z
//! ```
//!
//! And the moment you've all been waiting for, lets see this in action:
//!
//! ``` rust
//! # use ::gf256::*;
//! # use ::gf256::traits::FromLossy;
//! #
//! // given some data blocks
//! let mut data = vec![gf256(0); 10];
//! for i in 0..data.len() {
//!     data[i] = gf256::from_lossy(i);
//! }
//!
//! // calculate parity blocks p, q, r
//! //
//! // p = Σ di
//! //     i
//! //
//! // q = Σ di*g^i
//! //     i
//! //
//! // r = Σ di*h^i
//! //     i
//! //
//! let mut p = gf256(0);
//! let mut q = gf256(0);
//! let mut r = gf256(0);
//! for i in 0..data.len() {
//!     let g = gf256::GENERATOR.pow(i as u8);
//!     let h = g*g;
//!     p += data[i];
//!     q += data[i]*g;
//!     r += data[i]*h;
//! }
//!
//! // oh no! THREE blocks are corrupted!
//! data[1] = gf256(0xff);
//! data[2] = gf256(0xff);
//! data[3] = gf256(0xff);
//!
//! // reconstruct using our parity blocks p and q
//! //
//! //      (r - Σ di*h^i) - (q - Σ di*g^i)*(g^y - g^z) - (p - Σ di)*g^y*g^z
//! //         i!=x,y,z         i!=x,y,z                     i!=x,y,z
//! // dx = ----------------------------------------------------------------
//! //                      (g^x - g^y)*(g^x - g^z)
//! //
//! //      (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
//! //        i!=x,y,z         i!=x,y,z
//! // dy = ------------------------------------------------
//! //                         g^y - g^z
//! //
//! // dz = p - Σ di - dx - dy
//! //       i!=x,y,z
//! //
//! let mut pdelta = p;
//! let mut qdelta = q;
//! let mut rdelta = r;
//! for i in 0..data.len() {
//!     if i != 1 && i != 2 && i != 3 {
//!         let g = gf256::GENERATOR.pow(i as u8);
//!         let h = g*g;
//!         pdelta -= data[i];
//!         qdelta -= data[i]*g;
//!         rdelta -= data[i]*h;
//!     }
//! }
//! let gx = gf256::GENERATOR.pow(1);
//! let gy = gf256::GENERATOR.pow(2);
//! let gz = gf256::GENERATOR.pow(3);
//! data[1] = (rdelta - qdelta*(gy - gz) - pdelta*gy*gz) / ((gx - gy) * (gx - gz));
//! data[2] = (qdelta - pdelta*gz - data[1]*(gx - gz)) / (gy - gz);
//! data[3] = pdelta - data[1] - data[2];
//!
//! for i in 0..data.len() {
//!     assert_eq!(data[i], gf256::from_lossy(i));
//! }
//! ```
//! 
//! And those are RAID-parity schemes all the way up to RAID 7, aka triple-parity!
//!
//! There are still a number of steps not covered here. It's always possible
//! for your parity blocks themselves to fail, in which case you just need
//! to rebuild your parity blocks using the data blocks.
//!
//! It gets more tricky if both some parity blocks and some data blocks fail,
//! in which case you need to reconstruct the data blocks with whatever parity
//! blocks are left, and then rebuilding the missing parity blocks. This is
//! equivalent to solving a smaller RAID-parity scheme.
//!
//! ## Limitations
//!
//! RAID 5, aka single-parity, is actually the most flexible. It can support
//! any number of blocks, though it can only repair a single block failure.
//!
//! RAID 6 and RAID 7, aka double-parity and triple-parity, rely on the uniqueness
//! of powers of a generator in the field. Because of this, these schemes are
//! limited to the number of non-zero elements in the field. In the case of GF(256),
//! this limits RAID 6 and RAID 7 to 255 blocks.
//!
//! Each scheme can repair any block up to the number of parity blocks, however
//! they don't actually provide the detection of block failures. One way to do this
//! is attach a CRC or other checksum to each block.
//!
//! ## RAID8? >3 parity blocks?
//!
//! As it is, the current scheme only supports up to 3 parity blocks. But it is
//! actually possible to use a different scheme that works beyond 3 parity blocks.
//!
//! As outlined in James S. Plank’s paper [Note: Correction to the 1997 Tutorial
//! on Reed-Solomon Coding][plank], you can construct a modified Vandermonde matrix
//! that allows you to solve the linear system of equations for any number of parit
//! blocks.
//!
//! The downside Plank's approach is that at minimum you do need to store an array
//! of relatively arbitrary constants for each parity block.
//!
//! 
//!
//!
//! TODO cite correctly everywhere?
//! [plank]: https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.7.155&rep=rep1&type=pdf

//!
//! 


//!
//! TODO limitations
//! TODO note on more parity blocks
//! TODO note on optimizations (successive powers, moving gh consts out)
//! TODO efficient updates?
//! 
//! 

// Solve:
//
// dz = p - Σ di - dx - dy
//       i!=x,y,z
//
// Subst dz:
//
// dx*g^x + dy*g^y + (p - Σ di - dx - dy)*g^z = q - Σ di*g^i
//                     i!=x,y,z                  i!=x,y,z
//
// dx*g^x + dy*g^y + (p - Σ di)*g^z - dx*g^z - dy*g^z = q - Σ di*g^i
//                     i!=x,y,z                          i!=x,y,z
//
// dx*(g^x - g^z) + dy*(g^y - g^z) = (q - Σ di*g^i) - (p - Σ di)*g^z
//                                     i!=x,y,z         i!=x,y,z
//
// dy*(g^y - g^z) = (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
//                    i!=x,y,z         i!=x,y,z
//
//      (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
//        i!=x,y,z         i!=x,y,z
// dy = ------------------------------------------------
//                         g^y - g^z
//
// Subst dz:
//
// dx*h^x + dy*h^y + (p - Σ di - dx - dy)*h^z = r - Σ di*h^i
//                     i!=x,y,z                  i!=x,y,z
//
// dx*h^x + dy*h^y + (p - Σ di)*h^z - dx*h^z - dy*h^z = r - Σ di*h^i
//                     i!=x,y,z                          i!=x,y,z
//
// dx*(h^x - h^z) + dy*(h^y - h^z) + (p - Σ di)*h^z = r - Σ di*h^i
//                                     i!=x,y,z        i!=x,y,z
//
// Subst dy:
//
//                  ( (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z) )
//                  (   i!=x,y,z         i!=x,y,z                      )
// dx*(h^x - h^z) + ( ------------------------------------------------ )*(h^y - h^z) + (p - Σ di)*h^z = r - Σ di*h^i
//                  (                    g^y - g^z                     )                 i!=x,y,z        i!=x,y,z
//
//
//                  ( (q - Σ di*g^i) - (p - Σ di)*g^z )
//                  (   i!=x,y,z         i!=x,y,z     )               ( dx*(g^x - g^z) )
// dx*(h^x - h^z) + ( ------------------------------- )*(h^y - h^z) - ( -------------- )*(h^y - h^z) + (p - Σ di)*h^z = r - Σ di*h^i
//                  (             g^y - g^z           )               (   g^y - g^z    )                 i!=x,y,z        i!=x,y,z
//
//                                                                                     ( (q - Σ di*g^i) - (p - Σ di)*g^z )
//                  ( dx*(g^x - g^z) )                                                 (   i!=x,y,z         i!=x,y,z     )
// dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = (r - Σ di*h^i) - (p - Σ di)*h^z - ( ------------------------------- )*(h^y - h^z)
//                  (   g^y - g^z    )                 i!=x,y,z         i!=x,y,z       (             g^y - g^z           )
//
//                                                   (r - Σ di*h^i)*(g^y - g^z) - (p - Σ di)*h^z*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) + (p - Σ di)*g^z*(h^y - h^z)
//                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = -----------------------------------------------------------------------------------------------------------------
//                  (   g^y - g^z    )                                                              g^y - g^z
//
//                                                   (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(h^z*(g^y - g^z) - g^z*(h^y - h^z))
//                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = --------------------------------------------------------------------------------------------------------
//                  (   g^y - g^z    )                                                              g^y - g^z
//
//                                                   (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^z - g^z*h^y + g^z*h^z)
//                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = ------------------------------------------------------------------------------------------------------------
//                  (   g^y - g^z    )                                                              g^y - g^z
//
//                                                   (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = ----------------------------------------------------------------------------------------
//                  (   g^y - g^z    )                                                         g^y - g^z
//
//                                                           (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
// dx*(h^x - h^z)*(g^y - g^z) - dx*(g^x - g^z)*(h^y - h^z)     i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// ------------------------------------------------------- = ----------------------------------------------------------------------------------------
//                        g^y - g^z                                                                    g^y - g^z
//
// dx*(h^x - h^z)*(g^y - g^z) - dx*(g^x - g^z)*(h^y - h^z) = (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//                                                             i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//
// dx*((h^x - h^z)*(g^y - g^z) - (h^y - h^z)*(g^x - g^z)) = (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//                                                            i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//
//      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//        i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx = ----------------------------------------------------------------------------------------
//                       (h^x - h^z)*(g^y - g^z) - (h^y - h^z)*(g^x - g^z)
//
//










//! 
//!
//!
//!
//! Let there be three parity blocks, "p", "q",  and "r", each defined by
//! by a linearly independent equation using the constants we found above:
//!
//!
//!
//! Say, for our second parity block, "q", we came up with a different equation by
//! using some arbitrary constants, we'll call them "c0, c1, c2..." for now:
//!
//! ``` text
//! p = d0 + d1 + d2 + ...
//!
//! q = d0*c0 + d1*c1 + d2*c2 + ...
//!
//! Or:
//!
//! p = Σ di
//!     i
//!
//! q = Σ di*ci
//!     i
//! ```
//! 
//! ## Limitations
//!
//! 
//! 
//! TODO limitations
//! TODO note on more parity blocks
//! TODO note on optimizations (successive powers, moving gh consts out)
//! TODO efficient updates?









//!
//! But it turns out modular multiplication has this
//! fun property that multiplication by a constant will traverse all non-zero elements
//! if the constant and the modulo are coprime. 
//!
//!
//! 
//!
//! , because we have two variables. So we can actually rephrase this
//! question: Does c*g^(j*x) = g^(k*x) for a unique c, where j != k, for any x < 255
//!
//! Ah! flip these!
//!
//!   Say, worst case, y=0. Suddenly the whole right-side of the equations becomes
//! one big constant.
//!
//! This leaves us with the x-terms, which must not be multiples of each other.
//! Otherwise the system collapses and we can't solve for a unique solution any
//! more.
//!
//! So now the question is, is there a constant c such that: TODO wait no, how to phrase?
//!
//! ``` text
//! c*dx*g^(j*x) = dx*g^(k*x) when j != k
//! ```
//!
//! We can reduce this a bit:
//!
//! ``` text
//! c*dx*g^(j*x) = dx*g^(k*x) when j != k
//!
//! c*g^(j*x) = g^(k*x) when j != k
//!
//! c = g^(k*x - j*x) when j != k
//!
//! c = g^((k-j)*x) when j != k
//! ```
//!
//! And since our constant is arbitrary, we can substitute it for, say, the
//! log base g of c.
//!
//! Except this gets a bit tricky, recall that the powers of g form a multiplicative
//! cycle equal to the number of non-zero elements in our field, 255 for GF(256).
//! So when we take the logarithm, we actually end up with an equation that's true
//! under mod 255, because the powers of g loops.
//!
//! We're actually dealing with two number systems here, our finite-field and the
//! infinite integers. There's probably a better way to notate this mathematically
//! but I'm not sure what it is:
//!
//! ``` text
//! c = g^((k-j)*x) when j != k
//!
//! log_g(c) = (k-j)*x mod 255 when j != k
//! ```
//!
//!
//!
//! We can also note that we can find a solution as long as cx != cy, otherwise
//! we end up dividing by zero, which suggests the original equations were not
//! linearly independent.
//!
//! TODO rewrite this?:
//! We can choose any set of unique constants, but usually we use powers of a
//! generator in our field "g". Recall that generators, also called primitive
//! elements, by definition have a multiplicative cycle containing all non-zero
//! elements of our field.  Or in other words, the powers of g will iterate
//! through non-zero numbers before looping.
//!
//! TODO triple parity?
//! 
//!
//! ``` rust ignore
//! use ::gf256::*;
//! println!("hm {}", gf256(0xfe).pow(1));
//! println!("hm {}", gf256(0xfe).pow(2));
//! println!("hm {}", gf256(0xfe).pow(4));
//! println!("hm {}", gf256(0xfe).pow(8));
//! println!("hm {}", gf256(0xfe).pow(16));
//! println!("hm {}", gf256(0xfe).pow(32));
//! println!("hm {}", gf256(0xfe).pow(64));
//! println!("hm {}", gf256(0xfe).pow(128));
//! println!("hm {}", gf256(0xfe).pow((256 % 255) as u8));
//! println!("hm {}", gf256(0xfe).pow((512 % 255) as u8));
//! println!("hm {}", gf256(0xfe).pow((1024 % 255) as u8));
//! println!("hm {}", gf256(0xfe).pow((2048 % 255) as u8));
//! println!("hm {}", gf256(0xfe).pow((4096 % 255) as u8));
//! println!("hm {}", gf256(0xfe).pow(128)*gf256(0xfe).pow(128));
//! assert!(false);
//! ```
//!
//!
//! How far can we push this?
//!
//! 
//! dx*g^0x + dy*g^0y + dz*g^0z + ... = p - Σ di*g^0i
//!                                     i!=x,y,z,...
//!
//! dx*g^1x + dy*g^2y + dz*g^1z + ... = q - Σ di*g^1i
//!                                     i!=x,y,z,...
//!
//! dx*g^2x + dy*g^2y + dz*g^2z + ... = r - Σ di*g^2i
//!                                     i!=x,y,z,...
//!
//! ...
//!
//!
//! [ g^0x g^0y g^0z ... | p - Σ di*g^0i ]
//! [ g^1x g^1y g^1z ... | q - Σ di*g^1i ]
//! [ g^2x g^2y g^2z ... | r - Σ di*g^2i ]
//! [ ...                | ...           ]
//!
//!
//!
//! 
//!
//! dx*g^0x + dy*g^0y + dz*g^0z + dw*g^0w = p - Σ di*g^0i
//!                                          i!=x,y,z,w
//!
//! dx*g^1x + dy*g^1y + dz*g^1z + dw*g^1w = q - Σ di*g^1i
//!                                          i!=x,y,z,w
//!
//! dx*g^2x + dy*g^2y + dz*g^2z + dw*g^2w = r - Σ di*g^2i
//!                                          i!=x,y,z,w
//!
//! dx*g^4x + dy*g^4y + dz*g^4z + dw*g^4w = s - Σ di*g^4i
//!                                          i!=x,y,z,w
//!
//! Does there exist any i where g^i = g^4i?
//! 
//!
//!
//!
//!
//! 
//! 
//!
//! TODO limitations
//! TODO note on more parity blocks
//! TODO note on optimizations (successive powers, moving gh consts out)
//! TODO efficient updates?
//!
//! ## RAID8?
//!
//! TODO quadruple parity using hardcoded matrices
//! 
//! ## Limitations
//!
//! 
//!





// TODO document this
// TODO and implement triple parity?
//
// Single parity:
//
// p = d0 + d1 + d2 + ... = Σ di
//                          i
//
// For a missing block dx:
//
// dx = p - Σ di
//         i!=x
//
//
// Double parity:
//
// p = d0 + d1 + d2 + ... = Σ di
//                          i
//
// q = d0*g^0 + d1*g^1 + d2*g^2 = Σ di*g^i
//                                i
//
// For two missing blocks dx and dy:
//
// dx + dy = p - Σ di
//             i!=x,y
//
// dx*g^x + dy*g^y = q - Σ di*g^i
//                     i!=x,y
//
// Solve:
//
// dy = p - Σ di - dx
//        i!=x,y
//
// Subst dy:
//
// dx*g^x + (p - Σ di - dx)*g^y = q - Σ di*g^i
//             i!=x,y               i!=x,y
//
// dx*g^x + (p - Σ di)*g^y - dx*g^y = q - Σ di*g^i
//             i!=x,y                  i!=x,y
//
// dx*(g^x - g^y) + (p - Σ di)*g^y = q - Σ di*g^i
//                     i!=x,y          i!=x,y
//
// dx*(g^x - g^y) = (q - Σ di*g^i) - (p - Σ di)*g^y
//                     i!=x,y           i!=x,y
//
//      (q - Σ di*g^i) - (p - Σ di)*g^y
//         i!=x,y           i!=x,y
// dx = -------------------------------
//                g^x - g^y
//
//
// Triple parity:
//
// p = d0 + d1 + d2 + ... = Σ di
//                          i
//
// q = d0*g^0 + d1*g^1 + d2*g^2 = Σ di*g^i
//                                i
//
// r = d0*h^0 + d1*h^1 + d2*h^2 = Σ di*h^i
//                                i
//
// For three missing blocks dx, dy, and dz:
//
// dx + dy + dz = p - Σ di
//                 i!=x,y,z
//
// dx*g^x + dy*g^y + dz*g^z = q - Σ di*g^i
//                             i!=x,y,z
//
// dx*h^x + dy*h^y + dz*h^z = r - Σ di*h^i
//                             i!=x,y,z
//
// Solve:
//
// dz = p - Σ di - dx - dy
//       i!=x,y,z
//
// Subst dz:
//
// dx*g^x + dy*g^y + (p - Σ di - dx - dy)*g^z = q - Σ di*g^i
//                     i!=x,y,z                  i!=x,y,z
//
// dx*g^x + dy*g^y + (p - Σ di)*g^z - dx*g^z - dy*g^z = q - Σ di*g^i
//                     i!=x,y,z                          i!=x,y,z
//
// dx*(g^x - g^z) + dy*(g^y - g^z) = (q - Σ di*g^i) - (p - Σ di)*g^z
//                                     i!=x,y,z         i!=x,y,z
//
// dy*(g^y - g^z) = (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
//                    i!=x,y,z         i!=x,y,z
//
//      (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z)
//        i!=x,y,z         i!=x,y,z
// dy = ------------------------------------------------
//                         g^y - g^z
//
// Subst dz:
//
// dx*h^x + dy*h^y + (p - Σ di - dx - dy)*h^z = r - Σ di*h^i
//                     i!=x,y,z                  i!=x,y,z
//
// dx*h^x + dy*h^y + (p - Σ di)*h^z - dx*h^z - dy*h^z = r - Σ di*h^i
//                     i!=x,y,z                          i!=x,y,z
//
// dx*(h^x - h^z) + dy*(h^y - h^z) + (p - Σ di)*h^z = r - Σ di*h^i
//                                     i!=x,y,z        i!=x,y,z
//
// Subst dy:
//
//                  ( (q - Σ di*g^i) - (p - Σ di)*g^z - dx*(g^x - g^z) )
//                  (   i!=x,y,z         i!=x,y,z                      )
// dx*(h^x - h^z) + ( ------------------------------------------------ )*(h^y - h^z) + (p - Σ di)*h^z = r - Σ di*h^i
//                  (                    g^y - g^z                     )                 i!=x,y,z        i!=x,y,z
//
//
//                  ( (q - Σ di*g^i) - (p - Σ di)*g^z )
//                  (   i!=x,y,z         i!=x,y,z     )               ( dx*(g^x - g^z) )
// dx*(h^x - h^z) + ( ------------------------------- )*(h^y - h^z) - ( -------------- )*(h^y - h^z) + (p - Σ di)*h^z = r - Σ di*h^i
//                  (             g^y - g^z           )               (   g^y - g^z    )                 i!=x,y,z        i!=x,y,z
//
//                                                                                     ( (q - Σ di*g^i) - (p - Σ di)*g^z )
//                  ( dx*(g^x - g^z) )                                                 (   i!=x,y,z         i!=x,y,z     )
// dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = (r - Σ di*h^i) - (p - Σ di)*h^z - ( ------------------------------- )*(h^y - h^z)
//                  (   g^y - g^z    )                 i!=x,y,z         i!=x,y,z       (             g^y - g^z           )
//
//                                                   (r - Σ di*h^i)*(g^y - g^z) - (p - Σ di)*h^z*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) + (p - Σ di)*g^z*(h^y - h^z)
//                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = -----------------------------------------------------------------------------------------------------------------
//                  (   g^y - g^z    )                                                              g^y - g^z
//
//                                                   (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(h^z*(g^y - g^z) - g^z*(h^y - h^z))
//                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = --------------------------------------------------------------------------------------------------------
//                  (   g^y - g^z    )                                                              g^y - g^z
//
//                                                   (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^z - g^z*h^y + g^z*h^z)
//                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = ------------------------------------------------------------------------------------------------------------
//                  (   g^y - g^z    )                                                              g^y - g^z
//
//                                                   (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//                  ( dx*(g^x - g^z) )                 i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx*(h^x - h^z) - ( -------------- )*(h^y - h^z) = ----------------------------------------------------------------------------------------
//                  (   g^y - g^z    )                                                         g^y - g^z
//
//                                                           (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
// dx*(h^x - h^z)*(g^y - g^z) - dx*(g^x - g^z)*(h^y - h^z)     i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// ------------------------------------------------------- = ----------------------------------------------------------------------------------------
//                        g^y - g^z                                                                    g^y - g^z
//
// dx*(h^x - h^z)*(g^y - g^z) - dx*(g^x - g^z)*(h^y - h^z) = (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//                                                             i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//
// dx*((h^x - h^z)*(g^y - g^z) - (h^y - h^z)*(g^x - g^z)) = (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//                                                            i!=x,y,z                     i!=x,y,z                     i!=x,y,z
//
//      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//        i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx = ----------------------------------------------------------------------------------------
//                       (h^x - h^z)*(g^y - g^z) - (h^y - h^z)*(g^x - g^z)
//
//



// Recall h = g^2
//
//      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(h^y - h^z) - (p - Σ di)*(g^y*h^z - g^z*h^y)
//         i!=x,y,z                     i!=x,y,z                     i!=x,y,z
// dx = ----------------------------------------------------------------------------------------
//                       (h^x - h^z)*(g^y - g^z) - (h^y - h^z)*(g^x - g^z)
//
//      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(g^2y - g^2z) - (p - Σ di)*(g^y*g^2z - g^z*g^2y)
//         i!=x,y,z                     i!=x,y,z                       i!=x,y,z
// dx = --------------------------------------------------------------------------------------------
//                         (g^2x - g^2z)*(g^y - g^z) - (g^2y - g^2z)*(g^x - g^z)
//
//      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(g^2y - g^2z) - (p - Σ di)*(g^y*g^2z - g^z*g^2y)
//         i!=x,y,z                     i!=x,y,z                       i!=x,y,z
// dx = --------------------------------------------------------------------------------------------
//        (g^(2x+y) - g^(2x+z) - g^(2z+y) + g^(2z+z)) - (g^(2y+x) - g^(2y+z) - g^(2z+x) + g^(2z+z))
//
//      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(g^2y - g^2z) - (p - Σ di)*(g^y*g^2z - g^z*g^2y)
//         i!=x,y,z                     i!=x,y,z                       i!=x,y,z
// dx = --------------------------------------------------------------------------------------------
//                     g^(2x+y) - g^(2x+z) - g^(2z+y) - g^(2y+x) + g^(2y+z) + g^(2z+x)
//
//      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(g^2y - g^2z) - (p - Σ di)*(g^y*g^2z - g^z*g^2y)
//         i!=x,y,z                     i!=x,y,z                       i!=x,y,z
// dx = --------------------------------------------------------------------------------------------
//                     g^(2x+y) - g^(2x+z) - g^(2z+y) - g^(2y+x) + g^(2y+z) + g^(2z+x)
//
// wolfram says this is (g^x - g^y)*(g^x - g^z)*(g^y - g^z) ?
//
//      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(g^2y - g^2z) - (p - Σ di)*(g^y*g^2z - g^z*g^2y)
//         i!=x,y,z                     i!=x,y,z                       i!=x,y,z
// dx = --------------------------------------------------------------------------------------------
//                               (g^x - g^y)*(g^x - g^z)*(g^y - g^z)
//
//      (r - Σ di*h^i)*(g^y - g^z) - (q - Σ di*g^i)*(g^y - g^z)^2 - (p - Σ di)*g^y*g^z*(g^z - g^y)
//         i!=x,y,z                     i!=x,y,z                       i!=x,y,z
// dx = --------------------------------------------------------------------------------------------
//                               (g^x - g^y)*(g^x - g^z)*(g^y - g^z)
//
//      (r - Σ di*h^i) - (q - Σ di*g^i)*(g^y - g^z) - (p - Σ di)*g^y*g^z
//         i!=x,y,z         i!=x,y,z                     i!=x,y,z
// dx = ----------------------------------------------------------------
//                      (g^x - g^y)*(g^x - g^z)
//
// Hmm
//
//      (q - Σ di*g^i) - (p - Σ di)*g^y
//         i!=x,y           i!=x,y
// dx = -------------------------------
//                g^x - g^y
//










// NOTE NO
// Wait, actually (for double parity)
//
// p = d0 + d1 + d2 + ... = Σ di
//                          i
//
// q = d0*g^0 + d1*g^1 + d2*g^2 = Σ di*g^i
//                                i
//
// For two missing blocks dx and dy:
//
// dx + dy = p - Σ di
//             i!=x,y
//
// dx*g^x + dy*g^y = q - Σ di*g^i
//                     i!=x,y
//
// Solve:
//
// dy*g^y = q - Σ di*g^i - dx*g^x
//            i!=x,y
//
//      q - Σ di*g^i - dx*g^x
//        i!=x,y
// dy = ---------------------
//               g^y
//
// Subst dy:
//
//      ( q - Σ di*g^i - dx*g^x )
//      (   i!=x,y              )
// dx + ( --------------------- ) = p - Σ di
//      (          g^y          )     i!=x,y
//
//      ( q - Σ di*g^i )
//      (   i!=x,y     )   ( dx*g^x )
// dx + ( ------------ ) - ( ------ ) = p - Σ di
//      (     g^y      )   (  g^y   )     i!=x,y
//
//                              ( q - Σ di*g^i )
//      ( dx*g^x )              (   i!=x,y     )
// dx - ( ------ ) = p - Σ di - ( ------------ )
//      (  g^y   )     i!=x,y   (     g^y      )
//
// dx*g^y - dx*g^x   (p - Σ di)*g^y - q - Σ di*g^i
// --------------- = -----------------------------
//       g^y                      g^y
//
// dx*g^y - dx*g^x = (p - Σ di)*g^y - q - Σ di*g^i
//
//      (p - Σ di)*g^y - q - Σ di*g^i
// dx = -----------------------------
//                 g^y - g^x
//
// Hmmmmm
// 


// Also (for p + 2d corruption): 
//
// dx*g^x + dy*g^y = q - Σ di*g^i
//                         i!=x,y
//
// dx*h^x + dy*h^y = r - Σ di*h^i
//                         i!=x,y
//
// Subst:
//
// 
// dy*g^y = q - Σ di*g^i - dx^g^x
// 
//      q - Σ di*g^i - dx^g^x
// dy = ---------------------
//               g^y
// 
// Subst:
// 
//          ( q - Σ di*g^i - dx^g^x )
// dx*h^x + ( --------------------- )*h^y = r - Σ di*h^i
//          (          g^y          )
// 
//          ( q - Σ di*g^i )          ( g^x*h^y )
// dx*h^x + ( ------------ )*h^y - dx*( ------- ) = r - Σ di*h^i
//          (      g^y     )          (   g^y   )
// 
//             ( g^x*h^y )                   ( q - Σ di*g^i )    
// dx*h^x - dx*( ------- )  = r - Σ di*h^i - ( ------------ )*h^y
//             (   g^y   )                   (      g^y     )
// 
//             ( g^x*h^y )    (r - Σ di*h^i)*g^y - (q - Σ di*g^i)*h^y
// dx*h^x - dx*( ------- )  = ---------------------------------------
//             (   g^y   )                      g^y  
// 
//    ( h^x*g^y - g^x*h^y )    (r - Σ di*h^i)*g^y - (q - Σ di*g^i)*h^y
// dx*( ----------------- )  = ---------------------------------------
//    (        g^y        )                      g^y  
// 
// dx*(h^x*g^y - g^x*h^y) = (r - Σ di*h^i)*g^y - (q - Σ di*g^i)*h^y
// 
//      (r - Σ di*h^i)*g^y - (q - Σ di*g^i)*h^y
// dx = ---------------------------------------
//                 g^y*h^x - g^x*h^y



// (h^x - h^z)*(g^y - g^z) - (h^y - h^z)*(g^x - g^z)







use crate::macros::raid;


// RAID-parity functions
//

#[raid(parity=1)]
pub mod raid5 {}

#[raid(parity=2)]
pub mod raid6 {}

#[raid(parity=3)]
pub mod raid7 {}


#[cfg(test)]
mod test {
    use super::*;
    use crate::gf::*;
    use crate::macros::*;

    extern crate alloc;
    use alloc::vec::Vec;

    #[test]
    fn raid5() {
        let mut blocks = [
            (80..90).collect::<Vec<u8>>(),
            (20..30).collect::<Vec<u8>>(),
            (30..40).collect::<Vec<u8>>(),
        ];
        let mut p = (40..50).collect::<Vec<u8>>();

        // format
        raid5::format(&mut blocks, &mut p);

        // update
        raid5::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
        assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());

        for i in 0..blocks.len()+1 {
            // clobber
            if i < blocks.len() { blocks[i].fill(b'x'); }
            // repair
            raid5::repair(&mut blocks, &mut p, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
            assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn raid5_large() {
        let mut blocks = Vec::new();
        for i in 0..255 {
            blocks.push(((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }
        let mut p = (10..20).collect::<Vec<u8>>();

        // format
        raid5::format(&mut blocks, &mut p);

        // mount and update
        raid5::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        for i in 0..255 {
            assert_eq!(&blocks[i], &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }

        for i in 0..255 {
            // clobber
            blocks[i].fill(b'x');
            // repair
            raid5::repair(&mut blocks, &mut p, &[i]).unwrap();
            
            for i in 0..255 {
                assert_eq!(&blocks[i], &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
            }
        }
    }

    #[test]
    fn raid6() {
        let mut blocks = [
            (80..90).collect::<Vec<u8>>(),
            (20..30).collect::<Vec<u8>>(),
            (30..40).collect::<Vec<u8>>(),
        ];
        let mut p = (40..50).collect::<Vec<u8>>();
        let mut q = (50..60).collect::<Vec<u8>>();

        // format
        raid6::format(&mut blocks, &mut p, &mut q);

        // update
        raid6::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p, &mut q);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
        assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());

        for i in 0..blocks.len()+2 {
            // clobber
            if i < blocks.len() { blocks[i].fill(b'x'); }
            // repair
            raid6::repair(&mut blocks, &mut p, &mut q, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
            assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
        }

        for i in 0..blocks.len()+2 {
            for j in 0..blocks.len()+2 {
                if i == j {
                    continue;
                }

                // clobber
                if i < blocks.len() { blocks[i].fill(b'x'); }
                if j < blocks.len() { blocks[j].fill(b'x'); }
                // repair
                raid6::repair(&mut blocks, &mut p, &mut q, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
                assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
                assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
            }
        }
    }

    #[test]
    fn raid6_large() {
        let mut blocks = Vec::new();
        for i in 0..255 {
            blocks.push(((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }
        let mut p = (10..20).collect::<Vec<u8>>();
        let mut q = (10..20).collect::<Vec<u8>>();

        // format
        raid6::format(&mut blocks, &mut p, &mut q);

        // mount and update
        raid6::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p, &mut q);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        for i in 0..255 {
            assert_eq!(&blocks[i], &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }

        for i in 0..255-1 {
            // clobber
            blocks[i+0].fill(b'x');
            blocks[i+1].fill(b'x');
            // repair
            raid6::repair(&mut blocks, &mut p, &mut q, &[i+0, i+1]).unwrap();

            for i in 0..255 {
                assert_eq!(&blocks[i], &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
            }
        }
    }

    #[test]
    fn raid7() {
        let mut blocks = [
            (80..90).collect::<Vec<u8>>(),
            (20..30).collect::<Vec<u8>>(),
            (30..40).collect::<Vec<u8>>(),
        ];
        let mut p = (40..50).collect::<Vec<u8>>();
        let mut q = (50..60).collect::<Vec<u8>>();
        let mut r = (60..70).collect::<Vec<u8>>();

        // format
        raid7::format(&mut blocks, &mut p, &mut q, &mut r);

        // update
        raid7::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p, &mut q, &mut r);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
        assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());

        for i in 0..blocks.len()+3 {
            // clobber
            if i < blocks.len() { blocks[i].fill(b'x'); }
            // repair
            raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
            assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
        }

        for i in 0..blocks.len()+3 {
            for j in 0..blocks.len()+3 {
                if i == j {
                    continue;
                }

                // clobber
                if i < blocks.len() { blocks[i].fill(b'x'); }
                if j < blocks.len() { blocks[j].fill(b'x'); }
                // repair
                raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
                assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
                assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
            }
        }

        for i in 0..blocks.len()+3 {
            for j in 0..blocks.len()+3 {
                for k in 0..blocks.len()+3 {
                    if i == j || i == k || j == k {
                        continue;
                    }

                    // clobber
                    if i < blocks.len() { blocks[i].fill(b'x'); }
                    if j < blocks.len() { blocks[j].fill(b'x'); }
                    if k < blocks.len() { blocks[k].fill(b'x'); }
                    // repair
                    raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j, k]).unwrap();
                    assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
                    assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
                    assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
                }
            }
        }
    }

    #[test]
    fn raid7_large() {
        let mut blocks = Vec::new();
        for i in 0..255 {
            blocks.push(((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }
        let mut p = (10..20).collect::<Vec<u8>>();
        let mut q = (10..20).collect::<Vec<u8>>();
        let mut r = (10..20).collect::<Vec<u8>>();

        // format
        raid7::format(&mut blocks, &mut p, &mut q, &mut r);

        // mount and update
        raid7::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p, &mut q, &mut r);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        for i in 0..255 {
            assert_eq!(&blocks[i], &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }

        for i in 0..255-2 {
            // clobber
            blocks[i+0].fill(b'x');
            blocks[i+1].fill(b'x');
            blocks[i+2].fill(b'x');
            // repair
            raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i+0, i+1, i+2]).unwrap();

            for i in 0..255 {
                assert_eq!(&blocks[i], &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
            }
        }
    }

    // why do we have this option?
    #[raid(parity=0)]
    pub mod raid0 {}

    #[test]
    fn raid0() {
        let mut blocks = [
            (80..90).collect::<Vec<u8>>(),
            (20..30).collect::<Vec<u8>>(),
            (30..40).collect::<Vec<u8>>(),
        ];

        // format
        raid0::format(&mut blocks);

        // update
        raid0::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>());
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
        assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
    }

    // multi-byte RAID-parity
    #[raid(gf=gf2p64, u=u64, parity=3)]
    pub mod gf2p64_raid7 {}

    #[test]
    fn gf2p64_raid7() {
        let mut blocks = [
            (80..90).collect::<Vec<u64>>(),
            (20..30).collect::<Vec<u64>>(),
            (30..40).collect::<Vec<u64>>(),
        ];
        let mut p = (40..50).collect::<Vec<u64>>();
        let mut q = (50..60).collect::<Vec<u64>>();
        let mut r = (60..70).collect::<Vec<u64>>();

        // format
        gf2p64_raid7::format(&mut blocks, &mut p, &mut q, &mut r);

        // update
        gf2p64_raid7::update(0, &mut blocks[0], &(10..20).collect::<Vec<u64>>(), &mut p, &mut q, &mut r);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u64>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u64>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u64>>());
        assert_eq!(&blocks[2], &(30..40).collect::<Vec<u64>>());

        for i in 0..blocks.len()+3 {
            // clobber
            if i < blocks.len() { blocks[i].fill(0x7878787878787878); }
            // repair
            gf2p64_raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u64>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u64>>());
            assert_eq!(&blocks[2], &(30..40).collect::<Vec<u64>>());
        }

        for i in 0..blocks.len()+3 {
            for j in 0..blocks.len()+3 {
                if i == j {
                    continue;
                }

                // clobber
                if i < blocks.len() { blocks[i].fill(0x7878787878787878); }
                if j < blocks.len() { blocks[j].fill(0x7878787878787878); }
                // repair
                gf2p64_raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).collect::<Vec<u64>>());
                assert_eq!(&blocks[1], &(20..30).collect::<Vec<u64>>());
                assert_eq!(&blocks[2], &(30..40).collect::<Vec<u64>>());
            }
        }

        for i in 0..blocks.len()+3 {
            for j in 0..blocks.len()+3 {
                for k in 0..blocks.len()+3 {
                    if i == j || i == k || j == k {
                        continue;
                    }

                    // clobber
                    if i < blocks.len() { blocks[i].fill(0x7878787878787878); }
                    if j < blocks.len() { blocks[j].fill(0x7878787878787878); }
                    if k < blocks.len() { blocks[k].fill(0x7878787878787878); }
                    // repair
                    gf2p64_raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j, k]).unwrap();
                    assert_eq!(&blocks[0], &(10..20).collect::<Vec<u64>>());
                    assert_eq!(&blocks[1], &(20..30).collect::<Vec<u64>>());
                    assert_eq!(&blocks[2], &(30..40).collect::<Vec<u64>>());
                }
            }
        }
    }

    // RAID-parity with very odd sizes
    #[gf(polynomial=0x13, generator=0x2)]
    type gf16;
    #[raid(gf=gf16, u=u8, parity=3)]
    pub mod gf16_raid7 {}

    #[gf(polynomial=0x800021, generator=0x2)]
    type gf2p23;
    #[raid(gf=gf2p23, u=u32, parity=3)]
    pub mod gf2p23_raid7 {}

    #[test]
    fn gf16_raid7() {
        let mut blocks = [
            (80..90).map(|x| x%16).collect::<Vec<u8>>(),
            (20..30).map(|x| x%16).collect::<Vec<u8>>(),
            (30..40).map(|x| x%16).collect::<Vec<u8>>(),
        ];
        let mut p = (40..50).map(|x| x%16).collect::<Vec<u8>>();
        let mut q = (50..60).map(|x| x%16).collect::<Vec<u8>>();
        let mut r = (60..70).map(|x| x%16).collect::<Vec<u8>>();

        // format
        gf16_raid7::format(&mut blocks, &mut p, &mut q, &mut r);

        // update
        gf16_raid7::update(0, &mut blocks[0], &(10..20).map(|x| x%16).collect::<Vec<u8>>(), &mut p, &mut q, &mut r);
        blocks[0].copy_from_slice(&(10..20).map(|x| x%16).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).map(|x| x%16).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).map(|x| x%16).collect::<Vec<u8>>());
        assert_eq!(&blocks[2], &(30..40).map(|x| x%16).collect::<Vec<u8>>());

        for i in 0..blocks.len()+3 {
            // clobber
            if i < blocks.len() { blocks[i].fill(0x7); }
            // repair
            gf16_raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).map(|x| x%16).collect::<Vec<u8>>());
            assert_eq!(&blocks[1], &(20..30).map(|x| x%16).collect::<Vec<u8>>());
            assert_eq!(&blocks[2], &(30..40).map(|x| x%16).collect::<Vec<u8>>());
        }

        for i in 0..blocks.len()+3 {
            for j in 0..blocks.len()+3 {
                if i == j {
                    continue;
                }

                // clobber
                if i < blocks.len() { blocks[i].fill(0x7); }
                if j < blocks.len() { blocks[j].fill(0x7); }
                // repair
                gf16_raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).map(|x| x%16).collect::<Vec<u8>>());
                assert_eq!(&blocks[1], &(20..30).map(|x| x%16).collect::<Vec<u8>>());
                assert_eq!(&blocks[2], &(30..40).map(|x| x%16).collect::<Vec<u8>>());
            }
        }

        for i in 0..blocks.len()+3 {
            for j in 0..blocks.len()+3 {
                for k in 0..blocks.len()+3 {
                    if i == j || i == k || j == k {
                        continue;
                    }

                    // clobber
                    if i < blocks.len() { blocks[i].fill(0x7); }
                    if j < blocks.len() { blocks[j].fill(0x7); }
                    if k < blocks.len() { blocks[k].fill(0x7); }
                    // repair
                    gf16_raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j, k]).unwrap();
                    assert_eq!(&blocks[0], &(10..20).map(|x| x%16).collect::<Vec<u8>>());
                    assert_eq!(&blocks[1], &(20..30).map(|x| x%16).collect::<Vec<u8>>());
                    assert_eq!(&blocks[2], &(30..40).map(|x| x%16).collect::<Vec<u8>>());
                }
            }
        }
    }

    #[test]
    fn gf2p23_raid7() {
        let mut blocks = [
            (80..90).collect::<Vec<u32>>(),
            (20..30).collect::<Vec<u32>>(),
            (30..40).collect::<Vec<u32>>(),
        ];
        let mut p = (40..50).collect::<Vec<u32>>();
        let mut q = (50..60).collect::<Vec<u32>>();
        let mut r = (60..70).collect::<Vec<u32>>();

        // format
        gf2p23_raid7::format(&mut blocks, &mut p, &mut q, &mut r);

        // update
        gf2p23_raid7::update(0, &mut blocks[0], &(10..20).collect::<Vec<u32>>(), &mut p, &mut q, &mut r);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u32>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u32>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u32>>());
        assert_eq!(&blocks[2], &(30..40).collect::<Vec<u32>>());

        for i in 0..blocks.len()+3 {
            // clobber
            if i < blocks.len() { blocks[i].fill(0x787878); }
            // repair
            gf2p23_raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u32>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u32>>());
            assert_eq!(&blocks[2], &(30..40).collect::<Vec<u32>>());
        }

        for i in 0..blocks.len()+3 {
            for j in 0..blocks.len()+3 {
                if i == j {
                    continue;
                }

                // clobber
                if i < blocks.len() { blocks[i].fill(0x787878); }
                if j < blocks.len() { blocks[j].fill(0x787878); }
                // repair
                gf2p23_raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).collect::<Vec<u32>>());
                assert_eq!(&blocks[1], &(20..30).collect::<Vec<u32>>());
                assert_eq!(&blocks[2], &(30..40).collect::<Vec<u32>>());
            }
        }

        for i in 0..blocks.len()+3 {
            for j in 0..blocks.len()+3 {
                for k in 0..blocks.len()+3 {
                    if i == j || i == k || j == k {
                        continue;
                    }

                    // clobber
                    if i < blocks.len() { blocks[i].fill(0x787878); }
                    if j < blocks.len() { blocks[j].fill(0x787878); }
                    if k < blocks.len() { blocks[k].fill(0x787878); }
                    // repair
                    gf2p23_raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j, k]).unwrap();
                    assert_eq!(&blocks[0], &(10..20).collect::<Vec<u32>>());
                    assert_eq!(&blocks[1], &(20..30).collect::<Vec<u32>>());
                    assert_eq!(&blocks[2], &(30..40).collect::<Vec<u32>>());
                }
            }
        }
    }

    // this is a good way to test that this math works even in small fields
//    #[gf(polynomial=0x25, generator=0x2)]
//    type gf32;
//    #[raid(gf=gf32, u=u8, parity=3)]
//    pub mod gf32_raid7 {}

    #[test]
    fn gf16_raid7_exhaustive() {
        let mut blocks = Vec::new();
        for i in 0..15 {
            blocks.push(((i+1)*10..(i+2)*10).map(|x| (x as u8)%16).collect::<Vec<u8>>());
        }
        let mut p = (10..20).collect::<Vec<u8>>();
        let mut q = (10..20).collect::<Vec<u8>>();
        let mut r = (10..20).collect::<Vec<u8>>();

        // format
        gf16_raid7::format(&mut blocks, &mut p, &mut q, &mut r);

        for i in 0..blocks.len() {
            for j in 0..blocks.len() {
                for k in 0..blocks.len() {
                    if i == j || i == k || j == k {
                        continue;
                    }

                    // clobber
                    blocks[i].fill(0x7);
                    blocks[j].fill(0x7);
                    blocks[k].fill(0x7);

                    // repair
                    gf16_raid7::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j, k]).unwrap();

                    assert_eq!(&blocks[i], &((i+1)*10..(i+2)*10).map(|x| (x as u8)%16).collect::<Vec<u8>>());
                    assert_eq!(&blocks[j], &((j+1)*10..(j+2)*10).map(|x| (x as u8)%16).collect::<Vec<u8>>());
                    assert_eq!(&blocks[k], &((k+1)*10..(k+2)*10).map(|x| (x as u8)%16).collect::<Vec<u8>>());
                }
            }
        }
    }

    // all RAID-parity params
    #[raid(gf=gf256, u=u8, parity=3)]
    pub mod raid7_all_params {}

    #[test]
    fn raid_all_params() {
        let mut blocks = [
            (80..90).collect::<Vec<u8>>(),
            (20..30).collect::<Vec<u8>>(),
            (30..40).collect::<Vec<u8>>(),
        ];
        let mut p = (40..50).collect::<Vec<u8>>();
        let mut q = (50..60).collect::<Vec<u8>>();
        let mut r = (60..70).collect::<Vec<u8>>();

        // format
        raid7_all_params::format(&mut blocks, &mut p, &mut q, &mut r);

        // update
        raid7_all_params::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p, &mut q, &mut r);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
        assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());

        for i in 0..blocks.len()+3 {
            // clobber
            if i < blocks.len() { blocks[i].fill(b'x'); }
            // repair
            raid7_all_params::repair(&mut blocks, &mut p, &mut q, &mut r, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
            assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
        }

        for i in 0..blocks.len()+3 {
            for j in 0..blocks.len()+3 {
                if i == j {
                    continue;
                }

                // clobber
                if i < blocks.len() { blocks[i].fill(b'x'); }
                if j < blocks.len() { blocks[j].fill(b'x'); }
                // repair
                raid7_all_params::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
                assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
                assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
            }
        }

        for i in 0..blocks.len()+3 {
            for j in 0..blocks.len()+3 {
                for k in 0..blocks.len()+3 {
                    if i == j || i == k || j == k {
                        continue;
                    }

                    // clobber
                    if i < blocks.len() { blocks[i].fill(b'x'); }
                    if j < blocks.len() { blocks[j].fill(b'x'); }
                    if k < blocks.len() { blocks[k].fill(b'x'); }
                    // repair
                    raid7_all_params::repair(&mut blocks, &mut p, &mut q, &mut r, &[i, j, k]).unwrap();
                    assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
                    assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
                    assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
                }
            }
        }
    }
}
