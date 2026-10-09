//! Dropping metadata from a JPEG without decoding the picture.
//!
//! A JPEG is a list of segments, each a marker and a length, until the
//! image data starts. EXIF (with GPS and camera details), XMP and comments
//! live in their own segments, which are left out of the copy.

pub const START: [u8; 2] = [0xFF, 0xD8];
const START_OF_SCAN: u8 = 0xDA;
const COMMENT: u8 = 0xFE;
/// APP1 to APP15: EXIF, XMP, ICC profiles, maker notes. APP0 (JFIF) stays.
const METADATA_SEGMENTS: std::ops::RangeInclusive<u8> = 0xE1..=0xEF;

/// The JPEG without metadata segments, or `None` when it cannot be read.
pub fn without_metadata(bytes: &[u8]) -> Option<Vec<u8>> {
    if !bytes.starts_with(&START) {
        return None;
    }
    copy_segments(bytes)
}

/// Copies segment after segment, leaving out metadata, up to the image data.
fn copy_segments(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut copy = START.to_vec();
    let mut at = START.len();
    loop {
        let segment = segment_at(bytes, at)?;
        if segment.marker == START_OF_SCAN {
            copy.extend_from_slice(bytes.get(at..)?);
            return Some(copy);
        }
        if keeps(segment.marker) {
            copy.extend_from_slice(bytes.get(at..segment.end)?);
        }
        at = segment.end;
    }
}

struct Segment {
    marker: u8,
    end: usize,
}

fn segment_at(bytes: &[u8], at: usize) -> Option<Segment> {
    let header = bytes.get(at..at + 4)?;
    if header[0] != 0xFF {
        return None;
    }
    let length = usize::from(u16::from_be_bytes([header[2], header[3]]));
    let end = at + 2 + length;
    if length < 2 || end > bytes.len() {
        return None;
    }
    Some(Segment { marker: header[1], end })
}

fn keeps(marker: u8) -> bool {
    marker != COMMENT && !METADATA_SEGMENTS.contains(&marker)
}
