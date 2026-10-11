//! `mys3 upload-file`: upload a local file to a bucket.

use std::fs;
use std::path::{Path, PathBuf};

use crate::client::S3Client;
use crate::commands::object_info::fetch_object_info;
use crate::error::MyS3Error;

/// Arguments of the `upload-file` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Path of the local file to upload
    pub file_path: PathBuf,

    /// Name of the destination bucket
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Object key in the bucket (default: the file name)
    #[arg(long)]
    pub key: Option<String>,

    /// Replace the object if it already exists
    #[arg(long)]
    pub overwrite: bool,
}

/// Key of the uploaded object: `key` if given, otherwise the file name of
/// `file_path` (`None` if the path has no file name, like `..`).
pub fn object_key(file_path: &Path, key: Option<&str>) -> Option<String> {
    match key {
        Some(key) => Some(key.to_string()),
        None => file_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
    }
}

/// Runs the `upload-file` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    let file = args.file_path.display().to_string();
    if !args.file_path.is_file() {
        return Err(MyS3Error::LocalFileNotFound(file));
    }

    let key = object_key(&args.file_path, args.key.as_deref()).unwrap_or_default();
    if key.is_empty() {
        return Err(MyS3Error::ObjectNotFound {
            bucket: args.bucket_name,
            key,
        });
    }

    let body = fs::read(&args.file_path).map_err(|e| MyS3Error::LocalFileRead(file.clone(), e))?;

    let client = S3Client::connect(args.alias.as_deref())?;

    // Overwrite protection; also reports a missing bucket.
    if !args.overwrite {
        match fetch_object_info(&client, &args.bucket_name, &key) {
            Ok(_) => {
                return Err(MyS3Error::ObjectAlreadyExists {
                    bucket: args.bucket_name,
                    key,
                });
            }
            Err(MyS3Error::ObjectNotFound { .. }) => {}
            Err(err) => return Err(err),
        }
    }

    let path = format!("/{}/{}", args.bucket_name, key);
    let response = client.send("PUT", &path, &[], Vec::new(), body)?;

    match response.status {
        200 => {
            println!(
                "Uploaded '{}' to bucket '{}' as '{}'.",
                file, args.bucket_name, key
            );
            Ok(())
        }
        404 if response.error_code().as_deref() == Some("NoSuchBucket") => {
            Err(MyS3Error::BucketNotFound(args.bucket_name))
        }
        status => Err(MyS3Error::UnexpectedResponse {
            status,
            code: response.error_code().unwrap_or_default(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_key_is_the_file_name() {
        assert_eq!(
            object_key(Path::new("report.pdf"), None).as_deref(),
            Some("report.pdf")
        );
        assert_eq!(
            object_key(Path::new("documents/2026/report.pdf"), None).as_deref(),
            Some("report.pdf")
        );
    }

    #[test]
    fn explicit_key_is_respected() {
        assert_eq!(
            object_key(Path::new("report.pdf"), Some("archive/2026-report.pdf")).as_deref(),
            Some("archive/2026-report.pdf")
        );
    }

    #[test]
    fn path_without_file_name_has_no_key() {
        assert_eq!(object_key(Path::new(".."), None), None);
    }
}
