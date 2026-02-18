use std::env;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use xsfx::common::{Trailer, MAGIC, TRAILER_SIZE};
use xsfx::decompress::decompress_payload;

fn main() {
    if run_stub().is_err() {
        let _ = io::Write::write_all(&mut io::stderr(), b"SFX stub error\n");
        std::process::exit(1);
    }
}

fn run_stub() -> io::Result<()> {
    let exe_path = env::current_exe()?;
    let mut file = std::fs::File::open(&exe_path)?;
    let meta = file.metadata()?;
    let total_len = meta.len();

    if total_len < TRAILER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too small to contain trailer",
        ));
    }

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

    let mut limited_reader = BufReader::new(file.take(payload_len));
    let payload = decompress_payload(&mut limited_reader)?;

    let args: Vec<String> = env::args().skip(1).collect();

    let exit_code = exec_payload(&payload, &args, &exe_path)?;

    std::process::exit(exit_code);
}

#[cfg(target_os = "linux")]
fn fd_to_proc_path(fd: i32) -> String {
    let mut buf = *b"/proc/self/fd/0000000000";
    let mut n = fd as u32;
    let mut pos = buf.len();
    loop {
        pos -= 1;
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    let s = &buf[..14]; // "/proc/self/fd/"
    let d = &buf[pos..];
    let mut result = Vec::with_capacity(s.len() + d.len());
    result.extend_from_slice(s);
    result.extend_from_slice(d);
    unsafe { String::from_utf8_unchecked(result) }
}

#[cfg(target_os = "linux")]
fn exec_payload(payload: &[u8], args: &[String], argv0: &Path) -> io::Result<i32> {
    use std::ffi::CString;
    use std::io::Write;
    use std::os::unix::io::FromRawFd;
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    let fd = unsafe {
        let name = CString::new("rsfx-payload").expect("memfd name");
        let res = libc::syscall(libc::SYS_memfd_create, name.as_ptr(), libc::MFD_CLOEXEC);
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        res as i32
    };

    let mut memfd = unsafe { std::fs::File::from_raw_fd(fd) };
    memfd.write_all(payload)?;
    memfd.flush()?;

    let chmod_res = unsafe { libc::fchmod(fd, 0o700) };
    if chmod_res != 0 {
        return Err(io::Error::last_os_error());
    }

    let fd_path = fd_to_proc_path(fd);
    let status = Command::new(&fd_path).arg0(argv0).args(args).status()?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(target_os = "windows")]
fn exec_payload(payload: &[u8], args: &[String], _argv0: &Path) -> io::Result<i32> {
    xsfx::pe_loader::load_and_exec_pe(payload, args)
}

#[cfg(target_os = "macos")]
fn exec_payload(payload: &[u8], args: &[String], _argv0: &Path) -> io::Result<i32> {
    xsfx::macho_loader::load_and_exec_macho(payload, args)
}
