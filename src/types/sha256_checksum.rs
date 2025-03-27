pub struct SHA256Checksum {
    pub hash: Vec<u8>,
}

impl SHA256Checksum {
    pub fn to_hex_string(&self) -> String {
        hex::encode(&self.hash)
    }

    pub fn to_base58_string(&self) -> String {
        bs58::encode(&self.hash).into_string()
    }
}
