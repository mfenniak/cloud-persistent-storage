use serde_yaml;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub block_provider: BlockProvider,
    #[serde(default = "default_file_system")]
    pub file_system: FileSystem,
    #[serde(default = "default_mount")]
    pub mount: Mount,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlockProvider {
    AwsEbs(EbsBlockProviderConfig),
    ReservedForFuture,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct EbsBlockProviderConfig {
    #[serde(rename = "type")]
    #[serde(default = "default_ebs_volume_type")]
    pub volume_type: String,
    pub size: i32,
    pub ebs_tags: HashMap<String, String>,
}

fn default_ebs_volume_type() -> String {
    String::from("gp2")
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSystem {
    #[serde(default = "default_file_system_mkfs")]
    pub mkfs: String,
}

fn default_file_system() -> FileSystem {
    FileSystem { mkfs: default_file_system_mkfs() }
}

fn default_file_system_mkfs() -> String {
    String::from("-t ext4 -m 0")
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mount {
    #[serde(default = "default_mount_target")]
    pub target: String,
}

fn default_mount() -> Mount {
    Mount { target: default_mount_target() }
}

fn default_mount_target() -> String {
    String::from("/mnt")
}

#[derive(Debug)]
pub enum ConfigError {
    YamlParseError(serde_yaml::Error),
    ReservedForFuture,
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> ConfigError {
        ConfigError::YamlParseError(err)
    }
}

pub fn parse_config(config_str: &str) -> Result<Config, ConfigError> {
    let config = try!(serde_yaml::from_str(config_str));
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_MINIMAL_EBS_CONFIG: &'static str = r#"
block-provider:
  aws-ebs:
    ebs-tags: {}
    size: 200
"#;

    const EXAMPLE_FULL_EBS_CONFIG: &'static str = r#"
block-provider:
  aws-ebs:
    ebs-tags:
      tag-a: value-a
    type: gp2
    size: 200

file-system:
  mkfs: -t ext4 -m 5

mount:
  target: /mnt/test
"#;

    #[test]
    fn toplevel_deny_unknown_fields() {
        let config_text = String::from(EXAMPLE_MINIMAL_EBS_CONFIG) + "\n\nabc-123: hello";
        match parse_config(config_text.as_str()).unwrap_err() {
            ConfigError::YamlParseError(_) => {}
            _ => assert!(false, "expected YamlParseError"),
        }
    }

    #[test]
    fn parses_block_provider_aws_ebs() {
        let config = parse_config(EXAMPLE_FULL_EBS_CONFIG).unwrap();
        match config.block_provider {
            BlockProvider::AwsEbs(ebs_config) => {
                assert_eq!("gp2", ebs_config.volume_type);
                assert_eq!(200, ebs_config.size);
                assert!(ebs_config.ebs_tags.get("tag-a").unwrap() == "value-a");
            }
            _ => assert!(false, "expected AwsEbs block provider"),
        }
    }

    #[test]
    fn block_provider_aws_ebs_defaults() {
        let config = parse_config(EXAMPLE_MINIMAL_EBS_CONFIG).unwrap();
        match config.block_provider {
            BlockProvider::AwsEbs(ebs_config) => {
                assert_eq!("gp2", ebs_config.volume_type);
            }
            _ => assert!(false, "expected AwsEbs block provider"),
        }
    }

    #[test]
    fn block_provider_aws_ebs_deny_unknown_fields() {
        let config_text = r#"
block-provider:
  aws-ebs:
    ebs-tags: {}
    size: 200
    magic: true
"#;
        match parse_config(config_text).unwrap_err() {
            ConfigError::YamlParseError(_) => {}
            _ => assert!(false, "expected YamlParseError"),
        }
    }

    #[test]
    fn parses_file_system() {
        let config = parse_config(EXAMPLE_FULL_EBS_CONFIG).unwrap();
        assert_eq!("-t ext4 -m 5", config.file_system.mkfs);
    }

    #[test]
    fn file_system_default() {
        let config = parse_config(EXAMPLE_MINIMAL_EBS_CONFIG).unwrap();
        assert_eq!("-t ext4 -m 0", config.file_system.mkfs);
    }

    #[test]
    fn file_system_default_mkfs() {
        let config_text = String::from(EXAMPLE_MINIMAL_EBS_CONFIG) + "\n\nfile-system: {}";
        let config = parse_config(config_text.as_str()).unwrap();
        assert_eq!("-t ext4 -m 0", config.file_system.mkfs);
    }

    #[test]
    fn file_system_deny_unknown_fields() {
        let config_text = String::from(EXAMPLE_MINIMAL_EBS_CONFIG) +
                          "\n\nfile-system: { \"huh\": 123 }";
        match parse_config(config_text.as_str()).unwrap_err() {
            ConfigError::YamlParseError(_) => {}
            _ => assert!(false, "expected YamlParseError"),
        }
    }

    #[test]
    fn parses_mount() {
        let config = parse_config(EXAMPLE_FULL_EBS_CONFIG).unwrap();
        assert_eq!("/mnt/test", config.mount.target);
    }

    #[test]
    fn mount_default() {
        let config = parse_config(EXAMPLE_MINIMAL_EBS_CONFIG).unwrap();
        assert_eq!("/mnt", config.mount.target);
    }

    #[test]
    fn mount_default_target() {
        let config_text = String::from(EXAMPLE_MINIMAL_EBS_CONFIG) + "\n\nmount: {}";
        let config = parse_config(config_text.as_str()).unwrap();
        assert_eq!("/mnt", config.mount.target);
    }

    #[test]
    fn mount_deny_unknown_fields() {
        let config_text = String::from(EXAMPLE_MINIMAL_EBS_CONFIG) + "\n\nmount: { \"huh\": 123 }";
        match parse_config(config_text.as_str()).unwrap_err() {
            ConfigError::YamlParseError(_) => {}
            _ => assert!(false, "expected YamlParseError"),
        }
    }
}
