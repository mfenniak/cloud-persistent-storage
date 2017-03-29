resource "aws_iam_role" "example" {
  name = "cloud-persistent-storage-example-${var.environment}-instance-role"

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

resource "aws_iam_instance_profile" "example" {
  name  = "cloud-persistent-storage-example-${var.environment}-instance-profile"
  roles = ["${aws_iam_role.example.name}"]
}

resource "aws_iam_role_policy" "example" {
  name = "cloud-persistent-storage-example-${var.environment}-instance-role-polcy"
  role = "${aws_iam_role.example.id}"

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
