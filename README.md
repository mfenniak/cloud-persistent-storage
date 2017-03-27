# Cloud Persistent Storage

## Current Limitations

- Only works with ext4 filesystems.


## Dream Feature List

- Support multiple cloud providers
    - Amazon Web Services
    - Google Cloud Platform
    - Azure
- Creation options for persistent volume:
    - Disk size
    - Volume type (eg. EBS -> gp2, io1, st1, sc1)
    - Filesystem creation options
    - Create from snapshot, rather than creating empty volume
- Options for mounting persistent volume:
    - Location
    - Mount options
- Support for different attachment strategies
    - Attach any storage available in this AZ, or create one if none is available.
    - Attach any storage available for this autoscaling group, or, create one if none is available.
    - Auto-incrementing strategy; each machine in an autoscaling group is given a number, starting at 1, and incrementing for every *running* machine in the ASG.  If this machine is identified as "1", the volume for "1" is attached.  (If the volume for "1" is not in the correct availability zone, it is {snapshotted and copied to this AZ, or, the identification for "1" is revoked}).
- Some attachment strategies might require an external cluster coordinator; support for:
    - Consul
    - Zookeeper
    - Etcd
- Provide single executable download for multiple platforms
    - Linux
    - Windows
- Instructions for use should include signature/hash verification to ensure expected binaries are installed
- Documentation should show how to use this tool in an initdata script (AWS)
    - Linux
    - Windows
- Configuration via;
    - Command-line options
    - YAML file
- Logging
- Support for resizing volumes if configuration changes
    - eg. with AWS EBS, volume resize, filesystem resize to match, then mount

## AWS Without External Coordinator

- Optimistic attach:
    - DescribeVolumes for all volumes in correct {region, tag, availability-zone, status=available}
    - Iterate and attach until we get success
    - Create volume otherwise
    - (We can't really snapshot/create an existing volume here because there'd be no safe way to "lock" it; multiple machine startup could all get the same persistent volume)
- Slow consensus:
    - Find volume I'd like to own
        - If volume has been previously tagged as owned, but it is still available, ignore the tag after 180s (lock timeout)
    - Tag volume with {Token: ...uuid..., Timestamp: ...ts...}
    - Wait 30 seconds (hopeful consensus timeout)
    - Ensure volume has tags {Token: ...uuid..., Timestamp: ...ts...} as expected; it is now ours
    - Only give up and create a volume after all volumes are in-use

## AWS Development Environment

Based off Ubuntu 16.04, t2.medium:

```
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev
curl https://sh.rustup.rs -sSf | sh -- -y
source $HOME/.cargo/env
git clone https://github.com/mfenniak/cloud-persistent-storage.git
cd cloud-persistent-storage
cargo build
export RUST_LOG=cloud_persistent_storage
```
