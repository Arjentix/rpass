use num_bigint::BigUint;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// RSA-Key
#[derive(PartialEq, Eq, Debug)]
pub struct Key (BigUint, BigUint);

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
    /// 
    /// # Panics
    /// Panics if some error occurred during reading from `bytes`
    pub fn from_bytes(mut bytes: &[u8]) -> Self {
        Key(Self::read_part(&mut bytes), Self::read_part(&mut bytes))
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
    /// 
    /// # Panics
    /// Panics if some error occurred during reading from `read`
    fn read_part(read: &mut dyn Read) -> BigUint {
        let len = read.read_u64::<LittleEndian>().unwrap() as usize;
        let mut part_bytes = vec![0u8; len];
        read.read_exact(&mut part_bytes).unwrap();
        BigUint::from_bytes_le(&part_bytes)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigUint;

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

        let key = Key::from_bytes(&bytes);
        assert_eq!(key.0, big_e);
        assert_eq!(key.1, big_n);
    }

    #[test]
    fn test_from_as_bytes() {
        let e = 18764u64;
        let n = 8975u64;
        let key = Key(e.to_biguint().unwrap(), n.to_biguint().unwrap());

        assert_eq!(key, Key::from_bytes(&key.as_bytes()));
    }

    /// Computes number of bytes needful to represent `bits` number of bits
    fn bytes_per_bits(bits: u64) -> u64 {
        match bits % 8 {
            0 => bits / 8,
            _ => bits / 8 + 1
        }
    }

}
