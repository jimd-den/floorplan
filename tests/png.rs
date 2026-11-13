//! The PNG writer: signature, header, and a stored-deflate round-trip.
//!
//! The decoder here is deliberately dumb: it only understands the
//! stored blocks this writer emits, which is enough to prove the
//! pixels survive the trip without pulling in a compression crate.

use floorplan::adapters::png::encode_rgb;

const WIDTH: u32 = 2;
const HEIGHT: u32 = 2;

fn pixels() -> Vec<[u8; 3]> {
    vec![[255, 0, 0], [0, 255, 0], [0, 0, 255], [10, 20, 30]]
}

/// The body of the first chunk with `kind`, or an empty vec.
fn chunk_body(png: &[u8], kind: &[u8; 4]) -> Vec<u8> {
    let mut i = 8;
    while i + 12 <= png.len() {
        let len = u32::from_be_bytes(png[i..i + 4].try_into().unwrap()) as usize;
        if &png[i + 4..i + 8] == kind {
            return png[i + 8..i + 8 + len].to_vec();
        }
        i += 12 + len;
    }
    Vec::new()
}

/// Inflates the stored deflate blocks of a zlib stream.
fn inflate_stored(zlib: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 2;
    loop {
        let last = zlib[i] & 1 == 1;
        let len = u16::from_le_bytes([zlib[i + 1], zlib[i + 2]]) as usize;
        out.extend_from_slice(&zlib[i + 5..i + 5 + len]);
        i += 5 + len;
        if last {
            return out;
        }
    }
}

#[test]
fn the_signature_and_header_describe_an_rgb_image() {
    let png = encode_rgb(WIDTH, HEIGHT, &pixels());

    assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
    let ihdr = chunk_body(&png, b"IHDR");
    assert_eq!(&ihdr[0..4], &WIDTH.to_be_bytes(), "width");
    assert_eq!(&ihdr[4..8], &HEIGHT.to_be_bytes(), "height");
    assert_eq!(ihdr[8], 8, "bit depth");
    assert_eq!(ihdr[9], 2, "colour type: truecolour");
    let tail = &png[png.len() - 8..];
    assert_eq!(&tail[..4], b"IEND", "last chunk is IEND");
    assert_eq!(&tail[4..], [0xae, 0x42, 0x60, 0x82], "IEND CRC");
}

#[test]
fn the_pixels_survive_the_stored_deflate() {
    let png = encode_rgb(WIDTH, HEIGHT, &pixels());

    let raw = inflate_stored(&chunk_body(&png, b"IDAT"));

    let mut expected = Vec::new();
    for row in pixels().chunks(WIDTH as usize) {
        expected.push(0);
        for p in row {
            expected.extend_from_slice(p);
        }
    }
    assert_eq!(raw, expected, "one filter byte then RGB per row");
}

#[test]
fn a_one_pixel_image_is_valid() {
    let png = encode_rgb(1, 1, &[[1, 2, 3]]);

    assert_eq!(&chunk_body(&png, b"IHDR")[0..4], &1u32.to_be_bytes());
    assert_eq!(inflate_stored(&chunk_body(&png, b"IDAT")), vec![0, 1, 2, 3]);
}
