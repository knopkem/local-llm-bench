use flate2::write::ZlibEncoder;
use std::io::Write;

fn chunk(data: &[u8], typ: &[u8; 4]) -> Vec<u8> {
    let mut c = Vec::with_capacity(12 + data.len());
    c.extend_from_slice(&(data.len() as u32).to_be_bytes());
    c.extend_from_slice(typ);
    c.extend_from_slice(data);
    let mut crc_buf = Vec::with_capacity(4 + data.len());
    crc_buf.extend_from_slice(typ);
    crc_buf.extend_from_slice(data);
    c.extend_from_slice(&crc32fast::hash(&crc_buf).to_be_bytes());
    c
}

/// Write an RGBA8 buffer as a PNG file (minimal encoder, filter 0 only).
pub fn save_png(path: &str, rgba: &[u8], w: usize, h: usize) -> std::io::Result<()> {
    let mut raw = Vec::with_capacity(h * (1 + w * 4));
    for y in 0..h {
        raw.push(0);
        raw.extend_from_slice(&rgba[y * w * 4..(y + 1) * w * 4]);
    }
    let mut enc = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(&raw)?;
    let idat = enc.finish()?;

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(6); // color type RGBA
    ihdr.push(0); // compression
    ihdr.push(0); // filter
    ihdr.push(0); // interlace

    let mut out = vec![137u8, 80, 78, 71, 13, 10, 26, 10];
    out.extend(chunk(&ihdr, b"IHDR"));
    out.extend(chunk(&idat, b"IDAT"));
    out.extend(chunk(&[], b"IEND"));
    std::fs::write(path, out)
}
