//! `mys3 object-info`: show object details (name, size, last modified, content type, ETag).

use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;

use crate::cli::OutputFormat;
use crate::client::{Response, S3Client};
use crate::error::MyS3Error;

/// Arguments of the `object-info` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket
    pub bucket_name: String,

    /// Key of the object
    pub object_key: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, ignore_case = true)]
    pub output: OutputFormat,
}

/// Details of an object returned by `object-info`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObjectInfo {
    pub name: String,
    pub size: u64,
    pub last_modified: String,
    pub content_type: String,
    pub etag: String,
}

/// Reads the details of the object `key` from the headers of a `HEAD` response.
pub fn parse_object_info(key: &str, response: &Response) -> Result<ObjectInfo, MyS3Error> {
    let header = |name: &str| {
        response
            .headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    };

    let size = header("content-length")
        .and_then(|value| value.trim().parse().ok())
        .ok_or_else(|| MyS3Error::UnexpectedResponse {
            status: response.status,
            code: "missing or invalid Content-Length header".to_string(),
        })?;

    Ok(ObjectInfo {
        name: key.to_string(),
        size,
        last_modified: header("last-modified").map(to_iso_date).unwrap_or_default(),
        content_type: header("content-type").unwrap_or_default().to_string(),
        etag: header("etag")
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
    })
}

/// Converts an HTTP date to ISO 8601 UTC; a date that cannot be read is kept as is.
fn to_iso_date(http_date: &str) -> String {
    DateTime::parse_from_rfc2822(http_date)
        .map(|date| {
            date.with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Secs, true)
        })
        .unwrap_or_else(|_| http_date.to_string())
}

/// The details as aligned `label: value` lines.
pub fn format_text(info: &ObjectInfo) -> String {
    let unit = if info.size == 1 { "byte" } else { "bytes" };
    format!(
        "Name:          {}\n\
         Size:          {} {unit}\n\
         Last modified: {}\n\
         Content type:  {}\n\
         ETag:          {}\n",
        info.name, info.size, info.last_modified, info.content_type, info.etag
    )
}

/// A `HEAD` answer has no body: only a 404 tells that the bucket is missing.
fn bucket_exists(client: &S3Client, bucket: &str) -> Result<bool, MyS3Error> {
    let response = client.send("HEAD", &format!("/{bucket}"), &[], Vec::new(), Vec::new())?;
    Ok(response.status != 404)
}

/// Details of `key` in `bucket`, read with a `HEAD` request.
pub fn fetch_object_info(
    client: &S3Client,
    bucket: &str,
    key: &str,
) -> Result<ObjectInfo, MyS3Error> {
    let response = client.send(
        "HEAD",
        &format!("/{bucket}/{key}"),
        &[],
        Vec::new(),
        Vec::new(),
    )?;
    match response.status {
        200 => parse_object_info(key, &response),
        404 if !bucket_exists(client, bucket)? => {
            Err(MyS3Error::BucketNotFound(bucket.to_string()))
        }
        404 => Err(MyS3Error::ObjectNotFound {
            bucket: bucket.to_string(),
            key: key.to_string(),
        }),
        status => Err(MyS3Error::UnexpectedResponse {
            status,
            code: response.error_code().unwrap_or_default(),
        }),
    }
}

/// Runs the `object-info` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    if args.object_key.is_empty() {
        return Err(MyS3Error::ObjectNotFound {
            bucket: args.bucket_name,
            key: args.object_key,
        });
    }

    let client = S3Client::connect(args.alias.as_deref())?;
    let info = fetch_object_info(&client, &args.bucket_name, &args.object_key)?;

    match args.output {
        OutputFormat::Text => print!("{}", format_text(&info)),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&info)
                .expect("the object details are always serializable")
        ),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(headers: &[(&str, &str)]) -> Response {
        Response {
            status: 200,
            headers: headers
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
            body: Vec::new(),
        }
    }

    fn full() -> Response {
        response(&[
            ("content-length", "1024"),
            ("last-modified", "Thu, 08 Oct 2026 08:21:06 GMT"),
            ("content-type", "application/pdf"),
            ("etag", "\"5d41402abc4b2a76b9719d911017c592\""),
        ])
    }

    #[test]
    fn headers_become_object_info() {
        let info = parse_object_info("docs/report.pdf", &full()).unwrap();
        assert_eq!(
            info,
            ObjectInfo {
                name: "docs/report.pdf".to_string(),
                size: 1024,
                last_modified: "2026-10-08T08:21:06Z".to_string(),
                content_type: "application/pdf".to_string(),
                etag: "5d41402abc4b2a76b9719d911017c592".to_string(),
            }
        );
    }

    #[test]
    fn header_names_are_case_insensitive() {
        let info = parse_object_info(
            "k",
            &response(&[("Content-Length", "3"), ("ETag", "\"abc-3\"")]),
        )
        .unwrap();
        assert_eq!(info.size, 3);
        assert_eq!(info.etag, "abc-3");
    }

    #[test]
    fn http_dates_are_converted_to_iso() {
        assert_eq!(
            to_iso_date("Sun, 06 Nov 1994 08:49:37 GMT"),
            "1994-11-06T08:49:37Z"
        );
        assert_eq!(
            to_iso_date("Thu, 08 Oct 2026 08:21:06 GMT"),
            "2026-10-08T08:21:06Z"
        );
    }

    #[test]
    fn unparsable_date_is_kept() {
        assert_eq!(to_iso_date("yesterday"), "yesterday");
    }

    #[test]
    fn missing_optional_headers_are_empty() {
        let info = parse_object_info("k", &response(&[("content-length", "0")])).unwrap();
        assert_eq!(info.last_modified, "");
        assert_eq!(info.content_type, "");
        assert_eq!(info.etag, "");
    }

    #[test]
    fn missing_or_invalid_content_length_is_an_error() {
        for headers in [vec![], vec![("content-length", "abc")]] {
            assert!(matches!(
                parse_object_info("k", &response(&headers)),
                Err(MyS3Error::UnexpectedResponse { status: 200, .. })
            ));
        }
    }

    #[test]
    fn text_has_aligned_labels() {
        let info = parse_object_info("docs/report.pdf", &full()).unwrap();
        assert_eq!(
            format_text(&info),
            "Name:          docs/report.pdf\n\
             Size:          1024 bytes\n\
             Last modified: 2026-10-08T08:21:06Z\n\
             Content type:  application/pdf\n\
             ETag:          5d41402abc4b2a76b9719d911017c592\n"
        );
    }

    #[test]
    fn text_uses_singular_for_one_byte() {
        let mut info = parse_object_info("k", &full()).unwrap();
        info.size = 1;
        assert!(format_text(&info).contains("1 byte\n"));
    }

    #[test]
    fn json_has_exactly_five_keys() {
        let info = parse_object_info("k", &full()).unwrap();
        let value = serde_json::to_value(&info).unwrap();
        let object = value.as_object().unwrap();
        assert_eq!(object.len(), 5);
        for key in ["name", "size", "last_modified", "content_type", "etag"] {
            assert!(object.contains_key(key), "missing {key}");
        }
        assert!(object["size"].is_u64());
    }
}
