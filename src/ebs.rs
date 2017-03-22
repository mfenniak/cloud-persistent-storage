use rusoto;
use std;
use aws_instance_metadata;
use rusoto::{DefaultCredentialsProvider, ProvideAwsCredentials, DispatchSignedRequest};
use rusoto::ec2::{Ec2Client, DescribeVolumesRequest, Filter, AttachVolumeRequest};
use rusoto::default_tls_client;

#[derive(Debug)]
pub enum AttachVolumeError {
    NoVolumesAvailable,
    AllAttachesFailed,
    DescribeVolumesFailed(rusoto::ec2::DescribeVolumesError),
    DescribeVolumesPaginationSupportRequired,
}

impl From<rusoto::ec2::DescribeVolumesError> for AttachVolumeError {
    fn from(err: rusoto::ec2::DescribeVolumesError) -> AttachVolumeError {
        AttachVolumeError::DescribeVolumesFailed(err)
    }
}

pub fn find_and_attach_volume() -> Result<(), AttachVolumeError> {
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

    attach_volume(&ec2_client, &metadata)
}

fn attach_volume<P, D>(ec2_client: &Ec2Client<P, D>,
                       metadata: &aws_instance_metadata::metadata::InstanceMetadata)
                       -> Result<(), AttachVolumeError>
    where P: ProvideAwsCredentials,
          D: DispatchSignedRequest
{
    // FIXME: need command-line option to provide tag-name and tag-value
    // FIXME: add filter for attachment.status (possible options: {attaching | attached | detaching | detached})
    let filter = Some(vec![Filter {
                               name: Some("tag:tag-a".to_owned()),
                               values: Some(vec!["value-a".to_owned()]),
                           }]);
    let request = DescribeVolumesRequest {
        dry_run: None,
        filters: filter,
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
            let attach_volume_result = attach_specific_volume(metadata.instance_id.as_str(),
                                                              vol.volume_id.as_ref().unwrap(),
                                                              &ec2_client);
            if attach_volume_result.is_ok() {
                info!("successfully attached volume");
                return Ok(());
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

fn attach_specific_volume<P, D>(instance_id: &str,
                                volume_id: &str,
                                ec2_client: &Ec2Client<P, D>)
                                -> Result<(), rusoto::ec2::AttachVolumeError>
    where P: ProvideAwsCredentials,
          D: DispatchSignedRequest
{
    let request = AttachVolumeRequest {
        device: String::from("/dev/xvdh"), // FIXME
        dry_run: None, //
        instance_id: String::from(instance_id),
        volume_id: String::from(volume_id),
    };
    try!(ec2_client.attach_volume(&request));
    Ok(())
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
        let result = attach_specific_volume(&String::from("i-1234"),
                                            &String::from("vol-4321"),
                                            &mock_ec2_client);
        result.expect("success test case");
    }

    #[test]
    fn test_attach_specific_volume_failure() {
        let mock_request_dispatcher = Ec2RequestDispatcherAttachSpecificVolumeFailure {};
        let mock_ec2_client = rusoto::ec2::Ec2Client::new(mock_request_dispatcher,
                                                          MockProvideAwsCredentials {},
                                                          rusoto::Region::UsWest2);
        let result = attach_specific_volume(&String::from("i-1234"),
                                            &String::from("vol-4321"),
                                            &mock_ec2_client);
        assert!(result.is_err())
    }
}
