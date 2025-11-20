use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::process::Command;

use lzma_rs::xz_decompress;

use xsfx::common::{Trailer, MAGIC, TRAILER_SIZE};

struct TempDir {
    path: std::path::PathBuf,
}

impl TempDir {
    fn new() -> io::Result<Self> {
        let base = env::temp_dir();
        let pid = std::process::id();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        for counter in 0..1000 {
            let dir = base.join(format!("rsfx-{}-{}-{}", pid, timestamp, counter));
            match fs::create_dir(&dir) {
                Ok(_) => return Ok(Self { path: dir }),
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }

        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Unable to create temporary directory",
        ))
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }

    fn close(self) -> io::Result<()> {
        fs::remove_dir_all(&self.path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn main() {
    if let Err(e) = run_stub() {
        eprintln!("SFX stub error: {}", e);
        std::process::exit(1);
    }
}

fn run_stub() -> io::Result<()> {
    let exe_path = env::current_exe()?;
    let mut file = File::open(&exe_path)?;
    let meta = file.metadata()?;
    let total_len = meta.len();

    if total_len < TRAILER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too small to contain trailer",
        ));
    }

    // Read trailer from the end: last 16 bytes
    file.seek(SeekFrom::Start(total_len - TRAILER_SIZE))?;
    let trailer = Trailer::from_reader(&mut file)?;
    if trailer.magic != MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid SFX magic marker",
        ));
    }

    let payload_len = trailer.payload_len;
    if payload_len == 0 || payload_len > total_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid payload length in trailer",
        ));
    }

    let payload_start = total_len - TRAILER_SIZE - payload_len;
    file.seek(SeekFrom::Start(payload_start))?;

    // Limit reader to payload length
    let mut limited_reader = BufReader::new(file.take(payload_len));

    // Create temp dir and write decompressed payload
    let temp_dir = TempDir::new()?;
    let payload_exe_path = temp_dir.path().join("payload_exe");

    let mut out = File::create(&payload_exe_path)?;
    // lzma-rs works on BufRead; use the default decompression options.
    xz_decompress(&mut limited_reader, &mut out)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    out.flush()?;

    // Make it executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&payload_exe_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&payload_exe_path, perms)?;
    }

    // Build args: forward all original CLI args except argv[0]
    let args: Vec<String> = env::args().skip(1).collect();

    // Run the decompressed payload
    let status = Command::new(&payload_exe_path)
        .args(&args)
        .status()?;

    // Best-effort cleanup
    if let Err(err) = temp_dir.close() {
        eprintln!("Warning: failed to remove temp dir: {}", err);
    }

    std::process::exit(status.code().unwrap_or(1));
}
