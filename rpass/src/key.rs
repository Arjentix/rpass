pub use num_bigint::{BigUint, ParseBigIntError, ToBigUint};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;

/// RSA-Key
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Key(pub BigUint, pub BigUint);

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("parse error: {0}")]
    ParseKey(#[from] ParseError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("invalid format")]
    InvalidFormat,
    #[error("error parsing big int: {0}")]
    ParseBigInt(#[from] ParseBigIntError),
}

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
        Ok(Key(
            Self::read_part(&mut bytes)?,
            Self::read_part(&mut bytes)?,
        ))
    }

    /// Reads key from file by `path`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rpass::key::{Key, Result};
    ///
    /// # fn main() -> Result<()> {
    /// let key = Key::from_file("~/key.sec")?;
    /// key.decrypt("secret_message");
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let content = fs::read_to_string(path)?;
        Self::from_str(&content).map_err(|err| err.into())
    }

    /// Writes key to file by `path`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rpass::key::{Key, Result};
    ///
    /// # fn main() -> Result<()> {
    /// let (pub_key, sec_key) = Key::generate_pair();
    /// pub_key.write_to_file("~/key.pub")?;
    /// sec_key.write_to_file("~/key.sec")
    /// # }
    /// ```
    pub fn write_to_file<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let content = self.to_string();
        fs::write(path, content).map_err(|err| err.into())
    }

    /// Generate pair of public and secret keys
    ///
    /// TODO
    pub fn generate_pair() -> (Self, Self) {
        (
            Key(269.to_biguint().unwrap(), 221.to_biguint().unwrap()),
            Key(5.to_biguint().unwrap(), 221.to_biguint().unwrap()),
        )
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
    fn write_part<W: Write>(part: &BigUint, mut write: W) {
        let part_bytes = part.to_bytes_le();
        write
            .write_u64::<LittleEndian>(part_bytes.len() as u64)
            .unwrap();
        write.write_all(&part_bytes).unwrap();
    }

    /// Reads one part of key from the `read`
    fn read_part<R: Read>(mut read: R) -> Result<BigUint> {
        let len = read.read_u64::<LittleEndian>()? as usize;
        let mut part_bytes = vec![0u8; len];
        read.read_exact(&mut part_bytes)?;
        Ok(BigUint::from_bytes_le(&part_bytes))
    }
}

impl FromStr for Key {
    type Err = ParseError;

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
        let (pub_part, sec_part) = s.split_once(":").ok_or(ParseError::InvalidFormat)?;
        if pub_part.is_empty() || sec_part.is_empty() {
            return Err(ParseError::InvalidFormat);
        }
        Ok(Key(
            BigUint::from_str(pub_part)?,
            BigUint::from_str(sec_part)?,
        ))
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
        bytes
            .write_u64::<LittleEndian>(bytes_per_bits(big_e.bits()))
            .unwrap();
        bytes.write_u16::<LittleEndian>(e as u16).unwrap();
        bytes
            .write_u64::<LittleEndian>(bytes_per_bits(big_n.bits()))
            .unwrap();
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

        bytes
            .write_u64::<LittleEndian>(bytes_per_bits(big_e.bits()))
            .unwrap();
        bytes.write_u16::<LittleEndian>(e as u16).unwrap();
        bytes
            .write_u64::<LittleEndian>(bytes_per_bits(big_n.bits()))
            .unwrap();
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
        assert!(matches!(
            Key::from_str("156"),
            Err(ParseError::InvalidFormat)
        ));
        assert!(matches!(
            Key::from_str("19704:"),
            Err(ParseError::InvalidFormat)
        ));
        assert!(matches!(
            Key::from_str(":9758"),
            Err(ParseError::InvalidFormat)
        ));
    }

    #[test]
    fn test_from_str_not_a_number() {
        assert!(matches!(
            Key::from_str("public:key"),
            Err(ParseError::ParseBigInt(_))
        ));
    }

    /// Computes number of bytes needful to represent `bits` number of bits
    fn bytes_per_bits(bits: u64) -> u64 {
        match bits % 8 {
            0 => bits / 8,
            _ => bits / 8 + 1,
        }
    }
}
