//! `mys3 bucket-info`: show bucket details (name, creation date, object count, total size).

use serde::Serialize;

use crate::cli::OutputFormat;
use crate::client::{Response, S3Client};
use crate::commands::list_buckets::parse_buckets_xml;
use crate::commands::list_objects::parse_objects_xml;
use crate::error::MyS3Error;

/// Number of keys asked per `ListObjectsV2` page (the S3 maximum).
const PAGE_SIZE: u32 = 1000;

/// Arguments of the `bucket-info` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, ignore_case = true)]
    pub output: OutputFormat,
}

/// Details of a bucket returned by `bucket-info`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BucketInfo {
    pub name: String,
    pub creation_date: String,
    pub object_count: u64,
    pub total_size: u64,
}

/// What one page of a `ListObjectsV2` answer adds to the totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectsPage {
    pub object_count: u64,
    pub total_size: u64,
    /// Token of the next page, `None` on the last page.
    pub next_token: Option<String>,
}

/// Reads one page of a `ListObjectsV2` answer.
pub fn parse_objects_page(xml: &str) -> ObjectsPage {
    let objects = parse_objects_xml(xml);
    let truncated = tag_value(xml, "IsTruncated").as_deref() == Some("true");
    ObjectsPage {
        object_count: objects.len() as u64,
        total_size: objects.iter().map(|object| object.size).sum(),
        next_token: truncated
            .then(|| tag_value(xml, "NextContinuationToken"))
            .flatten(),
    }
}

/// Text of the first `<tag>...</tag>` element of an XML document.
fn tag_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = start + xml[start..].find(&close)?;
    Some(
        xml[start..end]
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&amp;", "&"),
    )
}

/// A size in bytes, followed by a readable size from 1 KiB.
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KiB", "MiB", "GiB", "TiB"];

    let exact = if bytes == 1 {
        "1 byte".to_string()
    } else {
        format!("{bytes} bytes")
    };
    if bytes < 1024 {
        return exact;
    }

    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{exact} ({value:.1} {})", UNITS[unit])
}

/// The details as aligned `label: value` lines.
pub fn format_text(info: &BucketInfo) -> String {
    format!(
        "Name:          {}\n\
         Creation date: {}\n\
         Objects:       {}\n\
         Total size:    {}\n",
        info.name,
        info.creation_date,
        info.object_count,
        format_size(info.total_size)
    )
}

/// Turns any answer other than `200` into an error.
fn expect_ok(response: &Response, bucket: &str) -> Result<(), MyS3Error> {
    match response.status {
        200 => Ok(()),
        404 => Err(MyS3Error::BucketNotFound(bucket.to_string())),
        status => Err(MyS3Error::UnexpectedResponse {
            status,
            code: response.error_code().unwrap_or_default(),
        }),
    }
}

/// Creation date of `bucket`, read from the list of buckets: S3 has no request
/// that returns it for a single bucket. Empty if the bucket is not listed.
fn creation_date(client: &S3Client, bucket: &str) -> Result<String, MyS3Error> {
    let response = client.send("GET", "/", &[], Vec::new(), Vec::new())?;
    expect_ok(&response, bucket)?;
    Ok(parse_buckets_xml(&String::from_utf8_lossy(&response.body))
        .into_iter()
        .find(|item| item.name == bucket)
        .map(|item| item.creation_date)
        .unwrap_or_default())
}

/// Object count and total size of `bucket`, page by page: a `ListObjectsV2`
/// answer holds at most `page_size` objects.
pub fn count_objects(
    client: &S3Client,
    bucket: &str,
    page_size: u32,
) -> Result<(u64, u64), MyS3Error> {
    let path = format!("/{bucket}");
    let max_keys = page_size.to_string();
    let (mut object_count, mut total_size) = (0, 0);
    let mut token: Option<String> = None;

    loop {
        let mut query = vec![("list-type", "2"), ("max-keys", max_keys.as_str())];
        if let Some(token) = &token {
            query.push(("continuation-token", token.as_str()));
        }
        let response = client.send("GET", &path, &query, Vec::new(), Vec::new())?;
        expect_ok(&response, bucket)?;

        let page = parse_objects_page(&String::from_utf8_lossy(&response.body));
        object_count += page.object_count;
        total_size += page.total_size;
        match page.next_token {
            Some(next) => token = Some(next),
            None => return Ok((object_count, total_size)),
        }
    }
}

