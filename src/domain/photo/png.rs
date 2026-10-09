//! Dropping metadata from a PNG without decoding the picture.
//!
//! A PNG is a signature and a list of chunks. Text, EXIF and time chunks
//! carry metadata and are left out of the copy; the picture's own chunks
//! stay as they are.

pub const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const METADATA_CHUNKS: [&[u8; 4]; 5] = [b"tEXt", b"iTXt", b"zTXt", b"eXIf", b"tIME"];
const LAST_CHUNK: &[u8; 4] = b"IEND";
/// Length, type and checksum around each chunk's data.
const CHUNK_OVERHEAD: usize = 12;

/// The PNG without metadata chunks, or `None` when it cannot be read.
pub fn without_metadata(bytes: &[u8]) -> Option<Vec<u8>> {
    if !bytes.starts_with(&SIGNATURE) {
        return None;
    }
    let mut copy = SIGNATURE.to_vec();
    let mut at = SIGNATURE.len();
    loop {
        let chunk = chunk_at(bytes, at)?;
        if !METADATA_CHUNKS.contains(&&chunk.kind) {
            copy.extend_from_slice(bytes.get(at..chunk.end)?);
        }
        if &chunk.kind == LAST_CHUNK {
            return Some(copy);
        }
        at = chunk.end;
    }
}

struct Chunk {
    kind: [u8; 4],
    end: usize,
}

fn chunk_at(bytes: &[u8], at: usize) -> Option<Chunk> {
    let header = bytes.get(at..at + 8)?;
    let length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
    let end = at + CHUNK_OVERHEAD + usize::try_from(length).ok()?;
    if end > bytes.len() {
        return None;
    }
    let kind = [header[4], header[5], header[6], header[7]];
    Some(Chunk { kind, end })
}
