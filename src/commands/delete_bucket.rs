//! `mys3 delete-bucket`: delete a bucket.

use crate::client::S3Client;
use crate::commands::list_objects::parse_objects_xml;
use crate::error::MyS3Error;
use crate::prompt;

/// Arguments of the `delete-bucket` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket to delete
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Skip the confirmation
    #[arg(short, long)]
    pub force: bool,

    /// Delete the objects of the bucket first (required for a non-empty bucket)
    #[arg(long)]
    pub recursive: bool,
}

/// Whether `bucket` exists on the server of `client`.
pub fn bucket_exists(client: &S3Client, bucket: &str) -> Result<bool, MyS3Error> {
    let response = client.send("HEAD", &format!("/{bucket}"), &[], Vec::new(), Vec::new())?;
    match response.status {
        200 => Ok(true),
        404 => Ok(false),
        status => Err(MyS3Error::UnexpectedResponse {
            status,
            code: response.error_code().unwrap_or_default(),
        }),
    }
}

/// Deletes every object of `bucket`.
///
/// The listing returns at most 1000 keys per page: the page is deleted, then
/// listed again, until the bucket is empty.
pub fn empty_bucket(client: &S3Client, bucket: &str) -> Result<(), MyS3Error> {
    loop {
        let response = client.send(
            "GET",
            &format!("/{bucket}"),
            &[("list-type", "2")],
            Vec::new(),
            Vec::new(),
        )?;
        match response.status {
            200 => {}
            404 => return Err(MyS3Error::BucketNotFound(bucket.to_string())),
            status => {
                let code = response.error_code().unwrap_or_default();
                return Err(MyS3Error::UnexpectedResponse { status, code });
            }
        }

        let objects = parse_objects_xml(&String::from_utf8_lossy(&response.body));
        if objects.is_empty() {
            return Ok(());
        }
        for object in objects {
            let response = client.send(
                "DELETE",
                &format!("/{bucket}/{}", object.key),
                &[],
                Vec::new(),
                Vec::new(),
            )?;
            // 404: the object is already gone, which is what we want.
            if !matches!(response.status, 200 | 204 | 404) {
                let code = response.error_code().unwrap_or_default();
                return Err(MyS3Error::UnexpectedResponse {
                    status: response.status,
                    code,
                });
            }
        }
    }
}

/// Deletes `bucket`, which must be empty, on the server of `client`.
pub fn delete_bucket(client: &S3Client, bucket: &str) -> Result<(), MyS3Error> {
    let response = client.send("DELETE", &format!("/{bucket}"), &[], Vec::new(), Vec::new())?;
    let code = response.error_code().unwrap_or_default();

    match (response.status, code.as_str()) {
        (200 | 204, _) => Ok(()),
        (404, _) => Err(MyS3Error::BucketNotFound(bucket.to_string())),
        (409, "BucketNotEmpty") => Err(MyS3Error::BucketNotEmpty(bucket.to_string())),
        (status, _) => Err(MyS3Error::UnexpectedResponse { status, code }),
    }
}

/// Runs the `delete-bucket` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    let client = S3Client::connect(args.alias.as_deref())?;
    let bucket = &args.bucket_name;

    // Checked first, so that no confirmation is asked for a missing bucket.
    if !bucket_exists(&client, bucket)? {
        return Err(MyS3Error::BucketNotFound(bucket.clone()));
    }
    // With --recursive, this single question covers the objects and the bucket.
    if !args.force && !prompt::confirm(&format!("Delete bucket '{bucket}'?"))? {
        println!("Aborted.");
        return Ok(());
    }
    if args.recursive {
        empty_bucket(&client, bucket)?;
    }
    delete_bucket(&client, bucket)?;

    println!("Bucket '{bucket}' deleted.");
    Ok(())
}
