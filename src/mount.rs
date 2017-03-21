use std;
use std::process::{Command, Stdio};

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

pub fn mount() -> Result<(), MountError> {
    // FIXME: config: mount options
    let result = try!(Command::new("/bin/mount")
                          .arg("/dev/xvdh")
                          .arg("/mnt/test")
                          .stdin(Stdio::null())
                          .output());
    if result.status.success() {
        Ok(())
    } else {
        let err_text =
            String::from_utf8(result.stderr).unwrap_or(String::from("unable to decode mount stderr"));
        Err(MountError::ExternalCommandFailed(err_text))
    }
}

#[cfg(test)]
mod tests {}
