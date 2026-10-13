use super::*;

/// The first bytes of a JPEG and a PNG: all the checks here look at.
fn jpeg() -> Vec<u8> {
    vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x04, b'J', b'F', 0xFF, 0xD9]
}

fn png() -> Vec<u8> {
    let mut bytes = PNG_SIGNATURE.to_vec();
    bytes.extend(b"IHDR");
    bytes
}

#[test]
fn a_photo_is_kept_exactly_as_it_arrived() {
    let photo = Photo::parse(Some("image/jpeg"), jpeg()).unwrap();
    assert_eq!(photo.bytes, jpeg());
    let photo = Photo::parse(Some("image/png"), png()).unwrap();
    assert_eq!(photo.bytes, png());
}

#[test]
fn a_content_type_names_the_kind_the_file_ending_and_the_mime_type() {
    let jpeg = ImageKind::from_content_type(" image/jpeg ").unwrap();
    assert_eq!((jpeg.extension(), jpeg.mime_type()), ("jpg", "image/jpeg"));
    let png = ImageKind::from_content_type("image/png").unwrap();
    assert_eq!((png.extension(), png.mime_type()), ("png", "image/png"));
}

#[test]
fn only_jpeg_and_png_of_a_sane_size_are_taken() {
    let refused = [
        (Some("image/heic"), jpeg(), PhotoError::UnsupportedType),
        (None, jpeg(), PhotoError::UnsupportedType),
        (Some("image/jpeg"), Vec::new(), PhotoError::Empty),
        (Some("image/png"), jpeg(), PhotoError::WrongType),
        (Some("image/jpeg"), png(), PhotoError::WrongType),
        (Some("image/jpeg"), vec![0xFF; MAX_PHOTO_BYTES + 1], PhotoError::TooLarge),
    ];
    for (content_type, bytes, error) in refused {
        assert_eq!(Photo::parse(content_type, bytes).err(), Some(error));
    }
}
