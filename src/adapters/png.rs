//! PNG writer: 8-bit RGB, no filtering, stored deflate blocks.
//!
//! Std only — the chunk CRCs, the zlib wrapper, and the Adler-32 are
//! hand-rolled. This is the screenshot adapter's last step; the
//! rasterizer that feeds it is `adapters::render`. The stored-block
//! choice keeps the encoder small and deterministic; screenshots are
//! written once, not streamed.

/// Encodes an RGB image as a PNG. `pixels` is row-major, top-left
/// first, one `[r, g, b]` per pixel.
pub fn encode_rgb(width: u32, height: u32, pixels: &[[u8; 3]]) -> Vec<u8> {
    assert_eq!(pixels.len(), (width * height) as usize, "pixel count must match the viewport");
    let mut out = Vec::new();
    out.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &zlib_stored(&scanlines(width, height, pixels)));
    chunk(&mut out, b"IEND", &[]);
    out
}

/// One filter byte (0, none) then the RGB bytes of each row.
fn scanlines(width: u32, height: u32, pixels: &[[u8; 3]]) -> Vec<u8> {
    let row = width as usize * 3;
    let mut raw = Vec::with_capacity((row + 1) * height as usize);
    for y in 0..height as usize {
        raw.push(0);
        for x in 0..width as usize {
            raw.extend_from_slice(&pixels[y * width as usize + x]);
        }
    }
    raw
}

/// A zlib stream of stored (uncompressed) deflate blocks.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut i = 0;
    while i < data.len() {
        let end = (i + 65_535).min(data.len());
        let len = (end - i) as u16;
        out.push(if end == data.len() { 1 } else { 0 });
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(&data[i..end]);
        i = end;
    }
    if data.is_empty() {
        out.extend_from_slice(&[1, 0, 0, 0xff, 0xff]);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(kind);
    crc_input.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + byte as u32) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_crc_of_the_png_signature_is_known() {
        // "IEND" with no data has the canonical CRC 0xAE426082.
        assert_eq!(crc32(b"IEND"), 0xae42_6082);
    }

    #[test]
    fn adler_of_an_empty_buffer_is_one() {
        assert_eq!(adler32(&[]), 1);
    }
}
