//! `mys3 download-file`: download an object to the local disk.

use std::fs;
use std::path::{Path, PathBuf};

use crate::client::S3Client;
use crate::error::MyS3Error;

/// Arguments of the `download-file` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket
    pub bucket_name: String,

    /// Key of the object to download
    pub object_key: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Local destination path (default: current directory, object name)
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Replace the local file if it already exists
    #[arg(long)]
    pub overwrite: bool,
}

/// Determines the local destination path.
///
/// If `output` is not specified, defaults to the file name from the object key
/// in the current directory (or the whole key if no `/` is present).
/// If `output` points to an existing directory, appends the file name to that directory.
pub fn resolve_destination(key: &str, output: Option<&Path>) -> PathBuf {
    let file_name = key.rsplit('/').next().unwrap_or(key);
    // If key ends with `/` or produces empty file name, fallback to whole key or "download"
    let file_name = if file_name.is_empty() {
        "download"
    } else {
        file_name
    };

    match output {
        None => PathBuf::from(file_name),
        Some(path) => {
            if path.is_dir() {
                path.join(file_name)
            } else {
                path.to_path_buf()
            }
        }
    }
}

/// Downloads an object from S3 and saves it to disk.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    if args.object_key.is_empty() {
        return Err(MyS3Error::ObjectNotFound {
            bucket: args.bucket_name,
            key: args.object_key,
        });
    }

    let destination = resolve_destination(&args.object_key, args.output.as_deref());

    if destination.exists() && !args.overwrite {
        return Err(MyS3Error::LocalFileAlreadyExists(
            destination.display().to_string(),
        ));
    }

    let client = S3Client::connect(args.alias.as_deref())?;
    let path = format!("/{}/{}", args.bucket_name, args.object_key);

    let response = client.send("GET", &path, &[], Vec::new(), Vec::new())?;

    match response.status {
        200 => {}
        404 => {
            let code = response.error_code().unwrap_or_default();
            if code == "NoSuchBucket" {
                return Err(MyS3Error::BucketNotFound(args.bucket_name));
            } else {
                return Err(MyS3Error::ObjectNotFound {
                    bucket: args.bucket_name,
                    key: args.object_key,
                });
            }
        }
        status => {
            return Err(MyS3Error::UnexpectedResponse {
                status,
                code: response.error_code().unwrap_or_default(),
            });
        }
    }

    // Ensure parent directory exists if user specified nested path
    if let Some(parent) = destination.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent)
            .map_err(|e| MyS3Error::ConfigWrite(parent.display().to_string(), e))?;
    }

    fs::write(&destination, &response.body)
        .map_err(|e| MyS3Error::ConfigWrite(destination.display().to_string(), e))?;

    println!(
        "Downloaded '{}' from bucket '{}' to '{}'.",
        args.object_key,
        args.bucket_name,
        destination.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_destination_is_filename_in_current_dir() {
        assert_eq!(
            resolve_destination("report.pdf", None),
            PathBuf::from("report.pdf")
        );
        assert_eq!(
            resolve_destination("documents/2026/report.pdf", None),
            PathBuf::from("report.pdf")
        );
    }

    #[test]
    fn explicit_destination_is_respected() {
        let explicit = Path::new("custom/path/file.txt");
        assert_eq!(
            resolve_destination("report.pdf", Some(explicit)),
            PathBuf::from("custom/path/file.txt")
        );
    }
}
