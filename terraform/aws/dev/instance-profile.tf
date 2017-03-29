resource "aws_iam_role" "test" {
  name = "cloud-persistent-storage-test-instance-role"

  assume_role_policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": "sts:AssumeRole",
      "Principal": {
        "Service": "ec2.amazonaws.com"
      },
      "Effect": "Allow",
      "Sid": ""
    }
  ]
}
EOF
}

resource "aws_iam_instance_profile" "test" {
  name  = "cloud-persistent-storage-test-instance-profile"
  roles = ["${aws_iam_role.test.name}"]
}

resource "aws_iam_role_policy" "test" {
  name = "cloud-persistent-storage-test-instance-role-polcy"
  role = "${aws_iam_role.test.id}"

  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "ec2:CreateVolume",
        "ec2:CreateTags",
        "ec2:AttachVolume",
        "ec2:DescribeVolumes"
      ],
      "Resource": "*"
    }
  ]
}
EOF
}
