use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use uploadnt_core::*;

const SIZE: u64 = 1 << 30;
const LANES: usize = 4;

fn chunk_data(chunk: u32, len: u32) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(chunk as u64);
    let mut data = vec![0; len as usize];
    rng.fill(&mut data[..]);
    data
}

fn manifest() -> Manifest {
    let files = vec![
        FileEntry { name: "a.bin".into(), size: SIZE - 12_345, mime: "application/octet-stream".into() },
        FileEntry { name: "b.bin".into(), size: 12_345, mime: "application/octet-stream".into() },
        FileEntry { name: "empty".into(), size: 0, mime: "".into() },
    ];
    let mut manifest = Manifest { files, hashes: vec![] };
    let n: u32 = manifest.files.iter().map(FileEntry::chunks).sum();
    manifest.hashes = (0..n).map(|c| hash_chunk(&chunk_data(c, manifest.chunk_len(c)))).collect();
    manifest
}

fn connect(secret: &Secret, manifest: &Manifest) -> (Sender, Receiver) {
    let (hs, msg_s) = Handshake::start(secret, Role::Sender);
    let (hr, msg_r) = Handshake::start(secret, Role::Receiver);
    let ks = hs.finish(&msg_r).unwrap();
    let kr = hr.finish(&msg_s).unwrap();
    (Sender::new(ks.data(), manifest.clone()), Receiver::new(kr.data()))
}

#[test]
fn moves_a_gigabyte_through_a_hostile_transport() {
    let mut rng = StdRng::seed_from_u64(7);
    let secret = Secret::generate();
    let manifest = manifest();
    let mut have: Option<Vec<u8>> = None;
    let mut resumes = 0;
    let mut now = 0u64;

    'session: loop {
        let (mut sender, mut receiver) = connect(&secret, &manifest);
        let mut to_receiver: Vec<Vec<Vec<u8>>> = vec![vec![]; LANES];
        let mut to_sender: Vec<Vec<u8>> = vec![];
        let mut written_this_session = 0;
        let mut sender_done = false;

        loop {
            now += 1;
            while let Some(msg) = sender.poll_transmit() {
                // reliable data channels lose nothing, but a dying path loses pieces
                if msg.len() > 1024 && rng.gen_bool(0.002) {
                    continue;
                }
                to_receiver[rng.gen_range(0..LANES)].push(msg);
            }
            while let Some(msg) = receiver.poll_transmit() {
                to_sender.push(msg);
            }

            for msg in to_sender.drain(..) {
                sender.handle(&msg).unwrap();
            }
            while let Some(event) = sender.poll_event() {
                match event {
                    Event::Read { chunk, len, .. } => sender.provide(chunk, &chunk_data(chunk, len)),
                    Event::Done => sender_done = true,
                    Event::Progress { .. } => {}
                    other => panic!("sender got {other:?}"),
                }
            }

            let idle = to_receiver.iter().all(Vec::is_empty);
            for lane in &mut to_receiver {
                for msg in lane.drain(..).rev() {
                    receiver.handle(&msg, now).unwrap();
                }
            }
            while let Some(event) = receiver.poll_event() {
                match event {
                    Event::Manifest { manifest: m } => {
                        assert_eq!(m.root(), manifest.root());
                        receiver.accept(have.as_deref(), now);
                    }
                    Event::Write { chunk, offset, data, .. } => {
                        assert_eq!(data, chunk_data(chunk, manifest.chunk_len(chunk)));
                        assert_eq!(Some(offset), manifest.locate(chunk).map(|(_, o, _)| o));
                        receiver.written(chunk, now);
                        written_this_session += 1;
                    }
                    Event::Progress { .. } => {}
                    Event::Done => receiver.complete(),
                    other => panic!("receiver got {other:?}"),
                }
            }

            if sender_done {
                assert!(receiver.have().is_full());
                break 'session;
            }
            if resumes < 3 && written_this_session >= 200 {
                have = Some(receiver.have().bytes().to_vec());
                resumes += 1;
                continue 'session;
            }
            if idle {
                now += 10_000;
                receiver.tick(now);
            }
        }
    }
    assert_eq!(resumes, 3);
}

#[test]
fn wrong_secret_cannot_read() {
    let manifest = manifest_small();
    let (hs, msg_s) = Handshake::start(&Secret::generate(), Role::Sender);
    let (hr, msg_r) = Handshake::start(&Secret::generate(), Role::Receiver);
    let mut sender = Sender::new(hs.finish(&msg_r).unwrap().data(), manifest);
    let mut receiver = Receiver::new(hr.finish(&msg_s).unwrap().data());
    let msg = sender.poll_transmit().unwrap();
    assert_eq!(receiver.handle(&msg, 0), Err(Error::Decrypt));
}

#[test]
fn link_round_trip() {
    let secret = Secret::generate();
    assert_eq!(secret.encode().len(), 26);
    let link = Link::parse(&format!("https://uploadnt.com/#{}@lobby.example", secret.encode())).unwrap();
    assert!(link.secret == secret);
    assert_eq!(link.lobby.as_deref(), Some("lobby.example"));
    assert!(Link::parse(&format!("nope://{}", secret.encode())).unwrap().secret == secret);
    assert!(Link::parse("nope://short").is_none());
}

fn manifest_small() -> Manifest {
    let data = chunk_data(0, 10);
    Manifest { files: vec![FileEntry { name: "x".into(), size: 10, mime: "".into() }], hashes: vec![hash_chunk(&data)] }
}
