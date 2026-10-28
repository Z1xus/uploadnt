use std::collections::{HashMap, HashSet, VecDeque};

use crate::{Bitmap, Channel, Error, FileEntry, Frame, Manifest, PIECE, VERSION, hash_chunk};

const MIN_WINDOW: usize = 4;
const MAX_WINDOW: usize = 32;
const STALL_MS: u64 = 10_000;
// keeps files and hashes frames under the 64 KiB data channel message limit
const FILES_PER_FRAME: usize = 40;
const HASHES_PER_FRAME: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Manifest { manifest: Manifest },
    Read { chunk: u32, file: u32, offset: u64, len: u32 },
    Write { chunk: u32, file: u32, offset: u64, data: Vec<u8> },
    Progress { bytes: u64, total: u64 },
    Done,
    Closed { reason: String },
}

struct Link {
    channel: Channel,
    out: VecDeque<Vec<u8>>,
    events: VecDeque<Event>,
}

impl Link {
    fn new(channel: Channel) -> Self {
        let mut link = Self { channel, out: VecDeque::new(), events: VecDeque::new() };
        link.send(&Frame::Hello { version: VERSION });
        link
    }

    fn send(&mut self, frame: &Frame) {
        let plain = postcard::to_allocvec(frame).unwrap();
        self.out.push_back(self.channel.seal(&plain));
    }

    fn open(&self, msg: &[u8]) -> Result<Frame, Error> {
        postcard::from_bytes(&self.channel.open(msg)?).map_err(|_| Error::Decode)
    }

    fn progress(&mut self, manifest: &Manifest, have: &Bitmap) {
        let bytes = (0..have.len()).filter(|&i| have.get(i)).map(|i| manifest.chunk_len(i) as u64).sum();
        self.events.push_back(Event::Progress { bytes, total: manifest.size() });
    }
}

fn check_version(version: u16) -> Result<(), Error> {
    if version == VERSION { Ok(()) } else { Err(Error::Version) }
}

pub struct Sender {
    link: Link,
    manifest: Manifest,
    root: [u8; 32],
    acked: Bitmap,
    reading: HashSet<u32>,
}

impl Sender {
    pub fn new(channel: Channel, manifest: Manifest) -> Self {
        let mut link = Link::new(channel);
        let count = manifest.files.len() as u32;
        for (i, files) in manifest.files.chunks(FILES_PER_FRAME).enumerate() {
            link.send(&Frame::Files { count, start: (i * FILES_PER_FRAME) as u32, files: files.to_vec() });
        }
        for (i, hashes) in manifest.hashes.chunks(HASHES_PER_FRAME).enumerate() {
            link.send(&Frame::Hashes { start: (i * HASHES_PER_FRAME) as u32, hashes: hashes.to_vec() });
        }
        Self { link, root: manifest.root(), acked: Bitmap::new(manifest.chunks()), manifest, reading: HashSet::new() }
    }

    pub fn handle(&mut self, msg: &[u8]) -> Result<(), Error> {
        match self.link.open(msg)? {
            Frame::Hello { version } => check_version(version)?,
            Frame::Have(bits) => {
                self.acked = Bitmap::from_bytes(&bits, self.manifest.chunks());
                self.link.progress(&self.manifest, &self.acked);
            }
            Frame::Want(chunks) => {
                for chunk in chunks {
                    let Some((file, offset, len)) = self.manifest.locate(chunk) else {
                        return Err(Error::Protocol);
                    };
                    if self.reading.insert(chunk) {
                        self.link.events.push_back(Event::Read { chunk, file, offset, len });
                    }
                }
            }
            Frame::Ack(chunk) => {
                self.acked.set(chunk);
                self.link.progress(&self.manifest, &self.acked);
            }
            Frame::Done(root) if root == self.root => self.link.events.push_back(Event::Done),
            Frame::Bye(reason) => self.link.events.push_back(Event::Closed { reason }),
            _ => return Err(Error::Protocol),
        }
        Ok(())
    }

