use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::process::Command;

#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

use lzma_rs::xz_decompress;

use xsfx::common::{Trailer, MAGIC, TRAILER_SIZE};

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

    // Limit reader to payload length, then decompress into memory.
    let mut limited_reader = BufReader::new(file.take(payload_len));
    let payload = decompress_payload(&mut limited_reader)?;

    // Build args: forward all original CLI args except argv[0]
    let args: Vec<String> = env::args().skip(1).collect();

    let exit_code = exec_payload(&payload, &args, &exe_path)?;

    std::process::exit(exit_code);
}

fn decompress_payload<R: io::BufRead>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut payload = Vec::new();
    // lzma-rs works on BufRead; use the default decompression options.
    xz_decompress(reader, &mut payload).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(payload)
}

#[cfg(target_os = "linux")]
fn exec_payload(payload: &[u8], args: &[String], argv0: &std::path::Path) -> io::Result<i32> {
    exec_payload_memfd(payload, args, argv0).or_else(|memfd_err| {
        eprintln!(
            "memfd execution failed (falling back to temp file): {}",
            memfd_err
        );
        exec_payload_tempfile(payload, args, argv0)
    })
}

#[cfg(not(target_os = "linux"))]
fn exec_payload(payload: &[u8], args: &[String], argv0: &std::path::Path) -> io::Result<i32> {
    exec_payload_tempfile(payload, args, argv0)
}

#[cfg(target_os = "linux")]
fn exec_payload_memfd(payload: &[u8], args: &[String], argv0: &std::path::Path) -> io::Result<i32> {
    use std::ffi::CString;
    use std::os::unix::io::FromRawFd;

    // Create an anonymous in-memory file.
    let fd = unsafe {
        let name = CString::new("rsfx-payload").expect("memfd name");
        let res = libc::syscall(libc::SYS_memfd_create, name.as_ptr(), libc::MFD_CLOEXEC);
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        res as i32
    };

    let mut memfd = unsafe { File::from_raw_fd(fd) };
    memfd.write_all(payload)?;
    memfd.flush()?;

    // Ensure executable permissions.
    let chmod_res = unsafe { libc::fchmod(fd, 0o700) };
    if chmod_res != 0 {
        return Err(io::Error::last_os_error());
    }

    let fd_path = format!("/proc/self/fd/{}", fd);
    let status = Command::new(&fd_path).arg0(argv0).args(args).status()?;
    Ok(status.code().unwrap_or(1))
}

// Fallback path used on non-Linux targets (or if memfd fails): write to a temp file.
fn exec_payload_tempfile(
    payload: &[u8],
    args: &[String],
    argv0: &std::path::Path,
) -> io::Result<i32> {
    let temp_file = TempFile::new(payload)?;
    let status = Command::new(&temp_file.path)
        .arg0(argv0)
        .args(args)
        .status()?;
    Ok(status.code().unwrap_or(1))
}

struct TempFile {
    path: std::path::PathBuf,
}

impl TempFile {
    fn new(contents: &[u8]) -> io::Result<Self> {
        let base = env::temp_dir();
        let pid = std::process::id();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        for counter in 0..1000 {
            let path = base.join(format!("rsfx-{}-{}-{}", pid, timestamp, counter));
            match File::create(&path) {
                Ok(mut f) => {
                    f.write_all(contents)?;
                    f.flush()?;
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = f.metadata()?.permissions();
                        perms.set_mode(0o755);
                        fs::set_permissions(&path, perms)?;
                    }
                    return Ok(Self { path });
                }
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }

        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Unable to create payload file",
        ))
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
