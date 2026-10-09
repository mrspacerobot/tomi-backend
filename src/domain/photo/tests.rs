use super::*;

/// A JPEG made of the segments that matter here: JFIF, EXIF with a
/// location, a comment, a quantisation table, then the image data.
fn jpeg_with_location() -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xD8];
    bytes.extend([0xFF, 0xE0, 0x00, 0x04, b'J', b'F']);
    bytes.extend([0xFF, 0xE1, 0x00, 0x07, b'G', b'P', b'S', b'4', b'7']);
    bytes.extend([0xFF, 0xFE, 0x00, 0x05, b'h', b'i', b'!']);
    bytes.extend([0xFF, 0xDB, 0x00, 0x03, 0x01]);
    bytes.extend([0xFF, 0xDA, 0x00, 0x02, 0x11, 0x22, 0xFF, 0xD9]);
    bytes
}

fn png_with_text() -> Vec<u8> {
    let chunk = |kind: &[u8; 4], data: &[u8]| {
        let mut bytes = u32::try_from(data.len()).unwrap().to_be_bytes().to_vec();
        bytes.extend(kind);
        bytes.extend(data);
        bytes.extend([0, 0, 0, 0]);
        bytes
    };
    let mut bytes = png::SIGNATURE.to_vec();
    bytes.extend(chunk(b"IHDR", &[1, 2, 3]));
    bytes.extend(chunk(b"tEXt", b"Location"));
    bytes.extend(chunk(b"IDAT", &[9, 9]));
    bytes.extend(chunk(b"IEND", &[]));
    bytes
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

#[test]
fn a_stored_jpeg_loses_its_location_and_comments() {
    let photo = Photo::parse(Some("image/jpeg"), jpeg_with_location()).unwrap();
    let stored = photo.without_metadata().unwrap();
    assert!(!contains(&stored, b"GPS"));
    assert!(!contains(&stored, b"hi!"));
    assert!(contains(&stored, &[0xFF, 0xE0, 0x00, 0x04, b'J', b'F']));
    assert!(stored.ends_with(&[0xFF, 0xDA, 0x00, 0x02, 0x11, 0x22, 0xFF, 0xD9]));
}

#[test]
fn a_stored_png_loses_its_text() {
    let photo = Photo::parse(Some("image/png"), png_with_text()).unwrap();
    let stored = photo.without_metadata().unwrap();
    assert!(!contains(&stored, b"Location"));
    assert!(contains(&stored, b"IDAT"));
    assert!(contains(&stored, b"IEND"));
}

#[test]
fn a_cut_off_photo_is_not_stored() {
    let mut bytes = jpeg_with_location();
    bytes.truncate(12);
    let photo = Photo::parse(Some("image/jpeg"), bytes).unwrap();
    assert_eq!(photo.without_metadata(), Err(PhotoError::Unreadable));
}

#[test]
fn only_jpeg_and_png_of_a_sane_size_are_taken() {
    let refused = [
        (Some("image/heic"), jpeg_with_location(), PhotoError::UnsupportedType),
        (None, jpeg_with_location(), PhotoError::UnsupportedType),
        (Some("image/jpeg"), Vec::new(), PhotoError::Empty),
        (Some("image/png"), jpeg_with_location(), PhotoError::WrongType),
        (Some("image/jpeg"), vec![0xFF; MAX_PHOTO_BYTES + 1], PhotoError::TooLarge),
    ];
    for (content_type, bytes, error) in refused {
        assert_eq!(Photo::parse(content_type, bytes).err(), Some(error));
    }
}
