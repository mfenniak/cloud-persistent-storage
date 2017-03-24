use serde_yaml;
use std;
use std::collections::HashMap;
use std::fs::File;
use std::error::Error;
use std::fmt;

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
    IoError(std::io::Error),
    InvalidBlockProviderAwsEbs(String),
    InvalidFileSystem(String),
    InvalidMount(String),
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::YamlParseError(ref err) => err.description(),
            ConfigError::IoError(ref err) => err.description(),
            ConfigError::InvalidBlockProviderAwsEbs(_) => "invalid configuration in block-provider aws-ebs",
            ConfigError::InvalidFileSystem(_) => "invalid configuration in file-system",
            ConfigError::InvalidMount(_) => "invalid configuration in mount",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ConfigError::YamlParseError(ref err) => Some(err),
            ConfigError::IoError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::YamlParseError(ref err) => err.fmt(f),
            ConfigError::IoError(ref err) => err.fmt(f),
            ConfigError::InvalidBlockProviderAwsEbs(ref msg) => write!(f, "{}", msg),
            ConfigError::InvalidFileSystem(ref msg) => write!(f, "{}", msg),
            ConfigError::InvalidMount(ref msg) => write!(f, "{}", msg),
        }
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> ConfigError {
        ConfigError::YamlParseError(err)
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> ConfigError {
        ConfigError::IoError(err)
    }
}

fn parse_config(config_str: &str) -> Result<Config, ConfigError> {
    let config = serde_yaml::from_str(config_str)?;
    if let Some(err) = validate_config(&config) {
        Err(err)
    } else {
        Ok(config)
    }
}

pub fn read_config_from_file(config_path: &str) -> Result<Config, ConfigError> {
    let config_file = File::open(config_path)?;
    let config = serde_yaml::from_reader(config_file)?;
    if let Some(err) = validate_config(&config) {
        Err(err)
    } else {
        Ok(config)
    }
}

pub fn validate_config(config: &Config) -> Option<ConfigError> {
    validate_block_provider(&config.block_provider)
        .or_else(|| validate_file_system(&config.file_system))
        .or_else(|| validate_mount(&config.mount))
}

fn validate_block_provider(block_provider: &BlockProvider) -> Option<ConfigError> {
    match block_provider {
        &BlockProvider::AwsEbs(ref ebs_block_provider_config) => {
            validate_block_provider_aws_ebs_config(&ebs_block_provider_config)
        }
        &BlockProvider::ReservedForFuture => panic!("huh"),
    }
}

fn validate_block_provider_aws_ebs_config(config: &EbsBlockProviderConfig) -> Option<ConfigError> {
    validate_block_provider_aws_ebs_volume_type(&config.volume_type)
    .or_else(|| if config.size < 0 { Some(ConfigError::InvalidBlockProviderAwsEbs(String::from("invalid volume size"))) } else { None })
    .or_else(|| if config.ebs_tags.len() <= 0 { Some(ConfigError::InvalidBlockProviderAwsEbs(String::from("at least one ebs tag is required"))) } else { None })
}

fn validate_block_provider_aws_ebs_volume_type(volume_type: &str) -> Option<ConfigError> {
    match volume_type {
        "gp2" | "io1" | "st1" | "sc1" => None,
        vt => {
            Some(ConfigError::InvalidBlockProviderAwsEbs(String::from("invalid volume type, expected gp2, io1, st1, sc1: ") +
                                                         vt))
        }
    }
}

fn validate_file_system(config: &FileSystem) -> Option<ConfigError> {
    if config.mkfs.is_empty() {
        Some(ConfigError::InvalidFileSystem(String::from("required parameter mkfs")))
    } else {
        None
    }
}

fn validate_mount(config: &Mount) -> Option<ConfigError> {
    if config.target.is_empty() {
        Some(ConfigError::InvalidMount(String::from("required parameter target")))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_block_provider_aws_ebs_volume_type() {
        let config = Config {
            block_provider: BlockProvider::AwsEbs(EbsBlockProviderConfig {
                                                      ebs_tags: HashMap::new(),
                                                      size: 200,
                                                      volume_type: String::from("grr-arg"),
                                                  }),
            file_system: default_file_system(),
            mount: default_mount(),
        };
        let err = validate_config(&config).expect("expected config error");
        assert_eq!("invalid configuration in block-provider aws-ebs",
                   err.description());
        assert_eq!("invalid volume type, expected gp2, io1, st1, sc1: grr-arg",
                   format!("{}", err));
    }

    #[test]
    fn validate_block_provider_aws_ebs_size() {
        let config = Config {
            block_provider: BlockProvider::AwsEbs(EbsBlockProviderConfig {
                                                      ebs_tags: HashMap::new(),
                                                      size: -100,
                                                      volume_type: String::from("gp2"),
                                                  }),
            file_system: default_file_system(),
            mount: default_mount(),
        };
        let err = validate_config(&config).expect("expected config error");
        assert_eq!("invalid configuration in block-provider aws-ebs",
                   err.description());
        assert_eq!("invalid volume size", format!("{}", err));
    }

    #[test]
    fn validate_block_provider_aws_ebs_tags() {
        let config = Config {
            block_provider: BlockProvider::AwsEbs(EbsBlockProviderConfig {
                                                      ebs_tags: HashMap::new(),
                                                      size: 200,
                                                      volume_type: String::from("gp2"),
                                                  }),
            file_system: default_file_system(),
            mount: default_mount(),
        };
        let err = validate_config(&config).expect("expected config error");
        assert_eq!("invalid configuration in block-provider aws-ebs",
                   err.description());
        assert_eq!("at least one ebs tag is required", format!("{}", err));
    }

    #[test]
    fn validate_file_system_mkfs() {
        let mut ebs_tags: HashMap<String, String> = HashMap::new();
        ebs_tags.insert(String::from("a"), String::from("b"));
        let config = Config {
            block_provider: BlockProvider::AwsEbs(EbsBlockProviderConfig {
                                                      ebs_tags: ebs_tags,
                                                      size: 200,
                                                      volume_type: String::from("gp2"),
                                                  }),
            file_system: FileSystem { mkfs: String::from("") },
            mount: default_mount(),
        };
        let err = validate_config(&config).expect("expected config error");
        assert_eq!("invalid configuration in file-system", err.description());
        assert_eq!("required parameter mkfs", format!("{}", err));
    }

    #[test]
    fn validate_mount_target() {
        let mut ebs_tags: HashMap<String, String> = HashMap::new();
        ebs_tags.insert(String::from("a"), String::from("b"));
        let config = Config {
            block_provider: BlockProvider::AwsEbs(EbsBlockProviderConfig {
                                                      ebs_tags: ebs_tags,
                                                      size: 200,
                                                      volume_type: String::from("gp2"),
                                                  }),
            file_system: default_file_system(),
            mount: Mount { target: String::from("") },
        };
        let err = validate_config(&config).expect("expected config error");
        assert_eq!("invalid configuration in mount", err.description());
        assert_eq!("required parameter target", format!("{}", err));
    }

    const EXAMPLE_MINIMAL_EBS_CONFIG: &'static str = r#"
block-provider:
  aws-ebs:
    ebs-tags: { a: "b" }
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
