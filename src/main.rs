
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rusoto;

use rusoto::{DefaultCredentialsProvider, Region};
use rusoto::ec2::{Ec2Client, DescribeVolumesRequest, Filter};
use rusoto::default_tls_client;

fn main() {
    env_logger::init().unwrap();

    let credentials = DefaultCredentialsProvider::new().unwrap();

    // FIXME: needs to be current region
    let client = Ec2Client::new(default_tls_client().unwrap(), credentials, Region::UsEast1);

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
    match client.describe_volumes(&request) {
        Ok(response) => {
            if response.next_token.is_some() {
                error!("DescribeVolumes returned multiple pages of results; this is not currently supported");
                std::process::exit(100);
            }
            match response.volumes {
                None => {}
                Some(volumes) => {
                    // FIXME: attempt to attach volume to this instance
                    for vol in &volumes {
                        debug!("target volume: {:?}", vol);
                    }
                }
            }
        }
        Err(error) => {
            error!("DescribeVolumes error: {}", error);
            std::process::exit(1);
        }
    }
}
