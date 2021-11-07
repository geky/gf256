
use crate::macros::raid;


// Default RAID-parity structs
//

#[raid(parity=1)]
pub struct Raid4 {}

#[raid(parity=2)]
pub struct Raid6 {}


#[cfg(test)]
mod test {
    use super::*;
    use crate::gf::*;
    use core::cell::RefCell;

    extern crate alloc;
    use alloc::vec::Vec;

    extern crate std;
    use std::io;
    use std::io::Write;

    #[test]
    fn raid4() {
        let mut blocks = [
            io::Cursor::new((80..90).collect::<Vec<u8>>()),
            io::Cursor::new((20..30).collect::<Vec<u8>>()),
            io::Cursor::new((30..40).collect::<Vec<u8>>()),
            io::Cursor::new((40..50).collect::<Vec<u8>>()),
        ];

        // format
        Raid4::format(&mut blocks).unwrap();

        // mount and update
        let blocks_cell = RefCell::new(blocks);
        let mut disks = Raid4::mount(&blocks_cell).unwrap();
        disks[0].write_all(&(10..20).collect::<Vec<u8>>()).unwrap();
        let mut blocks = blocks_cell.into_inner();
        assert_eq!(blocks[0].get_ref(), &(10..20).collect::<Vec<u8>>());
        assert_eq!(blocks[1].get_ref(), &(20..30).collect::<Vec<u8>>());
        assert_eq!(blocks[2].get_ref(), &(30..40).collect::<Vec<u8>>());

        for i in 0..blocks.len() {
            // clobber
            blocks[i].get_mut().fill(b'x');
            // repair
            Raid4::repair(&mut blocks, &[i]).unwrap();
            assert_eq!(blocks[0].get_ref(), &(10..20).collect::<Vec<u8>>());
            assert_eq!(blocks[1].get_ref(), &(20..30).collect::<Vec<u8>>());
            assert_eq!(blocks[2].get_ref(), &(30..40).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn raid4_large() {
        let mut blocks = Vec::new();
        for i in 0..255+1 {
            blocks.push(io::Cursor::new(((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>()));
        }

        // format
        Raid4::format(&mut blocks).unwrap();

        // mount and update
        let blocks_cell = RefCell::new(blocks);
        let mut disks = Raid4::mount(&blocks_cell).unwrap();
        disks[0].write_all(&(10..20).collect::<Vec<u8>>()).unwrap();
        let mut blocks = blocks_cell.into_inner();
        for i in 0..255 {
            assert_eq!(blocks[i].get_ref(), &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }

        // clobber
        blocks[0].get_mut().fill(b'x');
        // repair
        Raid4::repair(&mut blocks, &[0]).unwrap();
        for i in 0..255 {
            assert_eq!(blocks[i].get_ref(), &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn raid6() {
        let mut blocks = [
            io::Cursor::new((80..90).collect::<Vec<u8>>()),
            io::Cursor::new((20..30).collect::<Vec<u8>>()),
            io::Cursor::new((30..40).collect::<Vec<u8>>()),
            io::Cursor::new((40..50).collect::<Vec<u8>>()),
        ];

        // format
        Raid6::format(&mut blocks).unwrap();

        // mount and update
        let blocks_cell = RefCell::new(blocks);
        let mut disks = Raid6::mount(&blocks_cell).unwrap();
        disks[0].write_all(&(10..20).collect::<Vec<u8>>()).unwrap();
        let mut blocks = blocks_cell.into_inner();
        assert_eq!(blocks[0].get_ref(), &(10..20).collect::<Vec<u8>>());
        assert_eq!(blocks[1].get_ref(), &(20..30).collect::<Vec<u8>>());

        for i in 0..blocks.len() {
            // clobber
            blocks[i].get_mut().fill(b'x');
            // repair
            Raid6::repair(&mut blocks, &[i]).unwrap();
            assert_eq!(blocks[0].get_ref(), &(10..20).collect::<Vec<u8>>());
            assert_eq!(blocks[1].get_ref(), &(20..30).collect::<Vec<u8>>());
        }

        for i in 0..blocks.len() {
            for j in 0..blocks.len() {
                if i == j {
                    continue;
                }

                // clobber
                blocks[i].get_mut().fill(b'x');
                blocks[j].get_mut().fill(b'x');
                // repair
                Raid6::repair(&mut blocks, &[i, j]).unwrap();
                assert_eq!(blocks[0].get_ref(), &(10..20).collect::<Vec<u8>>());
                assert_eq!(blocks[1].get_ref(), &(20..30).collect::<Vec<u8>>());
            }
        }
    }

    #[test]
    fn raid6_large() {
        let mut blocks = Vec::new();
        for i in 0..255+2 {
            blocks.push(io::Cursor::new(((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>()));
        }

        // format
        Raid6::format(&mut blocks).unwrap();

        // mount and update
        let blocks_cell = RefCell::new(blocks);
        let mut disks = Raid6::mount(&blocks_cell).unwrap();
        disks[0].write_all(&(10..20).collect::<Vec<u8>>()).unwrap();
        let mut blocks = blocks_cell.into_inner();
        for i in 0..255 {
            assert_eq!(blocks[i].get_ref(), &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }

        // clobber
        blocks[0].get_mut().fill(b'x');
        blocks[1].get_mut().fill(b'x');
        // repair
        Raid6::repair(&mut blocks, &[0, 1]).unwrap();
        for i in 0..255 {
            assert_eq!(blocks[i].get_ref(), &((i+1)*10..(i+2)*10).map(|x| x as u8).collect::<Vec<u8>>());
        }
    }

    // why do we have this option?
    #[raid(parity=0)]
    pub struct Raid0 {}

    #[test]
    fn raid0() {
        let mut blocks = [
            io::Cursor::new((80..90).collect::<Vec<u8>>()),
            io::Cursor::new((20..30).collect::<Vec<u8>>()),
            io::Cursor::new((30..40).collect::<Vec<u8>>()),
            io::Cursor::new((40..50).collect::<Vec<u8>>()),
        ];

        // format
        Raid0::format(&mut blocks).unwrap();

        // mount and update
        let blocks_cell = RefCell::new(blocks);
        let mut disks = Raid0::mount(&blocks_cell).unwrap();
        disks[0].write_all(&(10..20).collect::<Vec<u8>>()).unwrap();
        let blocks = blocks_cell.into_inner();
        assert_eq!(blocks[0].get_ref(), &(10..20).collect::<Vec<u8>>());
        assert_eq!(blocks[1].get_ref(), &(20..30).collect::<Vec<u8>>());
        assert_eq!(blocks[2].get_ref(), &(30..40).collect::<Vec<u8>>());
        assert_eq!(blocks[3].get_ref(), &(40..50).collect::<Vec<u8>>());
    }

    // all RAID-parity params
    #[raid(parity=2, gf=gf256)]
    pub struct Raid6AllParams;

    #[test]
    fn raid_all_params() {
        let mut blocks = [
            io::Cursor::new((80..90).collect::<Vec<u8>>()),
            io::Cursor::new((20..30).collect::<Vec<u8>>()),
            io::Cursor::new((30..40).collect::<Vec<u8>>()),
            io::Cursor::new((40..50).collect::<Vec<u8>>()),
        ];

        // format
        Raid6AllParams::format(&mut blocks).unwrap();

        // mount and update
        let blocks_cell = RefCell::new(blocks);
        let mut disks = Raid6AllParams::mount(&blocks_cell).unwrap();
        disks[0].write_all(&(10..20).collect::<Vec<u8>>()).unwrap();
        let mut blocks = blocks_cell.into_inner();
        assert_eq!(blocks[0].get_ref(), &(10..20).collect::<Vec<u8>>());
        assert_eq!(blocks[1].get_ref(), &(20..30).collect::<Vec<u8>>());

        for i in 0..blocks.len() {
            // clobber
            blocks[i].get_mut().fill(b'x');
            // repair
            Raid6AllParams::repair(&mut blocks, &[i]).unwrap();
            assert_eq!(blocks[0].get_ref(), &(10..20).collect::<Vec<u8>>());
            assert_eq!(blocks[1].get_ref(), &(20..30).collect::<Vec<u8>>());
        }

        for i in 0..blocks.len() {
            for j in 0..blocks.len() {
                if i == j {
                    continue;
                }

                // clobber
                blocks[i].get_mut().fill(b'x');
                blocks[j].get_mut().fill(b'x');
                // repair
                Raid6AllParams::repair(&mut blocks, &[i, j]).unwrap();
                assert_eq!(blocks[0].get_ref(), &(10..20).collect::<Vec<u8>>());
                assert_eq!(blocks[1].get_ref(), &(20..30).collect::<Vec<u8>>());
            }
        }
    }
}