    pub fn provide(&mut self, chunk: u32, data: &[u8]) {
        if !self.reading.remove(&chunk) {
            return;
        }
        for (i, piece) in data.chunks(PIECE as usize).enumerate() {
            let offset = i as u32 * PIECE;
            self.link.send(&Frame::Piece { chunk, offset, data: piece.to_vec() });
        }
    }

    pub fn close(&mut self, reason: &str) {
        self.link.send(&Frame::Bye(reason.into()));
    }

    pub fn poll_transmit(&mut self) -> Option<Vec<u8>> {
        self.link.out.pop_front()
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        self.link.events.pop_front()
    }
}

struct Parts<T>(Vec<(u32, Vec<T>)>);

impl<T> Parts<T> {
    fn add(&mut self, start: u32, items: Vec<T>) {
        if !self.0.iter().any(|(s, _)| *s == start) {
            self.0.push((start, items));
        }
    }

    fn len(&self) -> usize {
        self.0.iter().map(|(_, items)| items.len()).sum()
    }

    fn items(&self) -> impl Iterator<Item = &T> {
        self.0.iter().flat_map(|(_, items)| items)
    }

    fn take(&mut self) -> Result<Vec<T>, Error> {
        self.0.sort_by_key(|(start, _)| *start);
        let mut all = Vec::with_capacity(self.len());
        for (start, items) in self.0.drain(..) {
            if start as usize != all.len() {
                return Err(Error::Protocol);
            }
            all.extend(items);
        }
        Ok(all)
    }
}

struct Partial {
    data: Vec<u8>,
    pieces: Bitmap,
}

pub struct Receiver {
    link: Link,
    file_count: Option<u32>,
    files: Parts<FileEntry>,
    hashes: Parts<[u8; 32]>,
    manifest: Option<Manifest>,
    have: Bitmap,
    inflight: HashMap<u32, Partial>,
    writing: HashSet<u32>,
    window: usize,
    max_window: usize,
    last_piece: u64,
    accepted: bool,
    done: bool,
}

impl Receiver {
    pub fn new(channel: Channel) -> Self {
        Self {
            link: Link::new(channel),
            file_count: None,
            files: Parts(Vec::new()),
            hashes: Parts(Vec::new()),
            manifest: None,
            have: Bitmap::new(0),
            inflight: HashMap::new(),
            writing: HashSet::new(),
            window: MIN_WINDOW,
            max_window: MAX_WINDOW,
            last_piece: 0,
            accepted: false,
            done: false,
        }
    }

    pub fn manifest(&self) -> Option<&Manifest> {
        self.manifest.as_ref()
    }

    pub fn have(&self) -> &Bitmap {
        &self.have
    }

    pub fn handle(&mut self, msg: &[u8], now: u64) -> Result<(), Error> {
        match self.link.open(msg)? {
            Frame::Hello { version } => check_version(version)?,
            Frame::Files { count, start, files } if self.manifest.is_none() => {
                if *self.file_count.get_or_insert(count) != count {
                    return Err(Error::Protocol);
                }
                self.files.add(start, files);
                self.assemble()?;
            }
            Frame::Hashes { start, hashes } if self.manifest.is_none() => {
                self.hashes.add(start, hashes);
                self.assemble()?;
            }
            Frame::Piece { chunk, offset, data } => self.piece(chunk, offset, &data, now)?,
            Frame::Bye(reason) => self.link.events.push_back(Event::Closed { reason }),
            _ => return Err(Error::Protocol),
        }
        Ok(())
    }

    pub fn accept(&mut self, have: Option<&[u8]>, now: u64) {
        let Some(manifest) = &self.manifest else { return };
        if let Some(bits) = have {
            self.have = Bitmap::from_bytes(bits, manifest.chunks());
        }
        self.accepted = true;
        self.link.send(&Frame::Have(self.have.bytes().to_vec()));
        self.link.progress(manifest, &self.have);
        self.fill(now);
    }

