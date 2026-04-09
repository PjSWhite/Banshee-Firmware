pub struct SerialBuffer<const SIZE: usize = 256> {
    data: [u8; SIZE],
    cursor: usize,
}

impl SerialBuffer {
    pub fn buffer(&self) -> &[u8] {
        &self.data[..self.cursor]
    }

    pub fn clear(&mut self) {
        self.cursor = 0;
    }
}

impl<const SIZE: usize> Default for SerialBuffer<SIZE> {
    fn default() -> Self {
        Self {
            data: [0; SIZE],
            cursor: 0,
        }
    }
}

impl core::fmt::Write for SerialBuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = &mut self.data[self.cursor..];
        if bytes.len() > remaining.len() {
            return Err(core::fmt::Error); // full
        }
        remaining[..bytes.len()].copy_from_slice(bytes);
        self.cursor += bytes.len();
        Ok(())
    }
}
