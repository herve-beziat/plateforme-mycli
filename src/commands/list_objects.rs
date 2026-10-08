//! `mys3 list-objects`: list the objects of a bucket.

use serde::Serialize;

use crate::cli::OutputFormat;
use crate::client::S3Client;
use crate::config::{Config, Credentials};
use crate::error::MyS3Error;

/// Arguments of the `list-objects` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Only list the keys starting with this prefix
    #[arg(long)]
    pub prefix: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, ignore_case = true)]
    pub output: OutputFormat,
}

/// Object item returned by `list-objects`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObjectItem {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
}

/// Parses `<Contents>...</Contents>` entries from an S3 `ListBucketResult` XML body.
pub fn parse_objects_xml(xml: &str) -> Vec<ObjectItem> {
    let mut objects = Vec::new();
    let open_content = "<Contents>";
    let close_content = "</Contents>";
    let mut rest = xml;

    while let Some(start) = rest.find(open_content) {
        rest = &rest[start + open_content.len()..];
        let Some(end) = rest.find(close_content) else {
            break;
        };
        let content_block = &rest[..end];
        rest = &rest[end + close_content.len()..];

        if let Some(key) = extract_tag_value(content_block, "Key") {
            let last_modified =
                extract_tag_value(content_block, "LastModified").unwrap_or_default();
            let size = extract_tag_value(content_block, "Size")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);

            objects.push(ObjectItem {
                key,
                size,
                last_modified,
            });
        }
    }

    objects
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

/// Runs the `list-objects` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    let config = Config::load()?;
    let resolved = config.resolve(
        args.alias.as_deref(),
        Credentials::default(),
        Credentials::from_env(),
    )?;

    let client = S3Client::new(resolved)?;
    let path = format!("/{}", args.bucket_name);

    let mut query_params: Vec<(&str, &str)> = vec![("list-type", "2")];
    if let Some(prefix) = &args.prefix {
        query_params.push(("prefix", prefix.as_str()));
    }

    let response = client.send("GET", &path, &query_params, Vec::new(), Vec::new())?;

    match response.status {
        200 => {}
        404 => return Err(MyS3Error::BucketNotFound(args.bucket_name)),
        status => {
            return Err(MyS3Error::ServerUnreachable(format!(
                "server returned unexpected HTTP status {status}"
            )));
        }
    }

    let xml_body = String::from_utf8_lossy(&response.body);
    let objects = parse_objects_xml(&xml_body);

    match args.output {
        OutputFormat::Text => {
            for object in &objects {
                println!("{}", object.key);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&objects)
                .map_err(|e| MyS3Error::InvalidConfig("json serialization".to_string(), e))?;
            println!("{json}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_objects_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Name>empty-bucket</Name>
    <Prefix></Prefix>
    <KeyCount>0</KeyCount>
    <MaxKeys>1000</MaxKeys>
    <IsTruncated>false</IsTruncated>
</ListBucketResult>"#;
        let objects = parse_objects_xml(xml);
        assert!(objects.is_empty());
    }

    #[test]
    fn parse_multiple_objects_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Name>my-bucket</Name>
    <Prefix></Prefix>
    <KeyCount>2</KeyCount>
    <MaxKeys>1000</MaxKeys>
    <IsTruncated>false</IsTruncated>
    <Contents>
        <Key>file1.txt</Key>
        <LastModified>2026-10-08T08:21:06.475Z</LastModified>
        <ETag>&#34;5d41402abc4b2a76b9719d911017c592&#34;</ETag>
        <Size>1024</Size>
        <StorageClass>STANDARD</StorageClass>
    </Contents>
    <Contents>
        <Key>folder/file2.txt</Key>
        <LastModified>2026-10-08T08:22:00.000Z</LastModified>
        <ETag>&#34;7d793037a0760186574b0282f2f435e7&#34;</ETag>
        <Size>2048</Size>
        <StorageClass>STANDARD</StorageClass>
    </Contents>
</ListBucketResult>"#;
        let objects = parse_objects_xml(xml);
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].key, "file1.txt");
        assert_eq!(objects[0].size, 1024);
        assert_eq!(objects[0].last_modified, "2026-10-08T08:21:06.475Z");
        assert_eq!(objects[1].key, "folder/file2.txt");
        assert_eq!(objects[1].size, 2048);
        assert_eq!(objects[1].last_modified, "2026-10-08T08:22:00.000Z");
    }

    #[test]
    fn parse_objects_xml_unescapes_entities() {
        let xml = r#"<ListBucketResult>
    <Contents>
        <Key>notes/foo &amp; bar.txt</Key>
        <LastModified>2026-10-08T00:00:00.000Z</LastModified>
        <Size>42</Size>
    </Contents>
</ListBucketResult>"#;
        let objects = parse_objects_xml(xml);
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].key, "notes/foo & bar.txt");
        assert_eq!(objects[0].size, 42);
    }
}