    pub fn written(&mut self, chunk: u32, now: u64) {
        if !self.writing.remove(&chunk) {
            return;
        }
        self.have.set(chunk);
        self.link.send(&Frame::Ack(chunk));
        let manifest = self.manifest.as_ref().unwrap();
        self.link.progress(manifest, &self.have);
        if self.writing.is_empty() {
            self.window = (self.window + 1).min(self.max_window);
        }
        self.fill(now);
    }

    pub fn tick(&mut self, now: u64) {
        if !self.inflight.is_empty() && now.saturating_sub(self.last_piece) >= STALL_MS {
            self.inflight.clear();
            self.window = MIN_WINDOW;
            self.fill(now);
        }
    }

    pub fn set_max_window(&mut self, chunks: usize) {
        self.max_window = chunks.clamp(MIN_WINDOW, MAX_WINDOW);
        self.window = self.window.min(self.max_window);
    }

    pub fn complete(&mut self) {
        if let Some(manifest) = self.manifest.as_ref().filter(|_| self.done) {
            self.link.send(&Frame::Done(manifest.root()));
        }
    }

    pub fn close(&mut self, reason: &str) {
        self.link.send(&Frame::Bye(reason.into()));
    }

    pub fn poll_transmit(&mut self) -> Option<Vec<u8>> {
        self.link.out.pop_front()
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        self.link.events.pop_front()
    }

    fn assemble(&mut self) -> Result<(), Error> {
        let Some(count) = self.file_count else { return Ok(()) };
        if self.files.len() > count as usize {
            return Err(Error::Protocol);
        }
        if self.files.len() < count as usize {
            return Ok(());
        }
        let total = self.files.items().map(FileEntry::chunks).sum::<u32>() as usize;
        if self.hashes.len() > total {
            return Err(Error::Protocol);
        }
        if self.hashes.len() < total {
            return Ok(());
        }
        let manifest = Manifest { files: self.files.take()?, hashes: self.hashes.take()? };
        if !manifest.is_valid() {
            return Err(Error::Protocol);
        }
        self.have = Bitmap::new(manifest.chunks());
        self.link.events.push_back(Event::Manifest { manifest: manifest.clone() });
        self.manifest = Some(manifest);
        Ok(())
    }

    fn piece(&mut self, chunk: u32, offset: u32, data: &[u8], now: u64) -> Result<(), Error> {
        let Some(partial) = self.inflight.get_mut(&chunk) else { return Ok(()) };
        let end = offset as usize + data.len();
        if !offset.is_multiple_of(PIECE) || end > partial.data.len() {
            return Err(Error::Protocol);
        }
        partial.data[offset as usize..end].copy_from_slice(data);
        partial.pieces.set(offset / PIECE);
        self.last_piece = now;
        if !partial.pieces.is_full() {
            return Ok(());
        }
        let partial = self.inflight.remove(&chunk).unwrap();
        let manifest = self.manifest.as_ref().unwrap();
        if hash_chunk(&partial.data) == manifest.hashes[chunk as usize] {
            let (file, offset, _) = manifest.locate(chunk).unwrap();
            self.writing.insert(chunk);
            self.link.events.push_back(Event::Write { chunk, file, offset, data: partial.data });
        } else {
            self.fill(now);
        }
        Ok(())
    }

    fn fill(&mut self, now: u64) {
        let Some(manifest) = &self.manifest else { return };
        if !self.accepted || self.done {
            return;
        }
        if self.have.is_full() {
            self.done = true;
            self.link.events.push_back(Event::Done);
            return;
        }
        let busy = self.inflight.len() + self.writing.len();
        let want: Vec<u32> = self
            .have
            .missing()
            .filter(|c| !self.inflight.contains_key(c) && !self.writing.contains(c))
            .take(self.window.saturating_sub(busy))
            .collect();
        if want.is_empty() {
            return;
        }
        if self.inflight.is_empty() {
            self.last_piece = now;
        }
        for &chunk in &want {
            let len = manifest.chunk_len(chunk);
            let partial = Partial { data: vec![0; len as usize], pieces: Bitmap::new(len.div_ceil(PIECE)) };
            self.inflight.insert(chunk, partial);
        }
        self.link.send(&Frame::Want(want));
    }
}
