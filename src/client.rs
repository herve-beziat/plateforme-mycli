//! HTTP client: sends requests signed with SigV4 to the server of an alias.
//!
//! Path-style addressing only: the bucket is the first segment of the path
//! (`http://host:port/<bucket>/<key>`).

use chrono::Utc;
use sha2::{Digest, Sha256};
use ureq::Agent;
use ureq::http::{self, Uri};

use crate::config::ResolvedAlias;
use crate::error::MyS3Error;
use crate::signer;

/// A client bound to one alias.
pub struct S3Client {
    agent: Agent,
    /// `scheme://host[:port]`, without a trailing slash.
    base_url: String,
    /// `host[:port]`, without the default port of the scheme: the value of the
    /// `Host` header, which is signed.
    host: String,
    alias: ResolvedAlias,
}

/// Answer of the server. Every status is returned except `403`, which is
/// turned into `MyS3Error::AuthenticationRefused`.
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl S3Client {
    /// Checks the URL of the alias (`http://` or `https://`, no path) and
    /// prepares the client.
    pub fn new(alias: ResolvedAlias) -> Result<Self, MyS3Error> {
        let invalid = || MyS3Error::InvalidUrl(alias.url.clone());

        let uri: Uri = alias.url.parse().map_err(|_| invalid())?;
        let (scheme, default_port) = match uri.scheme_str() {
            Some("http") => ("http", 80),
            Some("https") => ("https", 443),
            _ => return Err(invalid()),
        };
        let host = uri.host().ok_or_else(invalid)?;
        if !matches!(uri.path(), "" | "/") || uri.query().is_some() {
            return Err(invalid());
        }
        // Same rule as the `Host` header sent by ureq, otherwise the signature does not match.
        let host = match uri.port_u16() {
            Some(port) if port != default_port => format!("{host}:{port}"),
            _ => host.to_string(),
        };

        let agent = Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .into();

        Ok(Self {
            agent,
            base_url: format!("{scheme}://{host}"),
            host,
            alias,
        })
    }

    /// Signs and sends a request. `path` and `query` are not URI-encoded.
    pub fn send(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    ) -> Result<Response, MyS3Error> {
        let mut request = signer::Request {
            method: method.to_string(),
            host: self.host.clone(),
            path: path.to_string(),
            query: query
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
            headers,
            payload_hash: hex::encode(Sha256::digest(&body)),
        };
        signer::sign(
            &mut request,
            &self.alias.access_key,
            &self.alias.secret_key,
            &self.alias.region,
            Utc::now(),
        );

        let url = self.url(path, query);
        let mut builder = http::Request::builder().method(method).uri(&url);
        for (name, value) in &request.headers {
            builder = builder.header(name, value);
        }
        let invalid = |_| MyS3Error::InvalidUrl(url.clone());
        // ureq refuses a body on GET, HEAD and DELETE, even an empty one.
        let result = if body.is_empty() {
            self.agent.run(builder.body(()).map_err(invalid)?)
        } else {
            self.agent.run(builder.body(body).map_err(invalid)?)
        };
        let unreachable = |_| MyS3Error::ServerUnreachable(self.base_url.clone());
        let mut response = result.map_err(unreachable)?;

        let status = response.status().as_u16();
        if status == 403 {
            return Err(MyS3Error::AuthenticationRefused);
        }
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                let value = value.to_str().unwrap_or_default();
                (name.to_string(), value.to_string())
            })
            .collect();
        // `with_config` reads without the 10 MB limit of `read_to_vec`.
        let body = response
            .body_mut()
            .with_config()
            .read_to_vec()
            .map_err(unreachable)?;

        Ok(Response {
            status,
            headers,
            body,
        })
    }

    /// URL sent to the server, encoded exactly as in the signed canonical request.
    fn url(&self, path: &str, query: &[(&str, &str)]) -> String {
        let mut url = format!("{}{}", self.base_url, signer::uri_encode(path, false));
        if !query.is_empty() {
            let query = query
                .iter()
                .map(|(name, value)| {
                    format!(
                        "{}={}",
                        signer::uri_encode(name, true),
                        signer::uri_encode(value, true)
                    )
                })
                .collect::<Vec<_>>()
                .join("&");
            url.push('?');
            url.push_str(&query);
        }
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alias(url: &str) -> ResolvedAlias {
        ResolvedAlias {
            name: "test".to_string(),
            url: url.to_string(),
            access_key: "access".to_string(),
            secret_key: "secret".to_string(),
            region: "us-east-1".to_string(),
        }
    }

    fn client(url: &str) -> S3Client {
        S3Client::new(alias(url)).unwrap()
    }

    #[test]
    fn host_keeps_a_non_default_port() {
        let client = client("http://localhost:9000");
        assert_eq!(client.host, "localhost:9000");
        assert_eq!(client.base_url, "http://localhost:9000");
    }

    #[test]
    fn host_drops_the_default_port() {
        assert_eq!(client("http://example.com:80").host, "example.com");
        assert_eq!(client("https://s3.example.com:443").host, "s3.example.com");
        assert_eq!(client("https://s3.example.com/").host, "s3.example.com");
    }

    #[test]
    fn invalid_urls_are_refused() {
        for url in [
            "localhost:9000",
            "ftp://localhost:9000",
            "http://localhost:9000/bucket",
            "not a url",
        ] {
            assert!(
                matches!(S3Client::new(alias(url)), Err(MyS3Error::InvalidUrl(_))),
                "{url} should be refused"
            );
        }
    }

    #[test]
    fn url_is_encoded_like_the_signed_request() {
        let client = client("http://localhost:9000");
        assert_eq!(
            client.url(
                "/my-bucket/dir/my file.txt",
                &[("prefix", "a b"), ("list-type", "2")]
            ),
            "http://localhost:9000/my-bucket/dir/my%20file.txt?prefix=a%20b&list-type=2"
        );
    }

    /// Needs MinIO (`docker compose up -d`) and its keys in `MYS3_ACCESS_KEY`
    /// and `MYS3_SECRET_KEY`. Run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn signed_list_buckets_is_accepted_by_minio() {
        let mut alias = alias("http://localhost:9000");
        alias.access_key = std::env::var("MYS3_ACCESS_KEY").expect("MYS3_ACCESS_KEY is not set");
        alias.secret_key = std::env::var("MYS3_SECRET_KEY").expect("MYS3_SECRET_KEY is not set");

        let response = S3Client::new(alias)
            .unwrap()
            .send("GET", "/", &[], Vec::new(), Vec::new())
            .unwrap();
        assert_eq!(response.status, 200);
        assert!(String::from_utf8_lossy(&response.body).contains("ListAllMyBucketsResult"));
    }
}
