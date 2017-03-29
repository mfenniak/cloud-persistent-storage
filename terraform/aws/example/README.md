# Terraform AWS/EBS Example

This is an example of using cloud-persistent-storage with a [Terraform](https://www.terraform.io/)-based spin-up of an autoscaling group with automatic self-provisioning persistent storage on Amazon EBS volumes.

## Running This Example

Run `terraform` in this directory, and provide it with a `key_name` variable pointing at your EC2 key pair.  For example: `terraform -var key_name=my-private-key`.

To tear down the example instances later, run `terraform destroy` to cleanup the resources created, and then you will need to **manually** remove the persistent EBS volumes that were automatically created by cloud-persistent-storage.  These resources will cost you money if you do not remove them.

## user_data script

The majority of the special-sauce that occurs here is in the user_data script in the autoscaling launch configuration.  This user_data script downloads a release of cloud-persistent-storage, creates a configuration file for it, and runs the tool.

```bash
wget https://github.com/mfenniak/cloud-persistent-storage/releases/download/v1.0.0/cloud-persistent-storage_v1.0.0_linux_amd64.zip
unzip cloud-persistent-storage_v1.0.0_linux_amd64.zip
chmod +x ./cloud-persistent-storage

cat > /etc/cloud-persistent-storage.yml <<CONFIG
block-provider:
  aws-ebs:
    ebs-tags:
      Environment: Production
      Role: PostgreSQL
    type: gp2
    size: 200
CONFIG

export SSL_CERT_DIR=/etc/ssl/certs
export RUST_LOG=cloud_persistent_storage
./cloud-persistent-storage -c /etc/cloud-persistent-storage.yml
```

The impact of this is that servers coming up in this autoscaling group will automatically have a 200GB gp2 EBS volume attached to /mnt.  If a server is terminated for any reason, when a new server is launched in the autoscaling group, it will re-attach to the previous EBS volume (if launched in the same availability zone).

## Instance Profile Permissions

In addition to the user_data script described above, it is also necessary for the launched server to have permission to execute AWS API calls to query, create, and attach EBS volumes.  Those specific permissions are described in the `instance-profile.tf` file.

Broadly speaking, the following actions are required: `ec2:CreateVolume`, `ec2:CreateTags`,`ec2:AttachVolume`, `ec2:DescribeVolumes`.
