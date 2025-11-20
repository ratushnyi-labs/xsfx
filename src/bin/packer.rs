use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::PathBuf;

use lzma_rs::xz_compress;

use xsfx::common::Trailer;

#[cfg(feature = "native-compress")]
use xz2::write::XzEncoder;

#[cfg(target_os = "macos")]
const EMBEDDED_STUB: &[u8] = include_bytes!(env!("XSFX_STUB_PATH"));
#[cfg(target_os = "linux")]
const EMBEDDED_STUB: &[u8] = include_bytes!(env!("XSFX_STUB_PATH"));
#[cfg(target_os = "windows")]
const EMBEDDED_STUB: &[u8] = include_bytes!(env!("XSFX_STUB_PATH"));


fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!(
            "Usage: {} <input_payload> <output_sfx>",
            args[0]
        );
        std::process::exit(1);
    }

    let payload_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);

    let stub_bytes = EMBEDDED_STUB;

    // Read payload (the app to pack)
    let payload_bytes = fs::read(&payload_path).map_err(|e| {
        eprintln!("Failed to read payload {}: {}", payload_path.display(), e);
        e
    })?;

    // Compress payload using LZMA (lzma-rs)
    let compressed_payload = compress_lzma(&payload_bytes)?;

    let payload_len = compressed_payload.len() as u64;
    let trailer = Trailer::new(payload_len);
    let trailer_bytes = trailer.to_bytes();

    // Write out final SFX: [stub][compressed payload][trailer]
    let mut out = File::create(&output_path).map_err(|e| {
        eprintln!("Failed to create output {}: {}", output_path.display(), e);
        e
    })?;

    out.write_all(&stub_bytes)?;
    out.write_all(&compressed_payload)?;
    out.write_all(&trailer_bytes)?;
    out.flush()?;

    println!(
        "Created SFX: {} (stub: {} bytes, payload: {} bytes compressed)",
        output_path.display(),
        stub_bytes.len(),
        payload_len
    );

    Ok(())
}

fn compress_lzma(data: &[u8]) -> io::Result<Vec<u8>> {
    // Prefer native liblzma (xz2) when available; fallback to pure-Rust lzma-rs.
    #[cfg(feature = "native-compress")]
    {
        let mut encoder = XzEncoder::new(Vec::new(), 9); // level 9 = max compression
        encoder.write_all(data)?;
        encoder.flush()?;
        let compressed = encoder.finish()?;
        return Ok(compressed);
    }

    let mut reader = BufReader::new(io::Cursor::new(data));
    let mut compressed = Vec::new();

    // lzma-rs expects a BufRead; it uses default compression options internally.
    xz_compress(&mut reader, &mut compressed)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(compressed)
}
