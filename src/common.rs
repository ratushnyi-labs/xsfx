use std::io::{self, Read};

pub const TRAILER_SIZE: u64 = 16;
// Just a random constant marker: "SFXLZMA!" in hex-like style
pub const MAGIC: u64 = 0x5346584C5A4D4121; // "SFXLZMA!"

pub struct Trailer {
    pub payload_len: u64,
    pub magic: u64,
}

impl Trailer {
    pub fn new(payload_len: u64) -> Self {
        Self {
            payload_len,
            magic: MAGIC,
        }
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[..8].copy_from_slice(&self.payload_len.to_le_bytes());
        buf[8..16].copy_from_slice(&self.magic.to_le_bytes());
        buf
    }

    pub fn from_reader<R: Read>(mut r: R) -> io::Result<Self> {
        let mut buf = [0u8; 16];
        r.read_exact(&mut buf)?;
        let mut len_bytes = [0u8; 8];
        let mut magic_bytes = [0u8; 8];
        len_bytes.copy_from_slice(&buf[..8]);
        magic_bytes.copy_from_slice(&buf[8..16]);
        let payload_len = u64::from_le_bytes(len_bytes);
        let magic = u64::from_le_bytes(magic_bytes);
        Ok(Self { payload_len, magic })
    }
}
