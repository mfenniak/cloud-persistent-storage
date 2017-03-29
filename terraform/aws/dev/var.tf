variable "key_name" {
  type = "string"
}

variable "region" {
  default = "ca-central-1"
}

variable "zones" {
  default = ["ca-central-1a", "ca-central-1b"]
}

variable "ami" {
  default = "ami-7e57ea1a"
}

variable "instance_type" {
  default = "t2.medium"
}
