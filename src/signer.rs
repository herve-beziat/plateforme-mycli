//! AWS Signature Version 4 (SigV4) signing for S3 requests.

use chrono::{DateTime, Utc};
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

const ALGORITHM: &str = "AWS4-HMAC-SHA256";

/// A request to sign. `path` and `query` are not URI-encoded.
pub struct Request {
    pub method: String,
    /// With the port when it is not the default one (e.g. `localhost:9000`).
    pub host: String,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    /// Hex SHA-256 of the body.
    pub payload_hash: String,
}

/// SigV4 URI encoding. `/` is kept unless `encode_slash` is true.
pub fn uri_encode(input: &str, encode_slash: bool) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            b'/' if !encode_slash => encoded.push('/'),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

/// Encoded query parameters, sorted by name.
fn canonical_query(query: &[(String, String)]) -> String {
    let mut params: Vec<(String, String)> = query
        .iter()
        .map(|(name, value)| (uri_encode(name, true), uri_encode(value, true)))
        .collect();
    params.sort();
    params
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

/// Returns the canonical headers and the signed header names.
fn canonical_headers(request: &Request, amz_date: &str) -> (String, String) {
    let mut headers = vec![
        ("host".to_string(), request.host.clone()),
        ("x-amz-date".to_string(), amz_date.to_string()),
        (
            "x-amz-content-sha256".to_string(),
            request.payload_hash.clone(),
        ),
    ];
    for (name, value) in &request.headers {
        let name = name.to_lowercase();
        if headers.iter().all(|(signed, _)| *signed != name) {
            headers.push((name, value.split_whitespace().collect::<Vec<_>>().join(" ")));
        }
    }
    headers.sort();

    let block = headers
        .iter()
        .map(|(name, value)| format!("{name}:{value}\n"))
        .collect::<String>();
    let signed_headers = headers
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(";");
    (block, signed_headers)
}

/// Returns the canonical request and the signed header names.
fn canonical_request(request: &Request, amz_date: &str) -> (String, String) {
    let (headers, signed_headers) = canonical_headers(request, amz_date);
    let canonical = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        request.method,
        uri_encode(&request.path, false),
        canonical_query(&request.query),
        headers,
        signed_headers,
        request.payload_hash,
    );
    (canonical, signed_headers)
}

fn credential_scope(date: &str, region: &str) -> String {
    format!("{date}/{region}/s3/aws4_request")
}

fn string_to_sign(amz_date: &str, scope: &str, canonical_request: &str) -> String {
    let hashed_request = hex::encode(Sha256::digest(canonical_request.as_bytes()));
    format!("{ALGORITHM}\n{amz_date}\n{scope}\n{hashed_request}")
}

fn hmac_sha256(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts keys of any length");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn signing_key(secret_key: &str, date: &str, region: &str) -> Vec<u8> {
    let date_key = hmac_sha256(format!("AWS4{secret_key}").as_bytes(), date);
    let region_key = hmac_sha256(&date_key, region);
    let service_key = hmac_sha256(&region_key, "s3");
    hmac_sha256(&service_key, "aws4_request")
}

fn signature(signing_key: &[u8], string_to_sign: &str) -> String {
    hex::encode(hmac_sha256(signing_key, string_to_sign))
}

/// Signs `request` and adds the `Authorization`, `x-amz-date` and
/// `x-amz-content-sha256` headers.
pub fn sign(
    request: &mut Request,
    access_key: &str,
    secret_key: &str,
    region: &str,
    timestamp: DateTime<Utc>,
) {
    request.headers.retain(|(name, _)| {
        !matches!(
            name.to_lowercase().as_str(),
            "authorization" | "x-amz-date" | "x-amz-content-sha256"
        )
    });

    let amz_date = timestamp.format("%Y%m%dT%H%M%SZ").to_string();
    let date = timestamp.format("%Y%m%d").to_string();
    let scope = credential_scope(&date, region);

    let (canonical, signed_headers) = canonical_request(request, &amz_date);
    let key = signing_key(secret_key, &date, region);
    let sig = signature(&key, &string_to_sign(&amz_date, &scope, &canonical));

    let authorization = format!(
        "{ALGORITHM} Credential={access_key}/{scope}, SignedHeaders={signed_headers}, Signature={sig}"
    );
    let payload_hash = request.payload_hash.clone();
    request.headers.extend([
        ("Authorization".to_string(), authorization),
        ("x-amz-date".to_string(), amz_date),
        ("x-amz-content-sha256".to_string(), payload_hash),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // Examples from https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html
    const ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
    const SECRET_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
    const EMPTY_PAYLOAD_HASH: &str =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    fn authorization(
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        headers: &[(&str, &str)],
        payload_hash: &str,
    ) -> String {
        let mut request = Request {
            method: method.to_string(),
            host: "examplebucket.s3.amazonaws.com".to_string(),
            path: path.to_string(),
            query: query
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
            headers: headers
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
            payload_hash: payload_hash.to_string(),
        };
        let timestamp = Utc.with_ymd_and_hms(2013, 5, 24, 0, 0, 0).unwrap();
        sign(&mut request, ACCESS_KEY, SECRET_KEY, "us-east-1", timestamp);

        request
            .headers
            .into_iter()
            .find(|(name, _)| name == "Authorization")
            .map(|(_, value)| value)
            .unwrap()
    }

    #[test]
    fn get_object() {
        assert_eq!(
            authorization(
                "GET",
                "/test.txt",
                &[],
                &[("Range", "bytes=0-9")],
                EMPTY_PAYLOAD_HASH,
            ),
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request, \
             SignedHeaders=host;range;x-amz-content-sha256;x-amz-date, \
             Signature=f0e8bdb87c964420e857bd35b5d6ed310bd44f0170aba48dd91039c6036bdb41"
        );
    }

    #[test]
    fn put_object() {
        assert_eq!(
            authorization(
                "PUT",
                "/test$file.text",
                &[],
                &[
                    ("Date", "Fri, 24 May 2013 00:00:00 GMT"),
                    ("x-amz-storage-class", "REDUCED_REDUNDANCY"),
                ],
                // SHA-256 of "Welcome to Amazon S3."
                "44ce7dd67c959e0d3524ffac1771dfbba87d2b6b4b4e99e42034a8b803f8b072",
            ),
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request, \
             SignedHeaders=date;host;x-amz-content-sha256;x-amz-date;x-amz-storage-class, \
             Signature=98ad721746da40c64f1a55b78f14c238d841ea1380cd77a1b5971af0ece108bd"
        );
    }

    #[test]
    fn list_objects() {
        assert_eq!(
            authorization(
                "GET",
                "/",
                &[("max-keys", "2"), ("prefix", "J")],
                &[],
                EMPTY_PAYLOAD_HASH,
            ),
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request, \
             SignedHeaders=host;x-amz-content-sha256;x-amz-date, \
             Signature=34b48302e7b5fa45bde8084f4b7868a86f0a534bc59db6670ed5711ef69dc6f7"
        );
    }
}
