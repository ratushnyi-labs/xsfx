use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

use xsfx::common::Trailer;
use xsfx::compress::compress_lzma;

mod stub_catalog {
    include!(concat!(env!("OUT_DIR"), "/stub_catalog.rs"));
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args.len() > 5 {
        eprintln!(
            "Usage: {} <input_payload> <output_sfx> [--target <triple>]",
            args[0]
        );
        list_available_stubs();
        std::process::exit(1);
    }

    let payload_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);

    let mut selected_target: Option<String> = None;
    if args.len() > 3 {
        if args.len() == 5 && args[3] == "--target" {
            selected_target = Some(args[4].clone());
        } else {
            eprintln!(
                "Unknown arguments. Usage: {} <input_payload> <output_sfx> [--target <triple>]",
                args[0]
            );
            list_available_stubs();
            std::process::exit(1);
        }
    }

    let target_to_use = selected_target
        .or_else(|| env::var("XSFX_OUT_TARGET").ok())
        .unwrap_or_else(|| stub_catalog::DEFAULT_TARGET.to_string());

    let stub_bytes = match find_stub(&target_to_use) {
        Some(bytes) => bytes,
        None => {
            eprintln!(
                "Requested target '{}' not available in this build.",
                target_to_use
            );
            list_available_stubs();
            std::process::exit(2);
        }
    };

    let payload_bytes = fs::read(&payload_path).map_err(|e| {
        eprintln!("Failed to read payload {}: {}", payload_path.display(), e);
        e
    })?;

    let compressed_payload = compress_lzma(&payload_bytes)?;

    let payload_len = compressed_payload.len() as u64;
    let trailer = Trailer::new(payload_len);
    let trailer_bytes = trailer.to_bytes();

    let mut out = File::create(&output_path).map_err(|e| {
        eprintln!("Failed to create output {}: {}", output_path.display(), e);
        e
    })?;

    out.write_all(stub_bytes)?;
    out.write_all(&compressed_payload)?;
    out.write_all(&trailer_bytes)?;
    out.flush()?;

    println!(
        "Created SFX: {} (target: {}, stub: {} bytes, payload: {} bytes compressed)",
        output_path.display(),
        target_to_use,
        stub_bytes.len(),
        payload_len
    );

    Ok(())
}

fn find_stub(target: &str) -> Option<&'static [u8]> {
    for entry in stub_catalog::STUBS {
        if entry.target == target {
            return Some(entry.bytes);
        }
    }
    None
}

fn list_available_stubs() {
    eprintln!("Available stub targets in this build:");
    for entry in stub_catalog::STUBS {
        let suffix = if entry.target == stub_catalog::DEFAULT_TARGET {
            " (default)"
        } else {
            ""
        };
        eprintln!("  - {}{}", entry.target, suffix);
    }
}
