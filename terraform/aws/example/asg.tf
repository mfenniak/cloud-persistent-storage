resource "aws_launch_configuration" "example" {
  name_prefix          = "cloud-persistent-storage-example-${var.environment}"
  instance_type        = "${var.instance_type}"
  image_id             = "${var.ami}"
  key_name             = "${var.key_name}"
  iam_instance_profile = "${aws_iam_instance_profile.example.id}"

  security_groups = ["${aws_security_group.example.id}"]

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
      Environment: ${var.environment}
      Role: PostgreSQL
    type: gp2
    size: 200
CONFIG
SSL_CERT_DIR=/etc/ssl/certs RUST_LOG=cloud_persistent_storage ./cloud-persistent-storage -c /etc/cloud-persistent-storage.yml
EOF

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_autoscaling_group" "example" {
  name                 = "cloud-persistent-storage-example-${var.environment}"
  launch_configuration = "${aws_launch_configuration.example.name}"
  max_size             = "5"
  min_size             = "0"
  desired_capacity     = "1"
  vpc_zone_identifier  = ["${aws_subnet.a.id}", "${aws_subnet.b.id}"]

  tag {
    key                 = "Name"
    value               = "cloud-persistent-storage-example-${var.environment}"
    propagate_at_launch = true
  }

  tag {
    key                 = "Environment"
    value               = "${var.environment}"
    propagate_at_launch = true
  }

  lifecycle {
    create_before_destroy = true
  }
}
