
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

    // all RAID-parity params
    #[raid(parity=2, gf=gf256)]
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
