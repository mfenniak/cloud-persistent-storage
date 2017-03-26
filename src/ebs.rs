use rusoto;
use std;
use aws_instance_metadata;
use rusoto::{DefaultCredentialsProvider, ProvideAwsCredentials, DispatchSignedRequest};
use rusoto::ec2::{Ec2Client, DescribeVolumesRequest, DescribeVolumesError, Filter,
                  AttachVolumeRequest};
use rusoto::default_tls_client;
use config::EbsBlockProviderConfig;

#[derive(Debug)]
pub enum AttachVolumeError {
    NoVolumesAvailable,
    AllAttachesFailed,
    DescribeVolumesFailed(DescribeVolumesError),
    DescribeVolumesPaginationSupportRequired,
    TimeoutWaitingForVolumeToAttach,
}

impl From<rusoto::ec2::DescribeVolumesError> for AttachVolumeError {
    fn from(err: rusoto::ec2::DescribeVolumesError) -> AttachVolumeError {
        AttachVolumeError::DescribeVolumesFailed(err)
    }
}

pub fn create_filters(config: &EbsBlockProviderConfig) -> Vec<Filter> {
    let mut filters = Vec::with_capacity(config.ebs_tags.len() + 1);
    for (tag_name, tag_value) in &config.ebs_tags {
        filters.push(Filter {
                         name: Some(String::from("tag:") + tag_name),
                         values: Some(vec![tag_value.to_owned()]),
                     })
    }
    filters.push(Filter {
                     name: Some("status".to_owned()),
                     values: Some(vec!["available".to_owned()]),
                 });
    filters
}

pub fn find_and_attach_volume(block_device: &str,
                              config: &EbsBlockProviderConfig)
                              -> Result<(), AttachVolumeError> {
    let metadata = match aws_instance_metadata::get() {
        Ok(metadata) => metadata,
        Err(e) => {
            error!("Unable to retrieve instance metadata.  Am I running on EC2?  {:?}",
                   e);
            std::process::exit(100);
        }
    };
    let credentials = DefaultCredentialsProvider::new().unwrap();

    let ec2_client = Ec2Client::new(default_tls_client().unwrap(),
                                    credentials,
                                    metadata.region().unwrap());

    let request = DescribeVolumesRequest {
        dry_run: None,
        filters: Some(create_filters(config)),
        max_results: None,
        next_token: None,
        volume_ids: None,
    };

    trace!("executing DescribeVolumes");
    let response = try!(ec2_client.describe_volumes(&request));

    if response.next_token.is_some() {
        error!("DescribeVolumes returned multiple pages of results; this is not currently supported");
        return Err(AttachVolumeError::DescribeVolumesPaginationSupportRequired);
    }

    if let Some(volumes) = response.volumes {
        if volumes.is_empty() {
            return Err(AttachVolumeError::NoVolumesAvailable);
        }

        for vol in &volumes {
            debug!("attempting to attach target volume: {:?}", vol);
            let attach_volume_result = attach_specific_volume(block_device,
                                                              metadata.instance_id.as_str(),
                                                              vol.volume_id.as_ref().unwrap(),
                                                              &ec2_client);
            if attach_volume_result.is_ok() {
                info!("successfully issued attach request");
                return ensure_volume_attached(&ec2_client, vol.volume_id.as_ref().unwrap());
            } else {
                debug!("failed to attach volume: {:?}", attach_volume_result.err())
            }
        }

        info!("all queried volumes have been attempted");
        Err(AttachVolumeError::AllAttachesFailed)
    } else {
        Err(AttachVolumeError::NoVolumesAvailable)
    }
}

fn attach_specific_volume<P, D>(block_device: &str,
                                instance_id: &str,
                                volume_id: &str,
                                ec2_client: &Ec2Client<P, D>)
                                -> Result<(), rusoto::ec2::AttachVolumeError>
    where P: ProvideAwsCredentials,
          D: DispatchSignedRequest
{
    let request = AttachVolumeRequest {
        device: String::from(block_device),
        dry_run: None,
        instance_id: String::from(instance_id),
        volume_id: String::from(volume_id),
    };
    try!(ec2_client.attach_volume(&request));
    Ok(())
}

