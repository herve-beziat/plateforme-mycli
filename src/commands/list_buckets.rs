//! `mys3 list-buckets`: list all buckets on the server.

use serde::Serialize;

use crate::cli::OutputFormat;
use crate::client::S3Client;
use crate::config::{Config, Credentials};
use crate::error::MyS3Error;

/// Arguments of the `list-buckets` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, ignore_case = true)]
    pub output: OutputFormat,
}

/// Bucket item returned by `list-buckets`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BucketItem {
    pub name: String,
    pub creation_date: String,
}

/// Parses `<Bucket>...</Bucket>` entries from an S3 `ListAllMyBucketsResult` XML body.
pub fn parse_buckets_xml(xml: &str) -> Vec<BucketItem> {
    let mut buckets = Vec::new();
    let open_bucket = "<Bucket>";
    let close_bucket = "</Bucket>";
    let mut rest = xml;

    while let Some(start) = rest.find(open_bucket) {
        rest = &rest[start + open_bucket.len()..];
        let Some(end) = rest.find(close_bucket) else {
            break;
        };
        let bucket_content = &rest[..end];
        rest = &rest[end + close_bucket.len()..];

        if let Some(name) = extract_tag_value(bucket_content, "Name") {
            let creation_date =
                extract_tag_value(bucket_content, "CreationDate").unwrap_or_default();
            buckets.push(BucketItem {
                name,
                creation_date,
            });
        }
    }

    buckets
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

/// Runs the `list-buckets` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    let config = Config::load()?;
    let resolved = config.resolve(
        args.alias.as_deref(),
        Credentials::default(),
        Credentials::from_env(),
    )?;

    let client = S3Client::new(resolved)?;
    let response = client.send("GET", "/", &[], Vec::new(), Vec::new())?;

    if response.status != 200 {
        return Err(MyS3Error::ServerUnreachable(format!(
            "server returned unexpected HTTP status {}",
            response.status
        )));
    }

    let xml_body = String::from_utf8_lossy(&response.body);
    let buckets = parse_buckets_xml(&xml_body);

    match args.output {
        OutputFormat::Text => {
            if buckets.is_empty() {
                println!("No buckets found.");
            } else {
                for bucket in &buckets {
                    if bucket.creation_date.is_empty() {
                        println!("{}", bucket.name);
                    } else {
                        println!("{}\t{}", bucket.name, bucket.creation_date);
                    }
                }
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&buckets)
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
    fn parse_empty_buckets_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Owner><ID>123</ID><DisplayName>minio</DisplayName></Owner>
    <Buckets></Buckets>
</ListAllMyBucketsResult>"#;
        let buckets = parse_buckets_xml(xml);
        assert!(buckets.is_empty());
    }

    #[test]
    fn parse_multiple_buckets_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Owner><ID>123</ID><DisplayName>minio</DisplayName></Owner>
    <Buckets>
        <Bucket>
            <Name>first-bucket</Name>
            <CreationDate>2026-01-01T10:00:00.000Z</CreationDate>
        </Bucket>
        <Bucket>
            <Name>second-bucket</Name>
            <CreationDate>2026-02-02T12:00:00.000Z</CreationDate>
        </Bucket>
    </Buckets>
</ListAllMyBucketsResult>"#;
        let buckets = parse_buckets_xml(xml);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].name, "first-bucket");
        assert_eq!(buckets[0].creation_date, "2026-01-01T10:00:00.000Z");
        assert_eq!(buckets[1].name, "second-bucket");
        assert_eq!(buckets[1].creation_date, "2026-02-02T12:00:00.000Z");
    }

    #[test]
    fn parse_buckets_xml_unescapes_entities() {
        let xml = r#"<ListAllMyBucketsResult>
    <Buckets>
        <Bucket>
            <Name>test&amp;bucket</Name>
            <CreationDate>2026-01-01T00:00:00.000Z</CreationDate>
        </Bucket>
    </Buckets>
</ListAllMyBucketsResult>"#;
        let buckets = parse_buckets_xml(xml);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].name, "test&bucket");
    }
}
