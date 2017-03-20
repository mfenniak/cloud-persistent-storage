
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rusoto;
extern crate aws_instance_metadata;

use rusoto::{DefaultCredentialsProvider, ProvideAwsCredentials, DispatchSignedRequest};
use rusoto::ec2::{Ec2Client, DescribeVolumesRequest, Filter, AttachVolumeRequest};
use rusoto::default_tls_client;

fn main() {
    env_logger::init().unwrap();

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

    match attach_volume(&ec2_client, &metadata) {
        Ok(_) => {
            info!("attach volume succeeded")
            // FIXME: create/ensure filesystem
            // FIXME: mount volume
        }
        Err(e) => {
            error!("attach volume failed: {:?}", e);
            std::process::exit(101);
        }
    }
}

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
            let attach_volume_result = attach_specific_volume(&metadata.instance_id,
                                                              vol.volume_id.as_ref().unwrap(),
                                                              &ec2_client);
            if attach_volume_result.is_ok() {
                info!("successfully attached volume");
                return Ok(());
            }
        }

        info!("all queried volumes have been attempted");
        Err(AttachVolumeError::AllAttachesFailed)
    } else {
        Err(AttachVolumeError::NoVolumesAvailable)
    }
}

fn attach_specific_volume<P, D>(instance_id: &String,
                                volume_id: &String,
                                ec2_client: &Ec2Client<P, D>)
                                -> Result<(), rusoto::ec2::AttachVolumeError>
    where P: ProvideAwsCredentials,
          D: DispatchSignedRequest
{
    let request = AttachVolumeRequest {
        device: String::from("/dev/xvdh"),
        dry_run: None,
        instance_id: instance_id.clone(),
        volume_id: volume_id.clone(),
    };
    try!(ec2_client.attach_volume(&request));
    Ok(())
}