fn ensure_volume_attached<P, D>(ec2_client: &Ec2Client<P, D>,
                                volume_id: &str)
                                -> Result<(), AttachVolumeError>
    where P: ProvideAwsCredentials,
          D: DispatchSignedRequest
{
    info!("waiting for volume to attach");
    let request = DescribeVolumesRequest {
        dry_run: None,
        filters: None,
        max_results: None,
        next_token: None,
        volume_ids: Some(vec![String::from(volume_id)]),
    };

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5 * 60);
    let sleep = std::time::Duration::from_secs(5);
    while std::time::Instant::now().duration_since(start) < timeout {
        if check_volume_attached(&ec2_client, &request)? {
            return Ok(());
        }
        std::thread::sleep(sleep);
    }
    Err(AttachVolumeError::TimeoutWaitingForVolumeToAttach)
}

fn check_volume_attached<P, D>(ec2_client: &Ec2Client<P, D>,
                               request: &DescribeVolumesRequest)
                               -> Result<bool, AttachVolumeError>
    where P: ProvideAwsCredentials,
          D: DispatchSignedRequest
{
    trace!("checking DescribeVolumes to see if volume is attached");
    ec2_client.describe_volumes(&request)?
        .volumes
        .as_ref()
        .and_then(|volume_list| volume_list.get(0))
        .and_then(|volume| volume.attachments.as_ref())
        .and_then(|attachments| attachments.get(0))
        .and_then(|attachment| attachment.state.as_ref())
        .map_or(Ok(false), |state| Ok(state == "attached"))
}

#[cfg(test)]
mod tests {
    extern crate hyper;

    use super::*;
    use chrono::{Duration, UTC};
    use std::collections::HashMap;

    struct MockProvideAwsCredentials {}

    impl rusoto::ProvideAwsCredentials for MockProvideAwsCredentials {
        fn credentials(&self) -> Result<rusoto::AwsCredentials, rusoto::CredentialsError> {
            Ok(rusoto::AwsCredentials::new("key",
                                           "secret",
                                           None,
                                           UTC::now() + Duration::seconds(600)))
        }
    }

    struct Ec2RequestDispatcherAttachSpecificVolumeSuccess {}

    impl rusoto::DispatchSignedRequest for Ec2RequestDispatcherAttachSpecificVolumeSuccess {
        fn dispatch(&self,
                    request: &rusoto::SignedRequest)
                    -> Result<rusoto::HttpResponse, rusoto::HttpDispatchError> {
            assert!(request.params.get("Device") == Some(&Some(String::from("/dev/xvdh"))));
            assert!(request.params.get("InstanceId") == Some(&Some(String::from("i-1234"))));
            assert!(request.params.get("VolumeId") == Some(&Some(String::from("vol-4321"))));
            assert!(request.params.get("Action") == Some(&Some(String::from("AttachVolume"))));
            Ok(rusoto::HttpResponse {
                   status: hyper::status::StatusCode::Ok,
                   body: String::from(""),
                   raw_body: vec![],
                   headers: HashMap::new(),
               })
        }
    }

    struct Ec2RequestDispatcherAttachSpecificVolumeFailure {}

    impl rusoto::DispatchSignedRequest for Ec2RequestDispatcherAttachSpecificVolumeFailure {
        fn dispatch(&self,
                    _: &rusoto::SignedRequest)
                    -> Result<rusoto::HttpResponse, rusoto::HttpDispatchError> {
            Ok(rusoto::HttpResponse {
                   status: hyper::status::StatusCode::BadRequest,
                   body: String::from(""),
                   raw_body: vec![],
                   headers: HashMap::new(),
               })
        }
    }


    #[test]
    fn test_attach_specific_volume_success() {
        let mock_request_dispatcher = Ec2RequestDispatcherAttachSpecificVolumeSuccess {};
        let mock_ec2_client = rusoto::ec2::Ec2Client::new(mock_request_dispatcher,
                                                          MockProvideAwsCredentials {},
                                                          rusoto::Region::UsWest2);
        let result = attach_specific_volume("/dev/xvdh", "i-1234", "vol-4321", &mock_ec2_client);
        result.expect("success test case");
    }

    #[test]
    fn test_attach_specific_volume_failure() {
        let mock_request_dispatcher = Ec2RequestDispatcherAttachSpecificVolumeFailure {};
        let mock_ec2_client = rusoto::ec2::Ec2Client::new(mock_request_dispatcher,
                                                          MockProvideAwsCredentials {},
                                                          rusoto::Region::UsWest2);
        let result = attach_specific_volume("/dev/xvdh", "i-1234", "vol-4321", &mock_ec2_client);
        assert!(result.is_err())
    }
}
