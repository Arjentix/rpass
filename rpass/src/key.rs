pub use num_bigint::{BigUint, ToBigUint, ParseBigIntError};

use std::io::{Result, Read, Write};
use std::str::FromStr;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// RSA-Key
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Key (pub BigUint, pub BigUint);

impl Key {
    /// Returns byte representation of key
    ///
    /// # Panics
    ///
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

    /// Generate pair of public and secret keys
    ///
    /// TODO
    pub fn generate_pair() -> (Self, Self) {
        (Key(269.to_biguint().unwrap(), 221.to_biguint().unwrap()),
         Key(5.to_biguint().unwrap(), 221.to_biguint().unwrap()))
    }

    /// Encrypt `s` with key
    ///
    /// TODO
    pub fn encrypt(&self, s: &str) -> String {
        s.to_owned()
    }

    /// Decrypt `s` with key
    ///
    /// TODO
    pub fn decrypt(&self, s: &str) -> String {
        s.to_owned()
    }

    /// Writes one part of key to the `write`
    ///
    /// # Panics
    ///
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

#[derive(thiserror::Error, Debug)]
pub enum ParseKeyError {
    #[error("invalid key format")]
    InvalidKeyFormat,
    #[error("Error parsing big int: {0}")]
    ParseBigIntError(#[from] ParseBigIntError)
}

impl FromStr for Key {
    type Err = ParseKeyError;

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
        let (pub_part, sec_part) = s.split_once(":")
            .ok_or(ParseKeyError::InvalidKeyFormat)?;
        if pub_part.is_empty() || sec_part.is_empty() {
            return Err(ParseKeyError::InvalidKeyFormat);
        }
        Ok(Key(BigUint::from_str(pub_part)?,
            BigUint::from_str(sec_part)?))
    }
}

impl ToString for Key {
    /// Converts key to string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rpass::key::{Key, ToBigUint};
    ///
    /// let key = Key(845u64.to_biguint().unwrap(), 947u64.to_biguint().unwrap());
    /// assert_eq!(key.to_string(), "845:947");
    /// ```
    fn to_string(&self) -> String {
        self.0.to_string() + ":" + &self.1.to_string()
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

        let key = Key(big_e, big_n);

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
    fn test_from_invalid_format() {
        assert!(matches!(Key::from_str("156"),
            Err(ParseKeyError::InvalidKeyFormat)));
        assert!(matches!(Key::from_str("19704:"),
            Err(ParseKeyError::InvalidKeyFormat)));
        assert!(matches!(Key::from_str(":9758"),
            Err(ParseKeyError::InvalidKeyFormat)));
    }

    #[test]
    fn test_from_str_not_a_number() {
        assert!(matches!(Key::from_str("public:key"),
            Err(ParseKeyError::ParseBigIntError(_))));
    }

    /// Computes number of bytes needful to represent `bits` number of bits
    fn bytes_per_bits(bits: u64) -> u64 {
        match bits % 8 {
            0 => bits / 8,
            _ => bits / 8 + 1
        }
    }

}
