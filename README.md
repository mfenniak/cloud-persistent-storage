# Cloud Persistent Storage

cloud-persistent-storage is a tool desired to attach persistent storage devices to cloud servers that are configured to auto-scale.

Currently it supports attaching AWS EBS volumes to Linux AWS EC2 servers running in an autoscaling group.  In the future, I'd love to support more cloud platforms, more server platforms, and more provisioning strategies.

Here's how it works:

- When your server starts up, you run the cloud-persistent-storage with a simple YAML configuration file.
- cloud-persistent-storage searches for existing AWS EBS volumes that match its configuration, and are available to be attached to this EC2 instance.
    - If a volume is found, it is attached.
    - If no volume is found, it creates one based upon its configuration.
- After the volume is attached, it detects whether a filesystem exists on the volume.  If none is found, one is initialized.
- The filesystem is mounted at a configured mount point.

Pretty simple!

The biggest caveat is that EBS volumes can only be mounted to servers in the same availability zone that they were originally created in.  So, if you're using this tool in an autoscaling group that launches servers across multiple availability zones, it's possible to "orphan" volumes in the other AZ and not attach them when they're available.  Correcting this is planned in a future enhancement.

For a complete working example, check out the `terraform/aws/example` sub-directory and its documentation.

## Configuration

cloud-persistent-storage uses a simple YAML file for configuration.  Here's a complete documented example:

```yaml
# optional; the block device to mount EBS volumes to.  Defaults to /dev/xvdc.
block-device: /dev/xvdc

# required; the block device "provider"
block-provider:
  # aws-ebs is currently the only supported provider
  aws-ebs:
    # required; one or more tags to attach to the EBS volume.  When searching
    # for existing volumes to re-attach to, they must have all of these tags.
    # These tags will be automatically added to any EBS volume that we create
    # when we can't find an existing volume.
    ebs-tags:
      environment: Production
      role: PostgreSQL
    # optional; EBS volume type, "gp2" | "io1" | "st1" | "sc1".  gp2 default.
    type: gp2
    # required; size (GB) to create new volumes.
    size: 200

# optional; configuration about file system creation
file-system:
  # optional; command-line arguments to mkfs subprocess.  Defaults to creating
  # ext4 filesystem with no reserved superuser blocks (-m 0).
  mkfs:
    - -t
    - ext4
    - -m
    - 0

# optional; configuration about mounting filesystem
mount:
  # optional; mount point.  Defaults to /mnt
  target: /mnt
```

## Current Limitations

- As mentioned above, AWS EBS volumes can only be mounted on servers in the same AZ.  This tool does not currently do anything to address this issue; if volumes are unmountable because they're in the wrong AZ, they'll be skipped, and other available volumes will be mounted instead (or new volumes will be created).

- Only supports AWS + EC2 + EBS.  I'd like to support other cloud providers.

- Only supports Linux.

- Only works with Linux ext2/3/4 filesystems.  When a block storage device is attached, it needs to detect whether the device already has a filesystem (eg. from a previous VM being attached), or whether the filesystem needs to be created (eg. volume was just created, or, previous VM created it but failed to create a filesystem).  This detection currently reads the ext filesystem magic bytes to detect whether the filesystem exists.  This could and should be enhanced to support other filesystems.  See the `filesystem_exists` function in [mkfs.fs](https://github.com/mfenniak/cloud-persistent-storage/blob/master/src/mkfs.rs).

## Dream Feature List

- Support multiple cloud providers
    - Amazon Web Services
    - Google Cloud Platform
    - Azure
- Creation options for persistent volume:
    - ~~Disk size~~
    - ~~Volume type (eg. EBS -> gp2, io1, st1, sc1)~~
    - ~~Filesystem creation options~~
    - Create from snapshot, rather than creating empty volume
- Options for mounting persistent volume:
    - ~~Location~~
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
- Documentation should show how to use this tool in a user_data script (AWS)
    - ~~Linux~~
    - Windows
- Configuration via;
    - Command-line options
    - ~~YAML file~~
- Logging
- Support for resizing volumes if configuration changes
    - eg. with AWS EBS, volume resize, filesystem resize to match, then mount

### AWS Cross-AZ Sharing Without External Coordinator

In the future, I'd like to add the ability to move EBS volumes across AZs if they're available but in the wrong AZ.  This would be an important capability for running odd-numbered server clusters (eg. Consul) in an ASG across two AZs.  The struggle would be that multi-server simultanous spin-up could cause multiple servers to "claim" the same volume and try to move it into their AZ, which would cause both duplicate volumes and orphan volumes.  I'm theorizing that I could come up with a slow, pretty reasonable consensus algorithm using EC2 tags:

- Slow consensus:
    - Find volume I'd like to own
        - If volume has been previously tagged as owned, but it is still available, ignore the tag after 180s (lock timeout)
    - Tag volume with {Token: ...uuid..., Timestamp: ...ts...}
    - Wait 30 seconds (hopeful consensus timeout)
    - Ensure volume has tags {Token: ...uuid..., Timestamp: ...ts...} as expected; it is now ours
    - Only give up and create a volume after all volumes are in-use
