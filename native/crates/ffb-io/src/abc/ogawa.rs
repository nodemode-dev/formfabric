pub const DATA_BIT: u64 = 1 << 63;
pub const EMPTY_DATA: u64 = DATA_BIT;
pub const HEADER_LEN: usize = 16;

pub struct Writer {
    buf: Vec<u8>,
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer {
    pub fn new() -> Writer {
        let mut buf = Vec::with_capacity(1 << 16);
        buf.extend_from_slice(b"Ogawa");
        buf.push(0x00);
        buf.extend_from_slice(&[0x00, 0x01]);
        buf.extend_from_slice(&[0u8; 8]);
        Writer { buf }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.len() <= HEADER_LEN
    }

    pub fn data(&mut self, bytes: &[u8]) -> u64 {
        if bytes.is_empty() {
            return EMPTY_DATA;
        }
        let at = self.buf.len() as u64;
        self.buf.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        self.buf.extend_from_slice(bytes);
        at | DATA_BIT
    }

    pub fn group(&mut self, entries: &[u64]) -> u64 {
        if entries.is_empty() {
            return 0;
        }
        let at = self.buf.len() as u64;
        self.buf.extend_from_slice(&(entries.len() as u64).to_le_bytes());
        for e in entries {
            self.buf.extend_from_slice(&e.to_le_bytes());
        }
        at
    }

    pub fn finish(mut self, root: u64) -> Result<Vec<u8>, String> {
        if root == 0 || root & DATA_BIT != 0 {
            return Err("the root of an Ogawa file must be a group with children".into());
        }
        self.buf[8..16].copy_from_slice(&root.to_le_bytes());
        self.buf[5] = 0xFF;
        Ok(self.buf)
    }
}
