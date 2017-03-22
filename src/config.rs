use serde_yaml;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "block-provider")]
    block_provider: BlockProvider,
}

#[derive(Debug, Deserialize)]
pub enum BlockProvider {
    #[serde(rename = "aws-ebs")]
    AwsEbs(EbsBlockProviderConfig),
}

#[derive(Debug, Deserialize)]
pub struct EbsBlockProviderConfig {
    #[serde(rename = "type")]
    pub volume_type: String,
    pub size: i32,
}

#[derive(Debug)]
pub enum ConfigError {
    YamlParseError(serde_yaml::Error),
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

    const EXAMPLE_CONFIG_LINUX: &'static str = r#"
block-provider:
  aws-ebs:
    ebs-tags:
      tag-a: value-a
    type: gp2
    size: 200

file-system:
  mkfs: -t ext4 -m 0

mount:
  target: /mnt/test
"#;

    #[test]
    fn parses_ebs_block_provider() {
        let config = parse_config(EXAMPLE_CONFIG_LINUX).unwrap();
        match config.block_provider {
            BlockProvider::AwsEbs(ebs_config) => {
                assert_eq!("gp2", ebs_config.volume_type);
                assert_eq!(200, ebs_config.size);
            }
            _ => assert!(false, "expected AwsEbs block provider"),
        }
    }
}
