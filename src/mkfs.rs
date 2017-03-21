use std;
use std::process::{Command, Stdio};

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

pub fn make_filesystem() -> Result<(), MakeFilesystemError> {
    // FIXME: config: mkfs options
    let result = try!(Command::new("/sbin/mkfs")
                          .arg("-t")
                          .arg("ext4")
                          .arg("-m")
                          .arg("0")
                          .arg("/dev/xvdh")
                          .stdin(Stdio::null())
                          .output());
    if result.status.success() {
        Ok(())
    } else {
        let err_text =
            String::from_utf8(result.stderr).unwrap_or(String::from("unable to decode mkfs stderr"));
        Err(MakeFilesystemError::ExternalCommandFailed(err_text))
    }
}

#[cfg(test)]
mod tests {}
