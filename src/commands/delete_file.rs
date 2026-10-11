//! `mys3 delete-file`: delete an object.

use crate::client::S3Client;
use crate::commands::object_info::fetch_object_info;
use crate::error::MyS3Error;
use crate::prompt;

/// Arguments of the `delete-file` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket
    pub bucket_name: String,

    /// Key of the object to delete
    pub object_key: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Skip the confirmation
    #[arg(short, long)]
    pub force: bool,
}

/// Runs the `delete-file` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    if args.object_key.is_empty() {
        return Err(MyS3Error::ObjectNotFound {
            bucket: args.bucket_name,
            key: args.object_key,
        });
    }

    let client = S3Client::connect(args.alias.as_deref())?;

    // Verify object and bucket existence first
    fetch_object_info(&client, &args.bucket_name, &args.object_key)?;

    if !args.force
        && !prompt::confirm(&format!(
            "Delete object '{}' from bucket '{}'?",
            args.object_key, args.bucket_name
        ))?
    {
        println!("Aborted.");
        return Ok(());
    }

    let path = format!("/{}/{}", args.bucket_name, args.object_key);
    let response = client.send("DELETE", &path, &[], Vec::new(), Vec::new())?;

    match response.status {
        200 | 204 => {
            println!(
                "Object '{}' deleted from bucket '{}'.",
                args.object_key, args.bucket_name
            );
            Ok(())
        }
        404 => {
            let code = response.error_code().unwrap_or_default();
            if code == "NoSuchBucket" {
                Err(MyS3Error::BucketNotFound(args.bucket_name))
            } else {
                Err(MyS3Error::ObjectNotFound {
                    bucket: args.bucket_name,
                    key: args.object_key,
                })
            }
        }
        status => Err(MyS3Error::UnexpectedResponse {
            status,
            code: response.error_code().unwrap_or_default(),
        }),
    }
}