/// Details of `bucket`. A missing bucket is an error.
pub fn fetch_bucket_info(client: &S3Client, bucket: &str) -> Result<BucketInfo, MyS3Error> {
    let response = client.send("HEAD", &format!("/{bucket}"), &[], Vec::new(), Vec::new())?;
    expect_ok(&response, bucket)?;

    let creation_date = creation_date(client, bucket)?;
    let (object_count, total_size) = count_objects(client, bucket, PAGE_SIZE)?;
    Ok(BucketInfo {
        name: bucket.to_string(),
        creation_date,
        object_count,
        total_size,
    })
}

/// Runs the `bucket-info` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    let client = S3Client::connect(args.alias.as_deref())?;
    let info = fetch_bucket_info(&client, &args.bucket_name)?;

    match args.output {
        OutputFormat::Text => print!("{}", format_text(&info)),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&info)
                .expect("the bucket details are always serializable")
        ),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAST_PAGE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Name>my-bucket</Name>
  <KeyCount>2</KeyCount>
  <IsTruncated>false</IsTruncated>
  <Contents><Key>a.txt</Key><Size>5</Size></Contents>
  <Contents><Key>docs/b.txt</Key><Size>1024</Size></Contents>
</ListBucketResult>"#;

    const FIRST_PAGE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Name>my-bucket</Name>
  <KeyCount>1</KeyCount>
  <IsTruncated>true</IsTruncated>
  <NextContinuationToken>1a2b&amp;3c==</NextContinuationToken>
  <Contents><Key>a.txt</Key><Size>5</Size></Contents>
</ListBucketResult>"#;

    fn info(total_size: u64) -> BucketInfo {
        BucketInfo {
            name: "my-bucket".to_string(),
            creation_date: "2026-10-09T12:22:47.702Z".to_string(),
            object_count: 2,
            total_size,
        }
    }

    #[test]
    fn last_page_counts_objects_and_sums_sizes() {
        assert_eq!(
            parse_objects_page(LAST_PAGE),
            ObjectsPage {
                object_count: 2,
                total_size: 1029,
                next_token: None,
            }
        );
    }

    #[test]
    fn truncated_page_gives_the_next_token() {
        let page = parse_objects_page(FIRST_PAGE);
        assert_eq!(page.object_count, 1);
        assert_eq!(page.next_token.as_deref(), Some("1a2b&3c=="));
    }

    #[test]
    fn truncated_page_without_token_is_the_last() {
        let xml = FIRST_PAGE.replace(
            "<NextContinuationToken>1a2b&amp;3c==</NextContinuationToken>",
            "",
        );
        assert_eq!(parse_objects_page(&xml).next_token, None);
    }

    #[test]
    fn empty_bucket_page_has_no_object() {
        let xml = "<ListBucketResult><KeyCount>0</KeyCount><IsTruncated>false</IsTruncated></ListBucketResult>";
        assert_eq!(
            parse_objects_page(xml),
            ObjectsPage {
                object_count: 0,
                total_size: 0,
                next_token: None,
            }
        );
    }

    #[test]
    fn small_sizes_are_in_bytes() {
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(1), "1 byte");
        assert_eq!(format_size(1023), "1023 bytes");
    }

    #[test]
    fn large_sizes_also_show_a_readable_size() {
        assert_eq!(format_size(1024), "1024 bytes (1.0 KiB)");
        assert_eq!(format_size(1_234_567), "1234567 bytes (1.2 MiB)");
        assert_eq!(
            format_size(5 * 1024 * 1024 * 1024),
            "5368709120 bytes (5.0 GiB)"
        );
    }

    #[test]
    fn text_has_aligned_labels() {
        assert_eq!(
            format_text(&info(1029)),
            "Name:          my-bucket\n\
             Creation date: 2026-10-09T12:22:47.702Z\n\
             Objects:       2\n\
             Total size:    1029 bytes (1.0 KiB)\n"
        );
    }

    #[test]
    fn json_has_exactly_four_keys() {
        let value = serde_json::to_value(info(1029)).unwrap();
        let object = value.as_object().unwrap();
        assert_eq!(object.len(), 4);
        for key in ["name", "creation_date", "object_count", "total_size"] {
            assert!(object.contains_key(key), "missing {key}");
        }
        assert_eq!(object["object_count"], 2);
        assert_eq!(object["total_size"], 1029);
    }
}
