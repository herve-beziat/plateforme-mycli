//! `mys3 create-bucket`: create a bucket.

use crate::client::S3Client;
use crate::error::MyS3Error;

/// Region where S3 creates a bucket when no location constraint is sent.
const DEFAULT_REGION: &str = "us-east-1";

/// Arguments of the `create-bucket` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket to create
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Region of the bucket
    #[arg(long)]
    pub region: Option<String>,
}

/// Checks the S3 bucket naming rules.
pub fn validate_bucket_name(name: &str) -> Result<(), MyS3Error> {
    let invalid = || Err(MyS3Error::InvalidBucketName(name.to_string()));

    if !(3..=63).contains(&name.len()) {
        return invalid();
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
    {
        return invalid();
    }
    let is_alphanumeric = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    if !is_alphanumeric(name.as_bytes()[0]) || !is_alphanumeric(name.as_bytes()[name.len() - 1]) {
        return invalid();
    }
    if name.contains("..") || name.parse::<std::net::Ipv4Addr>().is_ok() {
        return invalid();
    }
    if ["xn--", "sthree-"].iter().any(|p| name.starts_with(p))
        || ["-s3alias", "--ol-s3"].iter().any(|s| name.ends_with(s))
    {
        return invalid();
    }
    Ok(())
}

/// Body of the `PUT` request: empty for `us-east-1`, otherwise the location constraint.
pub fn location_body(region: &str) -> Vec<u8> {
    if region == DEFAULT_REGION {
        return Vec::new();
    }
    format!(
        "<CreateBucketConfiguration xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">\
         <LocationConstraint>{region}</LocationConstraint>\
         </CreateBucketConfiguration>"
    )
    .into_bytes()
}

/// Creates `bucket` in `region` on the server of `client`.
pub fn create_bucket(client: &S3Client, bucket: &str, region: &str) -> Result<(), MyS3Error> {
    let response = client.send(
        "PUT",
        &format!("/{bucket}"),
        &[],
        Vec::new(),
        location_body(region),
    )?;
    let code = response.error_code().unwrap_or_default();

    match (response.status, code.as_str()) {
        (200, _) => Ok(()),
        (409, _) => Err(MyS3Error::BucketAlreadyExists(bucket.to_string())),
        (400, "InvalidBucketName") => Err(MyS3Error::InvalidBucketName(bucket.to_string())),
        (status, _) => Err(MyS3Error::UnexpectedResponse { status, code }),
    }
}

/// Runs the `create-bucket` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    validate_bucket_name(&args.bucket_name)?;

    let client = S3Client::connect(args.alias.as_deref())?;
    let region = args.region.as_deref().unwrap_or(client.region());
    create_bucket(&client, &args.bucket_name, region)?;

    println!("Bucket '{}' created.", args.bucket_name);
    Ok(())
}
