use crc::crc32;
use md5;

use crate::consts::{Position,Resource};

/**
 * Generic Hasher interface
 */
pub trait Hasher {
    fn hash(&self, value: Resource) -> Position;
}


/**
 * MD5 Hasher
 */
pub struct Md5Hasher {}
impl Hasher for Md5Hasher {
    fn hash(&self, value: Resource) -> Position {
        let digest = md5::compute(value);
        return format!("{:x}", digest);
    }
}

#[cfg(test)]
#[test]
fn test_md5() {
    let hasher = Md5Hasher {};
    assert_eq!(
        hasher.hash(String::from("test")),
        "098f6bcd4621d373cade4e832627b4f6"
    );
    assert_eq!(
        hasher.hash(String::from("test")),
        "098f6bcd4621d373cade4e832627b4f6"
    );
    assert_eq!(
        hasher.hash(String::from("different")),
        "29e4b66fa8076de4d7a26c727b8dbdfa"
    );
}


/**
 * CRC32 Hasher
 */
pub struct Crc32Hasher {}
impl Hasher for Crc32Hasher {
    fn hash(&self, value: Resource) -> Position {
        let digest = crc32::checksum_ieee(value.as_bytes());
        return format!("{}", digest);
    }
}

#[cfg(test)]
#[test]
fn test_crc32() {
    let hasher = Crc32Hasher {};
    assert_eq!(hasher.hash(String::from("test")), "3632233996");
    assert_eq!(hasher.hash(String::from("test")), "3632233996");
    assert_eq!(hasher.hash(String::from("different")), "1812431075");
}