pub use num_bigint::{BigUint, ToBigUint, ParseBigIntError};

use std::io::{Result, Read, Write};
use std::str::FromStr;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// RSA-Key
#[derive(PartialEq, Eq, Debug)]
pub struct Key (pub BigUint, pub BigUint);

impl Key {
    /// Returns byte representation of key
    /// 
    /// # Panics
    /// Panics if can't write to the buffer
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        Self::write_part(&self.0, &mut bytes);
        Self::write_part(&self.1, &mut bytes);

        bytes
    }

    /// Constructs new key from bytes
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        Ok(Key(Self::read_part(&mut bytes)?, Self::read_part(&mut bytes)?))
    }

    /// Writes one part of key to the `write`
    /// 
    /// # Panics
    /// Panics if can't write to the buffer
    fn write_part(part: &BigUint, write: &mut dyn Write) {
        let part_bytes = part.to_bytes_le();
        write.write_u64::<LittleEndian>(part_bytes.len() as u64).unwrap();
        write.write_all(&part_bytes).unwrap();
    }

    /// Reads one part of key from the `read`
    fn read_part(read: &mut dyn Read) -> Result<BigUint> {
        let len = read.read_u64::<LittleEndian>()? as usize;
        let mut part_bytes = vec![0u8; len];
        read.read_exact(&mut part_bytes)?;
        Ok(BigUint::from_bytes_le(&part_bytes))
    }
}

impl FromStr for Key {
    type Err = ParseBigIntError;

    /// Constructs new key from string in format `<first_num>:<second_num>`
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use std::str::FromStr;
    /// use rpass::key::{Key, BigUint, ToBigUint};
    /// 
    /// let key = Key::from_str("898:19634").unwrap();
    /// assert_eq!(key.0, 898u64.to_biguint().unwrap());
    /// assert_eq!(key.1, 19634u64.to_biguint().unwrap());
    /// ```
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut part_iter = s.split(':');
        Ok(Key(BigUint::from_str(part_iter.next().unwrap_or(""))?,
            BigUint::from_str(part_iter.next().unwrap_or(""))?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_bytes() {
        let e = 734u64;
        let n = 1040u64;
        let big_e = e.to_biguint().unwrap();
        let big_n = n.to_biguint().unwrap();

        let mut bytes = vec![];
        bytes.write_u64::<LittleEndian>(bytes_per_bits(big_e.bits())).unwrap();
        bytes.write_u16::<LittleEndian>(e as u16).unwrap();
        bytes.write_u64::<LittleEndian>(bytes_per_bits(big_n.bits())).unwrap();
        bytes.write_u16::<LittleEndian>(n as u16).unwrap();

        let key = Key (big_e, big_n);
        
        assert_eq!(bytes, key.as_bytes());
    }

    #[test]
    fn test_from_bytes() {
        let mut bytes = vec![];
        let e = 657u64;
        let n = 298u64;
        let big_e = e.to_biguint().unwrap();
        let big_n = n.to_biguint().unwrap();

        bytes.write_u64::<LittleEndian>(bytes_per_bits(big_e.bits())).unwrap();
        bytes.write_u16::<LittleEndian>(e as u16).unwrap();
        bytes.write_u64::<LittleEndian>(bytes_per_bits(big_n.bits())).unwrap();
        bytes.write_u16::<LittleEndian>(n as u16).unwrap();

        let key = Key::from_bytes(&bytes).unwrap();
        assert_eq!(key.0, big_e);
        assert_eq!(key.1, big_n);
    }

    #[test]
    fn test_from_as_bytes() {
        let e = 18764u64;
        let n = 8975u64;
        let key = Key(e.to_biguint().unwrap(), n.to_biguint().unwrap());

        assert_eq!(key, Key::from_bytes(&key.as_bytes()).unwrap());
    }

    #[test]
    fn test_from_str_empty() {
        assert!(Key::from_str("").is_err());
    }

    #[test]
    fn test_from_str_one_part() {
        assert!(Key::from_str("156").is_err());
        assert!(Key::from_str("19704:").is_err());
        assert!(Key::from_str(":9758").is_err());
        assert!(Key::from_str("41958:key").is_err());
    }

    #[test]
    fn test_from_str_not_a_number() {
        assert!(Key::from_str("public:key").is_err());
    }

    /// Computes number of bytes needful to represent `bits` number of bits
    fn bytes_per_bits(bits: u64) -> u64 {
        match bits % 8 {
            0 => bits / 8,
            _ => bits / 8 + 1
        }
    }

}
