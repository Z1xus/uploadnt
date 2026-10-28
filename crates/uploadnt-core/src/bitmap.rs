#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitmap {
    bits: Vec<u8>,
    len: u32,
}

impl Bitmap {
    pub fn new(len: u32) -> Self {
        Self { bits: vec![0; len.div_ceil(8) as usize], len }
    }

    pub fn from_bytes(bytes: &[u8], len: u32) -> Self {
        let mut map = Self::new(len);
        let n = map.bits.len().min(bytes.len());
        map.bits[..n].copy_from_slice(&bytes[..n]);
        if !len.is_multiple_of(8)
            && let Some(last) = map.bits.last_mut()
        {
            *last &= (1u8 << (len % 8)) - 1;
        }
        map
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bits
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, i: u32) -> bool {
        i < self.len && self.bits[(i / 8) as usize] & (1 << (i % 8)) != 0
    }

    pub fn set(&mut self, i: u32) {
        if i < self.len {
            self.bits[(i / 8) as usize] |= 1 << (i % 8);
        }
    }

    pub fn count(&self) -> u32 {
        self.bits.iter().map(|b| b.count_ones()).sum()
    }

    pub fn is_full(&self) -> bool {
        self.count() == self.len
    }

    pub fn missing(&self) -> impl Iterator<Item = u32> + '_ {
        (0..self.len).filter(|&i| !self.get(i))
    }
}
