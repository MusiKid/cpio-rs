/// Representation of the header of an archive's entry.
/// It support a couple of formats.
// Different versions of CPIO have incompatible header format.
use std::{convert::TryInto, io::Write};

#[derive(Clone, Default)]
#[repr(C)]
pub struct OldHeader {
    /// Represents the magic number.
    magic: u16,
    /// Represents the device number from the disk.
    dev: u16,
    /// Represents the inode number from the disk.
    ino: u16,
    /// Represents the file mode.
    mode: u16,
    /// Represents the user id.
    uid: u16,
    /// Represents the group id.
    gid: u16,
    /// Represents the number of links to this file. Directories always have a value of at least two here. Note that hardlinked files include file data with every copy in the archive.
    nlink: u16,
    /// Represents the device number associated to block special and character special entries (major/minor).
    rdev: u16,
    /// Represents the modification time of this file.
    mtime: [u16; 2],
    /// Represents the number of bytes in the pathname which follows the header.
    namesize: u16,
    /// Represents the size of this file.
    filesize: [u16; 2],
}

#[derive(Clone, Default)]
#[repr(C)]
pub struct OdcHeader {
    /// Represents the magic number
    magic: [u8; 6],
    /// Represents the device number from the disk.
    dev: [u8; 6],
    /// Represents the inode number from the disk.
    ino: [u8; 6],
    /// Represents the file mode.    
    mode: [u8; 6],
    /// Represents the user id.
    uid: [u8; 6],
    /// Represents the group id.
    gid: [u8; 6],
    /// Represents the number of links to this file. Directories always have a value of at least two here. Note that hardlinked files include file data with every copy in the archive.
    nlink: [u8; 6],
    /// Represents the device number associated to block special and character special entries (major/minor).
    rdev: [u8; 6],
    /// Represents the modification time of this file.
    mtime: [u8; 11],
    /// Represents the number of bytes in the pathname which follows the header.
    namesize: [u8; 6],
    /// Represents the size of this file.
    filesize: [u8; 11],
}

//`newc` and `crc` have the same layout, the only differences are that the checksum is added in `crc` and the magic number is `070702`.
#[derive(Clone, Default)]
#[repr(C)]
pub struct NewcHeader {
    /// Represents the magic number
    magic: [u8; 6],
    /// Represents the inode number from the disk.
    ino: [u8; 8],
    /// Represents the file mode.    
    mode: [u8; 8],
    /// Represents the user id.
    uid: [u8; 8],
    /// Represents the group id.
    gid: [u8; 8],
    /// Represents the number of links to this file. Directories always have a value of at least two here. Note that hardlinked files include file data with every copy in the archive.
    nlink: [u8; 8],
    /// Represents the modification time of this file.
    mtime: [u8; 8],
    /// Represents the size of this file.
    filesize: [u8; 8],
    /// Represents the device major number from the disk.
    devmajor: [u8; 8],
    /// Represents the device minor number from the disk.
    devminor: [u8; 8],
    /// Represents the device major number for special file.
    rdevmajor: [u8; 8],
    /// Represents the device minor number for special file.
    rdevminor: [u8; 8],
    /// Represents the number of bytes in the pathname which follows the header.
    namesize: [u8; 8],
    /// Represents the checksum.
    check: [u8; 8],
}

unsafe fn cast<T, U>(a: &T) -> &U {
    assert_eq!(std::mem::size_of_val(a), std::mem::size_of::<U>());
    assert_eq!(std::mem::align_of_val(a), std::mem::align_of::<U>());
    &*(a as *const T).cast()
}

//WARNING: This function should be used with a lot more caution because it does not check the memory alignement
unsafe fn cast_no_align<T, U>(a: &T) -> &U {
    assert_eq!(std::mem::size_of_val(a), std::mem::size_of::<U>());
    &*(a as *const T).cast()
}

pub trait Header: Sized {
    fn new() -> Self;
    fn as_bytes(&self) -> &[u8];
}

const OLD_HEADER_LEN: usize = 26;
const OLD_MAGIC: &'static [u8] = &[199, 113];
impl Header for OldHeader {
    fn new() -> Self {
        OldHeader {
            magic: u16::from_ne_bytes(OLD_MAGIC.try_into().unwrap()),
            ..Default::default()
        }
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { cast_no_align::<_, [u8; OLD_HEADER_LEN as usize]>(self) }
    }
}

const ODC_HEADER_LEN: usize = 76;
const ODC_MAGIC: &'static [u8] = b"070707";
impl Header for OdcHeader {
    fn new() -> Self {
        OdcHeader {
            magic: ODC_MAGIC.try_into().unwrap(),
            ..Default::default()
        }
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { cast::<_, [u8; ODC_HEADER_LEN as usize]>(self) }
    }
}

const NEWC_HEADER_LEN: usize = 110;
const NEWC_MAGIC: &'static [u8] = b"070701";
impl Header for NewcHeader {
    fn new() -> Self {
        NewcHeader {
            magic: NEWC_MAGIC.try_into().unwrap(),
            ..Default::default()
        }
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { cast::<_, [u8; NEWC_HEADER_LEN as usize]>(self) }
    }
}

trait CRCHeaderExt: Header {
    fn new_crc() -> Self;
}

const CRC_MAGIC: &'static [u8] = b"070702";
impl CRCHeaderExt for NewcHeader {
    fn new_crc() -> Self {
        NewcHeader {
            magic: CRC_MAGIC.try_into().unwrap(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_header() {
        let old_h = OldHeader::new();

        assert!(old_h.as_bytes()[0..2] == OLD_MAGIC[0..2]);
        assert!(old_h.as_bytes().len() == OLD_HEADER_LEN);
    }

    #[test]
    fn odc_header() {
        let odc_h = OdcHeader::new();

        assert!(odc_h.as_bytes()[0..6] == ODC_MAGIC[0..6]);
        assert!(odc_h.as_bytes().len() == ODC_HEADER_LEN);
    }

    #[test]
    fn newc_header() {
        let newc_h = NewcHeader::new();

        assert!(newc_h.as_bytes()[0..6] == NEWC_MAGIC[0..6]);
        assert!(newc_h.as_bytes().len() == NEWC_HEADER_LEN);
    }

    #[test]
    fn crc_header() {
        let crc_h = NewcHeader::new_crc();

        assert!(crc_h.as_bytes()[0..6] == CRC_MAGIC[0..6]);
        assert!(crc_h.as_bytes().len() == NEWC_HEADER_LEN);
    }
}
