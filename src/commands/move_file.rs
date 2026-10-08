//! `mys3 move-file`: move an object between buckets of the same server (copy, then delete the source).

use crate::client::S3Client;
use crate::commands::object_info::fetch_object_info;
use crate::error::MyS3Error;

/// Arguments of the `move-file` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the source bucket
    pub source_bucket: String,

    /// Key of the object in the source bucket
    pub object_key: String,

    /// Name of the destination bucket
    pub destination_bucket: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Key in the destination bucket (default: same key)
    #[arg(long, value_name = "NEW_KEY")]
    pub key: Option<String>,

    /// Replace the object if it already exists at the destination
    #[arg(long)]
    pub overwrite: bool,
}

/// Checks whether a bucket exists using `HEAD /<bucket>`.
fn bucket_exists(client: &S3Client, bucket: &str) -> Result<bool, MyS3Error> {
    let response = client.send("HEAD", &format!("/{bucket}"), &[], Vec::new(), Vec::new())?;
    Ok(response.status != 404)
}

/// Runs the `move-file` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    if args.object_key.is_empty() {
        return Err(MyS3Error::ObjectNotFound {
            bucket: args.source_bucket,
            key: args.object_key,
        });
    }

    let client = S3Client::connect(args.alias.as_deref())?;

    // 1. Verify source object (and source bucket) exists
    fetch_object_info(&client, &args.source_bucket, &args.object_key)?;

    // 2. Verify destination bucket exists
    if !bucket_exists(&client, &args.destination_bucket)? {
        return Err(MyS3Error::BucketNotFound(args.destination_bucket));
    }

    let dest_key = args.key.unwrap_or_else(|| args.object_key.clone());
    if dest_key.is_empty() {
        return Err(MyS3Error::ObjectNotFound {
            bucket: args.destination_bucket,
            key: dest_key,
        });
    }

    // Special case: moving to exactly the same location
    if args.source_bucket == args.destination_bucket && args.object_key == dest_key {
        println!(
            "Moved '{}/{}' to '{}/{}'.",
            args.source_bucket, args.object_key, args.destination_bucket, dest_key
        );
        return Ok(());
    }

    // 3. Overwrite protection check on destination
    if !args.overwrite {
        match fetch_object_info(&client, &args.destination_bucket, &dest_key) {
            Ok(_) => {
                return Err(MyS3Error::ObjectAlreadyExists {
                    bucket: args.destination_bucket,
                    key: dest_key,
                });
            }
            Err(MyS3Error::ObjectNotFound { .. }) => {}
            Err(err) => return Err(err),
        }
    }

    // 4. Perform S3 PUT with x-amz-copy-source header
    let copy_source = format!("/{}/{}", args.source_bucket, args.object_key);
    let dest_path = format!("/{}/{}", args.destination_bucket, dest_key);
    let headers = vec![("x-amz-copy-source".to_string(), copy_source)];

    let copy_response = client.send("PUT", &dest_path, &[], headers, Vec::new())?;

    match copy_response.status {
        200 => {}
        404 => {
            let code = copy_response.error_code().unwrap_or_default();
            if code == "NoSuchBucket" {
                return Err(MyS3Error::BucketNotFound(args.destination_bucket));
            } else {
                return Err(MyS3Error::ObjectNotFound {
                    bucket: args.source_bucket,
                    key: args.object_key,
                });
            }
        }
        status => {
            return Err(MyS3Error::UnexpectedResponse {
                status,
                code: copy_response.error_code().unwrap_or_default(),
            });
        }
    }

    // 5. After successful copy, delete the source object
    let source_path = format!("/{}/{}", args.source_bucket, args.object_key);
    let delete_response = client.send("DELETE", &source_path, &[], Vec::new(), Vec::new())?;

    match delete_response.status {
        200 | 204 => {
            println!(
                "Moved '{}/{}' to '{}/{}'.",
                args.source_bucket, args.object_key, args.destination_bucket, dest_key
            );
            Ok(())
        }
        status => Err(MyS3Error::UnexpectedResponse {
            status,
            code: delete_response.error_code().unwrap_or_default(),
        }),
    }
}
