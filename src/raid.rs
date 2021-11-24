//! ## RAID-parity functions and macros
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








use crate::macros::raid;


// RAID-parity functions
//

#[raid(parity=1)]
pub mod raid4 {}

#[raid(parity=2)]
pub mod raid6 {}


#[cfg(test)]
mod test {
    use super::*;
    use crate::gf::*;
    use crate::macros::*;

    extern crate alloc;
    use alloc::vec::Vec;

    #[test]
    fn raid4() {
        let mut blocks = [
            (80..90).collect::<Vec<u8>>(),
            (20..30).collect::<Vec<u8>>(),
            (30..40).collect::<Vec<u8>>(),
        ];
        let mut p = (40..50).collect::<Vec<u8>>();

        // format
        raid4::format(&mut blocks, &mut p);

        // update
        raid4::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
        assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());

        for i in 0..blocks.len() {
            // clobber
            blocks[i].fill(b'x');
            // repair
            raid4::repair(&mut blocks, &mut p, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
            assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn raid4_large() {
        let mut blocks = Vec::new();
        for i in 0..255 {
            blocks.push(((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }
        let mut p = (10..20).collect::<Vec<u8>>();

        // format
        raid4::format(&mut blocks, &mut p);

        // mount and update
        raid4::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        for i in 0..255 {
            assert_eq!(&blocks[i], &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }

        // clobber
        blocks[0].fill(b'x');
        // repair
        raid4::repair(&mut blocks, &mut p, &[0]).unwrap();
        for i in 0..255 {
            assert_eq!(&blocks[i], &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn raid6() {
        let mut blocks = [
            (80..90).collect::<Vec<u8>>(),
            (20..30).collect::<Vec<u8>>(),
        ];
        let mut p = (30..40).collect::<Vec<u8>>();
        let mut q = (40..50).collect::<Vec<u8>>();

        // format
        raid6::format(&mut blocks, &mut p, &mut q);

        // update
        raid6::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p, &mut q);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());

        for i in 0..blocks.len() {
            // clobber
            blocks[i].fill(b'x');
            // repair
            raid6::repair(&mut blocks, &mut p, &mut q, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
        }

        for i in 0..blocks.len() {
            for j in 0..blocks.len() {
                if i == j {
                    continue;
                }

                // clobber
                blocks[i].fill(b'x');
                blocks[j].fill(b'x');
                // repair
                raid6::repair(&mut blocks, &mut p, &mut q, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
                assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
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

        // clobber
        blocks[0].fill(b'x');
        blocks[1].fill(b'x');
        // repair
        raid6::repair(&mut blocks, &mut p, &mut q, &[0, 1]).unwrap();
        for i in 0..255 {
            assert_eq!(&blocks[i], &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
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
            (40..50).collect::<Vec<u8>>(),
        ];

        // format
        raid0::format(&mut blocks);

        // update
        raid0::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>());
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
        assert_eq!(&blocks[2], &(30..40).collect::<Vec<u8>>());
        assert_eq!(&blocks[3], &(40..50).collect::<Vec<u8>>());
    }

    // multi-byte RAID-parity
    #[raid(gf=gf2p64, u=u64, parity=2)]
    pub mod gf2p64_raid6 {}

    #[test]
    fn gf2p64_raid6() {
        let mut blocks = [
            (80..90).collect::<Vec<u64>>(),
            (20..30).collect::<Vec<u64>>(),
        ];
        let mut p = (30..40).collect::<Vec<u64>>();
        let mut q = (40..50).collect::<Vec<u64>>();

        // format
        gf2p64_raid6::format(&mut blocks, &mut p, &mut q);

        // update
        gf2p64_raid6::update(0, &mut blocks[0], &(10..20).collect::<Vec<u64>>(), &mut p, &mut q);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u64>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u64>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u64>>());

        for i in 0..blocks.len() {
            // clobber
            blocks[i].fill(0x7878787878787878);
            // repair
            gf2p64_raid6::repair(&mut blocks, &mut p, &mut q, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u64>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u64>>());
        }

        for i in 0..blocks.len() {
            for j in 0..blocks.len() {
                if i == j {
                    continue;
                }

                // clobber
                blocks[i].fill(0x7878787878787878);
                blocks[j].fill(0x7878787878787878);
                // repair
                gf2p64_raid6::repair(&mut blocks, &mut p, &mut q, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).collect::<Vec<u64>>());
                assert_eq!(&blocks[1], &(20..30).collect::<Vec<u64>>());
            }
        }
    }

    // RAID-parity with ver odd sizes
    #[gf(polynomial=0x13, generator=0x2)]
    type gf16;
    #[raid(gf=gf16, u=u8, parity=2)]
    pub mod gf16_raid6 {}

    #[gf(polynomial=0x800021, generator=0x2)]
    type gf2p23;
    #[raid(gf=gf2p23, u=u32, parity=2)]
    pub mod gf2p23_raid6 {}

    #[test]
    fn gf16_raid6() {
        let mut blocks = [
            (80..90).map(|x| x%16).collect::<Vec<u8>>(),
            (20..30).map(|x| x%16).collect::<Vec<u8>>(),
        ];
        let mut p = (30..40).map(|x| x%16).collect::<Vec<u8>>();
        let mut q = (40..50).map(|x| x%16).collect::<Vec<u8>>();

        // format
        gf16_raid6::format(&mut blocks, &mut p, &mut q);

        // update
        gf16_raid6::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p, &mut q);
        blocks[0].copy_from_slice(&(10..20).map(|x| x%16).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).map(|x| x%16).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).map(|x| x%16).collect::<Vec<u8>>());

        for i in 0..blocks.len() {
            // clobber
            blocks[i].fill(0x7);
            // repair
            gf16_raid6::repair(&mut blocks, &mut p, &mut q, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).map(|x| x%16).collect::<Vec<u8>>());
            assert_eq!(&blocks[1], &(20..30).map(|x| x%16).collect::<Vec<u8>>());
        }

        for i in 0..blocks.len() {
            for j in 0..blocks.len() {
                if i == j {
                    continue;
                }

                // clobber
                blocks[i].fill(0x7);
                blocks[j].fill(0x7);
                // repair
                gf16_raid6::repair(&mut blocks, &mut p, &mut q, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).map(|x| x%16).collect::<Vec<u8>>());
                assert_eq!(&blocks[1], &(20..30).map(|x| x%16).collect::<Vec<u8>>());
            }
        }
    }

    #[test]
    fn gf2p23_raid6() {
        let mut blocks = [
            (80..90).collect::<Vec<u32>>(),
            (20..30).collect::<Vec<u32>>(),
        ];
        let mut p = (30..40).collect::<Vec<u32>>();
        let mut q = (40..50).collect::<Vec<u32>>();

        // format
        gf2p23_raid6::format(&mut blocks, &mut p, &mut q);

        // update
        gf2p23_raid6::update(0, &mut blocks[0], &(10..20).collect::<Vec<u32>>(), &mut p, &mut q);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u32>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u32>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u32>>());

        for i in 0..blocks.len() {
            // clobber
            blocks[i].fill(0x787878);
            // repair
            gf2p23_raid6::repair(&mut blocks, &mut p, &mut q, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u32>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u32>>());
        }

        for i in 0..blocks.len() {
            for j in 0..blocks.len() {
                if i == j {
                    continue;
                }

                // clobber
                blocks[i].fill(0x787878);
                blocks[j].fill(0x787878);
                // repair
                gf2p23_raid6::repair(&mut blocks, &mut p, &mut q, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).collect::<Vec<u32>>());
                assert_eq!(&blocks[1], &(20..30).collect::<Vec<u32>>());
            }
        }
    }

    // all RAID-parity params
    #[raid(gf=gf256, u=u8, parity=2)]
    pub mod raid6_all_params {}

    #[test]
    fn raid_all_params() {
        let mut blocks = [
            (80..90).collect::<Vec<u8>>(),
            (20..30).collect::<Vec<u8>>(),
        ];
        let mut p = (30..40).collect::<Vec<u8>>();
        let mut q = (40..50).collect::<Vec<u8>>();

        // format
        raid6_all_params::format(&mut blocks, &mut p, &mut q);

        // update
        raid6_all_params::update(0, &mut blocks[0], &(10..20).collect::<Vec<u8>>(), &mut p, &mut q);
        blocks[0].copy_from_slice(&(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
        assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());

        for i in 0..blocks.len() {
            // clobber
            blocks[i].fill(b'x');
            // repair
            raid6_all_params::repair(&mut blocks, &mut p, &mut q, &[i]).unwrap();
            assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
            assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
        }

        for i in 0..blocks.len() {
            for j in 0..blocks.len() {
                if i == j {
                    continue;
                }

                // clobber
                blocks[i].fill(b'x');
                blocks[j].fill(b'x');
                // repair
                raid6_all_params::repair(&mut blocks, &mut p, &mut q, &[i, j]).unwrap();
                assert_eq!(&blocks[0], &(10..20).collect::<Vec<u8>>());
                assert_eq!(&blocks[1], &(20..30).collect::<Vec<u8>>());
            }
        }
    }
}
