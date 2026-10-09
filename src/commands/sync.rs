//! `mys3 sync`: one-way sync from a local folder to a bucket.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::client::S3Client;
use crate::error::MyS3Error;
use crate::prompt;

/// Arguments of the `sync` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Local folder whose contents are synchronised
    pub local_folder: PathBuf,

    /// Name of the destination bucket
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Destination prefix in the bucket
    #[arg(long)]
    pub prefix: Option<String>,

    /// Show what would be done without doing it
    #[arg(long)]
    pub dry_run: bool,

    /// Also remove from the bucket the objects that no longer exist locally
    #[arg(long)]
    pub delete: bool,

    /// Skip the confirmation
    #[arg(short, long)]
    pub force: bool,
}

/// S3 object summary extracted from ListObjectsV2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteObject {
    pub key: String,
    pub size: u64,
}

/// Information about a local file to sync.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalFile {
    /// Relative path within the folder, formatted with forward slashes `/`.
    pub rel_key: String,
    /// Absolute or resolved path on disk.
    pub path: PathBuf,
    pub size: u64,
}

/// Checks whether a bucket exists using `HEAD /<bucket>`.
fn bucket_exists(client: &S3Client, bucket: &str) -> Result<bool, MyS3Error> {
    let response = client.send("HEAD", &format!("/{bucket}"), &[], Vec::new(), Vec::new())?;
    Ok(response.status != 404)
}

/// Recursively collects all files in `dir`, returning them with keys relative to `dir`.
pub fn scan_local_folder(dir: &Path) -> Result<Vec<LocalFile>, MyS3Error> {
    if !dir.exists() {
        return Err(MyS3Error::LocalFileNotFound(dir.display().to_string()));
    }
    if !dir.is_dir() {
        return Err(MyS3Error::LocalFileNotFound(format!(
            "'{}' is not a directory",
            dir.display()
        )));
    }

    let mut files = Vec::new();
    collect_files_recursive(dir, dir, &mut files)?;
    Ok(files)
}

fn collect_files_recursive(
    base: &Path,
    current: &Path,
    out: &mut Vec<LocalFile>,
) -> Result<(), MyS3Error> {
    let entries = fs::read_dir(current)
        .map_err(|e| MyS3Error::ConfigRead(current.display().to_string(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| MyS3Error::ConfigRead(current.display().to_string(), e))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|e| MyS3Error::ConfigRead(path.display().to_string(), e))?;

        if metadata.is_dir() {
            collect_files_recursive(base, &path, out)?;
        } else if metadata.is_file() {
            let rel = path
                .strip_prefix(base)
                .map_err(|_| MyS3Error::LocalFileNotFound(path.display().to_string()))?;

            // Convert path segments to `/` delimited S3 keys
            let rel_key = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");

            out.push(LocalFile {
                rel_key,
                path,
                size: metadata.len(),
            });
        }
    }

    Ok(())
}

/// Fetches all objects in the bucket matching the given prefix.
pub fn list_remote_objects(
    client: &S3Client,
    bucket: &str,
    prefix: Option<&str>,
) -> Result<BTreeMap<String, RemoteObject>, MyS3Error> {
    let mut map = BTreeMap::new();
    let mut continuation_token: Option<String> = None;
    let path = format!("/{bucket}");

    loop {
        let mut query_params: Vec<(&str, &str)> = vec![("list-type", "2")];
        if let Some(p) = prefix {
            query_params.push(("prefix", p));
        }
        if let Some(token) = &continuation_token {
            query_params.push(("continuation-token", token.as_str()));
        }

        let response = client.send("GET", &path, &query_params, Vec::new(), Vec::new())?;

        match response.status {
            200 => {}
            404 => return Err(MyS3Error::BucketNotFound(bucket.to_string())),
            status => {
                return Err(MyS3Error::UnexpectedResponse {
                    status,
                    code: response.error_code().unwrap_or_default(),
                });
            }
        }

        let body = String::from_utf8_lossy(&response.body);
        let (objects, is_truncated, next_token) = parse_list_v2_page(&body);

        for obj in objects {
            map.insert(obj.key.clone(), obj);
        }

        if is_truncated && next_token.is_some() {
            continuation_token = next_token;
        } else {
            break;
        }
    }

    Ok(map)
}

fn parse_list_v2_page(xml: &str) -> (Vec<RemoteObject>, bool, Option<String>) {
    let mut objects = Vec::new();
    let open_content = "<Contents>";
    let close_content = "</Contents>";
    let mut rest = xml;

    while let Some(start) = rest.find(open_content) {
        rest = &rest[start + open_content.len()..];
        let Some(end) = rest.find(close_content) else {
            break;
        };
        let block = &rest[..end];
        rest = &rest[end + close_content.len()..];

        if let Some(key) = extract_tag_value(block, "Key") {
            let size = extract_tag_value(block, "Size")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            objects.push(RemoteObject { key, size });
        }
    }

    let is_truncated = extract_tag_value(xml, "IsTruncated")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let next_token = extract_tag_value(xml, "NextContinuationToken");

    (objects, is_truncated, next_token)
}

fn extract_tag_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let after_open = &xml[start + open.len()..];
    let end = after_open.find(&close)?;
    Some(xml_unescape(&after_open[..end]))
}

