resource "aws_launch_configuration" "test" {
  name_prefix          = "cloud-persistent-storage-test"
  instance_type        = "${var.instance_type}"
  image_id             = "${var.ami}"
  key_name             = "${var.key_name}"
  iam_instance_profile = "${aws_iam_instance_profile.test.id}"

  security_groups = ["${aws_security_group.test.id}"]

  user_data = <<EOF
#!/usr/bin/env bash
set -euxo pipefail

apt-get install unzip
wget https://github.com/mfenniak/cloud-persistent-storage/releases/download/v1.0.0/cloud-persistent-storage_v1.0.0_linux_amd64.zip
unzip cloud-persistent-storage_v1.0.0_linux_amd64.zip
chmod +x ./cloud-persistent-storage

cat > /etc/cloud-persistent-storage.yml <<CONFIG
block-provider:
  aws-ebs:
    ebs-tags:
      environment: production
      role: postgresql
    type: gp2
    size: 200
CONFIG
RUST_LOG=cloud_persistent_storage ./cloud-persistent-storage -c /etc/cloud-persistent-storage.yml
EOF

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_autoscaling_group" "test" {
  name                 = "cloud-persistent-storage-test"
  launch_configuration = "${aws_launch_configuration.test.name}"
  max_size             = "5"
  min_size             = "0"
  desired_capacity     = "1"
  vpc_zone_identifier  = ["${aws_subnet.a.id}", "${aws_subnet.b.id}"]

  tag {
    key                 = "Name"
    value               = "cloud-persistent-storage-test"
    propagate_at_launch = true
  }

  lifecycle {
    create_before_destroy = true
  }
}
