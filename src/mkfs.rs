use std;
use std::process::{Command, Stdio};
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
            String::from_utf8(result.stderr).unwrap_or(String::from("unable to decode mkfs stderr"));
        Err(MakeFilesystemError::ExternalCommandFailed(err_text))
    }
}

#[cfg(test)]
mod tests {}
