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

apt-get update
apt-get install -y build-essential pkg-config libssl-dev python-pip
locale-gen en_CA.UTF-8
update-locale
pip install awscli
sudo -H -u ubuntu sh -c "curl https://sh.rustup.rs -sSf | sh -s -- -y"
sudo -H -u ubuntu sh -c "cd ~/ && git clone https://github.com/mfenniak/cloud-persistent-storage.git && cd cloud-persistent-storage && ~/.cargo/bin/cargo build"
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
