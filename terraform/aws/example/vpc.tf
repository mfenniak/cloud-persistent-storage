resource "aws_vpc" "main" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_support   = true
  enable_dns_hostnames = true

  tags {
    Name = "cloud-persistent-storage-example-${var.environment}"
  }
}

resource "aws_subnet" "a" {
  vpc_id                  = "${aws_vpc.main.id}"
  cidr_block              = "10.0.1.0/24"
  availability_zone       = "ca-central-1a"
  map_public_ip_on_launch = true

  tags {
    Name = "cloud-persistent-storage-example-subnet-a-${var.environment}"
  }
}

resource "aws_subnet" "b" {
  vpc_id                  = "${aws_vpc.main.id}"
  cidr_block              = "10.0.2.0/24"
  availability_zone       = "ca-central-1b"
  map_public_ip_on_launch = true

  tags {
    Name = "cloud-persistent-storage-example-subnet-b-${var.environment}"
  }
}

resource "aws_internet_gateway" "gw" {
  vpc_id = "${aws_vpc.main.id}"

  tags {
    Name = "cloud-persistent-storage-example-${var.environment}"
  }
}

resource "aws_route_table" "main" {
  vpc_id = "${aws_vpc.main.id}"

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = "${aws_internet_gateway.gw.id}"
  }

  tags {
    Name = "cloud-persistent-storage-example-${var.environment}"
  }
}

resource "aws_route_table_association" "a" {
  subnet_id      = "${aws_subnet.a.id}"
  route_table_id = "${aws_route_table.main.id}"
}

resource "aws_route_table_association" "b" {
  subnet_id      = "${aws_subnet.b.id}"
  route_table_id = "${aws_route_table.main.id}"
}
