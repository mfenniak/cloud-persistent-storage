#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rusoto;
extern crate aws_instance_metadata;
extern crate chrono;
extern crate serde_yaml;
#[macro_use]
extern crate serde_derive;

mod mkfs;
mod ebs;
mod mount;
mod config;

fn main() {
    env_logger::init().unwrap();

    // FIXME: config: mount point
    // FIXME: config: filesystem type
    // FIXME: windows platform support for mkfs/mount
    // FIXME: allocate volume if find_and_attach fails due to NoVolumesAvailable/AllAttachesFailed

    let block_device = "/dev/xvdh";

    match ebs::find_and_attach_volume(block_device) {
        Ok(_) => info!("attach volume succeeded"),
        Err(e) => {
            error!("attach volume failed: {:?}", e);
            std::process::exit(101);
        }
    }
    // FIXME: poll describe-volumes until .Volumes[0].Attachments[0].State == "attached"


    // FIXME: detect whether device has a filesystem on it
    match mkfs::make_filesystem(block_device) {
        Ok(_) => info!("created filesystem successfully"),
        Err(e) => {
            error!("failed to create filesystem: {:?}", e);
            std::process::exit(102);
        }
    }

    let mount_point = "/mnt/test";
    match std::fs::create_dir_all(mount_point) {
        Ok(_) => info!("created/ensured mount point directory successfully"),
        Err(e) => {
            error!("failed to create mount point directory: {:?}", e);
            std::process::exit(103);
        }
    }

    match mount::mount(block_device, mount_point) {
        Ok(_) => info!("mounted filesystem successfully"),
        Err(e) => {
            error!("failed to mount filesystem: {:?}", e);
            std::process::exit(104);
        }
    }
}

#[cfg(test)]
mod tests {}
