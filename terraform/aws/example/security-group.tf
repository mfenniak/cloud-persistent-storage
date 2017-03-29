resource "aws_security_group" "example" {
  name   = "cloud-persistent-storage-example-${var.environment}"
  vpc_id = "${aws_vpc.main.id}"

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port = 0
    to_port   = 0
    protocol  = "-1"
    self      = true
  }

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["68.147.122.199/32"]
  }

  tags {
    Name = "cloud-persistent-storage-example-${var.environment}"
  }
}
