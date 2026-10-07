use mys3::client::S3Client;
use mys3::commands::create_bucket::{create_bucket, location_body, validate_bucket_name};
use mys3::config::ResolvedAlias;
use mys3::error::MyS3Error;

#[test]
fn test_valid_bucket_names() {
    for name in ["my-bucket", "abc", "my.bucket.01", &"a".repeat(63)] {
        assert!(
            validate_bucket_name(name).is_ok(),
            "'{name}' should be valid"
        );
    }
}

#[test]
fn test_invalid_bucket_names() {
    for name in [
        "ab",
        &"a".repeat(64),
        "My-Bucket",
        "my_bucket",
        "-bucket",
        "bucket-",
        "my..bucket",
        "192.168.0.1",
        "xn--bucket",
        "bucket-s3alias",
    ] {
        assert!(
            matches!(
                validate_bucket_name(name),
                Err(MyS3Error::InvalidBucketName(_))
            ),
            "'{name}' should be invalid"
        );
    }
}

#[test]
fn test_location_body_is_empty_for_us_east_1() {
    assert!(location_body("us-east-1").is_empty());
}

#[test]
fn test_location_body_contains_the_region() {
    let body = String::from_utf8(location_body("eu-west-1")).unwrap();
    assert!(body.contains("<LocationConstraint>eu-west-1</LocationConstraint>"));
}

/// Client on the local MinIO, with the keys of `MYS3_ACCESS_KEY` / `MYS3_SECRET_KEY`.
fn minio_client() -> S3Client {
    S3Client::new(ResolvedAlias {
        name: "test".to_string(),
        url: "http://localhost:9000".to_string(),
        access_key: std::env::var("MYS3_ACCESS_KEY").expect("MYS3_ACCESS_KEY is not set"),
        secret_key: std::env::var("MYS3_SECRET_KEY").expect("MYS3_SECRET_KEY is not set"),
        region: "us-east-1".to_string(),
    })
    .unwrap()
}

/// Needs MinIO (`docker compose up -d`). Run with `cargo test -- --ignored`.
#[test]
#[ignore]
fn test_create_bucket_then_already_exists() {
    let client = minio_client();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let bucket = format!("mys3-test-{nanos}");

    let first = create_bucket(&client, &bucket, "us-east-1");
    let second = create_bucket(&client, &bucket, "us-east-1");
    client
        .send("DELETE", &format!("/{bucket}"), &[], Vec::new(), Vec::new())
        .unwrap();

    assert!(first.is_ok(), "{first:?}");
    assert!(
        matches!(second, Err(MyS3Error::BucketAlreadyExists(_))),
        "{second:?}"
    );
}
