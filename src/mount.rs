use std;
use std::process::{Command, Stdio};
use config::Mount;

#[derive(Debug)]
pub enum MountError {
    SpawnFailed(std::io::Error),
    ExternalCommandFailed(String),
}

impl From<std::io::Error> for MountError {
    fn from(err: std::io::Error) -> MountError {
        MountError::SpawnFailed(err)
    }
}

pub fn mount(config: &Mount, block_device: &str) -> Result<(), MountError> {
    let mut cmd = Command::new("/bin/mount");
    cmd.arg(block_device);
    cmd.arg(config.target.to_owned());
    trace!("invoking mount: {:?}", cmd);
    let result = try!(cmd.stdin(Stdio::null()).output());
    if result.status.success() {
        trace!("external mount command succeeded");
        Ok(())
    } else {
        let err_text =
            String::from_utf8(result.stderr).unwrap_or_else(|_| String::from("unable to decode mount stderr"));
        Err(MountError::ExternalCommandFailed(err_text))
    }
}

#[cfg(test)]
mod tests {}
