use std;
use std::process::{Command, Stdio};
use std::fs::File;
use std::io::Read;
use config::FileSystem;

#[derive(Debug)]
pub enum MakeFilesystemError {
    SpawnFailed(std::io::Error),
    ExternalCommandFailed(String),
}

impl From<std::io::Error> for MakeFilesystemError {
    fn from(err: std::io::Error) -> MakeFilesystemError {
        MakeFilesystemError::SpawnFailed(err)
    }
}

pub fn make_filesystem(config: &FileSystem, block_device: &str) -> Result<(), MakeFilesystemError> {
    let mut cmd = Command::new("/sbin/mkfs");
    for arg in &config.mkfs {
        cmd.arg(arg);
    }
    cmd.arg(block_device);
    trace!("invoking mkfs: {:?}", cmd);

    let result = try!(cmd.stdin(Stdio::null()).output());
    if result.status.success() {
        trace!("external mkfs command succeeded");
        Ok(())
    } else {
        let err_text =
            String::from_utf8(result.stderr).unwrap_or_else(|_| String::from("unable to decode mkfs stderr"));
        Err(MakeFilesystemError::ExternalCommandFailed(err_text))
    }
}

pub fn filesystem_exists(block_device: &str) -> Result<bool, MakeFilesystemError> {
    let mut buf = [0; 2048];
    let mut file = File::open(block_device)?;
    let bytes_read = file.read(&mut buf)?;
    if bytes_read < 2048 {
        Ok(false)
    } else if buf[0x438] == 0x53 && buf[0x439] == 0xEF {
        // ext2/3/4 filesystem
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {}