fn xml_unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// Normalizes prefix: strips leading slashes. If non-empty and doesn't end with `/`, appends `/`.
pub fn normalize_prefix(prefix: Option<&str>) -> String {
    match prefix {
        None => String::new(),
        Some(p) => {
            let trimmed = p.trim_start_matches('/');
            if trimmed.is_empty() {
                String::new()
            } else if trimmed.ends_with('/') {
                trimmed.to_string()
            } else {
                format!("{trimmed}/")
            }
        }
    }
}

/// Runs the `sync` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    let local_files = scan_local_folder(&args.local_folder)?;
    let client = S3Client::connect(args.alias.as_deref())?;

    if !bucket_exists(&client, &args.bucket_name)? {
        return Err(MyS3Error::BucketNotFound(args.bucket_name));
    }

    let prefix_str = normalize_prefix(args.prefix.as_deref());
    let prefix_arg = if prefix_str.is_empty() {
        None
    } else {
        Some(prefix_str.as_str())
    };

    let remote_objects = list_remote_objects(&client, &args.bucket_name, prefix_arg)?;

    // 1. Identify files to upload (missing or different size)
    let mut to_upload: Vec<(LocalFile, String)> = Vec::new();
    let mut expected_remote_keys = BTreeSet::new();

    for local in local_files {
        let remote_key = format!("{}{}", prefix_str, local.rel_key);
        expected_remote_keys.insert(remote_key.clone());

        match remote_objects.get(&remote_key) {
            None => {
                to_upload.push((local, remote_key));
            }
            Some(remote) if remote.size != local.size => {
                to_upload.push((local, remote_key));
            }
            _ => {
                // Same key and size: up to date
            }
        }
    }

    // 2. Identify objects to delete if --delete is requested
    let mut to_delete: Vec<String> = Vec::new();
    if args.delete {
        for remote_key in remote_objects.keys() {
            if !expected_remote_keys.contains(remote_key) {
                to_delete.push(remote_key.clone());
            }
        }
    }

    // Dry-run reporting
    if args.dry_run {
        println!("Dry run summary:");
        if to_upload.is_empty() && to_delete.is_empty() {
            println!("Everything is up to date.");
        }
        for (_, remote_key) in &to_upload {
            println!("upload: {remote_key}");
        }
        for remote_key in &to_delete {
            println!("delete: {remote_key}");
        }
        return Ok(());
    }

    // 3. Confirm deletion if any objects are to be removed
    if !to_delete.is_empty()
        && !args.force
        && !prompt::confirm(&format!(
            "Delete {} remote object(s) missing locally from bucket '{}'?",
            to_delete.len(),
            args.bucket_name
        ))?
    {
        println!("Deletion aborted. Proceeding with upload only.");
        to_delete.clear();
    }

    // 4. Perform uploads
    for (local, remote_key) in &to_upload {
        let content = fs::read(&local.path)
            .map_err(|e| MyS3Error::ConfigRead(local.path.display().to_string(), e))?;

        let path = format!("/{}/{}", args.bucket_name, remote_key);
        let response = client.send("PUT", &path, &[], Vec::new(), content)?;

        if response.status != 200 {
            return Err(MyS3Error::UnexpectedResponse {
                status: response.status,
                code: response.error_code().unwrap_or_default(),
            });
        }
        println!("Uploaded '{remote_key}'.");
    }

    // 5. Perform deletions
    for remote_key in &to_delete {
        let path = format!("/{}/{}", args.bucket_name, remote_key);
        let response = client.send("DELETE", &path, &[], Vec::new(), Vec::new())?;

        if !matches!(response.status, 200 | 204) {
            return Err(MyS3Error::UnexpectedResponse {
                status: response.status,
                code: response.error_code().unwrap_or_default(),
            });
        }
        println!("Deleted '{remote_key}'.");
    }

    if to_upload.is_empty() && to_delete.is_empty() {
        println!(
            "Folder is already in sync with bucket '{}'.",
            args.bucket_name
        );
    } else {
        println!(
            "Sync complete: {} uploaded, {} deleted.",
            to_upload.len(),
            to_delete.len()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_prefix() {
        assert_eq!(normalize_prefix(None), "");
        assert_eq!(normalize_prefix(Some("")), "");
        assert_eq!(normalize_prefix(Some("/")), "");
        assert_eq!(normalize_prefix(Some("backup")), "backup/");
        assert_eq!(normalize_prefix(Some("/backup/")), "backup/");
        assert_eq!(normalize_prefix(Some("backup/data")), "backup/data/");
    }

    #[test]
    fn test_parse_list_v2_page() {
        let xml = r#"<ListBucketResult>
            <Contents>
                <Key>file1.txt</Key>
                <Size>123</Size>
            </Contents>
            <Contents>
                <Key>sub/file2.txt</Key>
                <Size>456</Size>
            </Contents>
            <IsTruncated>false</IsTruncated>
        </ListBucketResult>"#;

        let (objects, truncated, token) = parse_list_v2_page(xml);
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].key, "file1.txt");
        assert_eq!(objects[0].size, 123);
        assert_eq!(objects[1].key, "sub/file2.txt");
        assert_eq!(objects[1].size, 456);
        assert!(!truncated);
        assert_eq!(token, None);
    }
}
