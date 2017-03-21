#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rusoto;
extern crate aws_instance_metadata;
extern crate chrono;

mod mkfs;
mod ebs;
mod mount;

fn main() {
    env_logger::init().unwrap();

    // FIXME: config: mount point
    // FIXME: config: filesystem type
    // FIXME: windows platform support for mkfs/mount
    // FIXME: allocate volume if find_and_attach fails due to NoVolumesAvailable/AllAttachesFailed

    match ebs::find_and_attach_volume() {
        Ok(_) => info!("attach volume succeeded"),
        Err(e) => {
            error!("attach volume failed: {:?}", e);
            std::process::exit(101);
        }
    }
    // FIXME: poll describe-volumes until .Volumes[0].Attachments[0].State == "attached"

    // FIXME: detect whether device has a filesystem on it
    match mkfs::make_filesystem() {
        Ok(_) => info!("created filesystem successfully"),
        Err(e) => {
            error!("failed to create filesystem: {:?}", e);
            std::process::exit(102);
        }
    }

    // FIXME: rust equivalent to mkdir -p on the mount target

    match mount::mount() {
        Ok(_) => info!("mounted filesystem successfully"),
        Err(e) => {
            error!("failed to mount filesystem: {:?}", e);
            std::process::exit(103);
        }
    }
}

#[cfg(test)]
mod tests {}
