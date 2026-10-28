use serde::{Deserialize, Serialize};

pub const CHUNK: u32 = 1 << 20;
// browsers disagree on the largest data channel message; 64 KiB passes everywhere
pub const PIECE: u32 = 1 << 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub mime: String,
}

impl FileEntry {
    pub fn chunks(&self) -> u32 {
        self.size.div_ceil(CHUNK as u64) as u32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub files: Vec<FileEntry>,
    pub hashes: Vec<[u8; 32]>,
}

impl Manifest {
    pub fn root(&self) -> [u8; 32] {
        *blake3::hash(&postcard::to_allocvec(self).unwrap()).as_bytes()
    }

    pub fn chunks(&self) -> u32 {
        self.hashes.len() as u32
    }

    pub fn size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    pub fn is_valid(&self) -> bool {
        self.files.iter().map(FileEntry::chunks).sum::<u32>() == self.chunks()
    }

    pub fn locate(&self, chunk: u32) -> Option<(u32, u64, u32)> {
        let mut first = 0;
        for (file, entry) in self.files.iter().enumerate() {
            let n = entry.chunks();
            if chunk < first + n {
                let offset = (chunk - first) as u64 * CHUNK as u64;
                let len = (entry.size - offset).min(CHUNK as u64) as u32;
                return Some((file as u32, offset, len));
            }
            first += n;
        }
        None
    }

    pub fn chunk_len(&self, chunk: u32) -> u32 {
        self.locate(chunk).map_or(0, |(_, _, len)| len)
    }
}

pub fn hash_chunk(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}
