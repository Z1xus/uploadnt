use serde::{Deserialize, Serialize};

pub const CHUNK: u32 = 1 << 20;
// a piece plus its framing must fit the 64 KiB data channel message limit
pub const PIECE: u32 = 63 << 10;
pub const MAX_NAME: usize = 1024;
pub const MAX_MIME: usize = 255;
pub const MAX_FILES: usize = 100_000;
// 1 TiB, so chunk numbers fit a u32 and the hash list stays near 32 MiB
pub const MAX_CHUNKS: u32 = 1 << 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub mime: String,
}

impl FileEntry {
    pub fn is_valid(&self) -> bool {
        self.name.len() <= MAX_NAME && self.mime.len() <= MAX_MIME && self.size <= MAX_CHUNKS as u64 * CHUNK as u64
    }

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
        self.files.len() <= MAX_FILES
            && self.hashes.len() <= MAX_CHUNKS as usize
            && self.files.iter().all(FileEntry::is_valid)
            && self.files.iter().map(|f| f.chunks() as u64).sum::<u64>() == self.hashes.len() as u64
    }

    pub fn locate(&self, chunk: u32) -> Option<(u32, u64, u32)> {
        Layout::new(self).locate(self, chunk)
    }

    pub fn chunk_len(&self, chunk: u32) -> u32 {
        self.locate(chunk).map_or(0, |(_, _, len)| len)
    }
}

// first chunk of every file, so a chunk is found by binary search instead of a walk
pub struct Layout {
    starts: Vec<u32>,
    total: u32,
}

impl Layout {
    pub fn new(manifest: &Manifest) -> Self {
        let mut total = 0;
        let starts = manifest
            .files
            .iter()
            .map(|f| {
                let start = total;
                total += f.chunks();
                start
            })
            .collect();
        Self { starts, total }
    }

    pub fn locate(&self, manifest: &Manifest, chunk: u32) -> Option<(u32, u64, u32)> {
        if chunk >= self.total {
            return None;
        }
        let file = self.starts.partition_point(|&first| first <= chunk) - 1;
        let offset = (chunk - self.starts[file]) as u64 * CHUNK as u64;
        let len = (manifest.files[file].size - offset).min(CHUNK as u64) as u32;
        Some((file as u32, offset, len))
    }
}

pub fn hash_chunk(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}
