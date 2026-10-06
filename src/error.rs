//! Shared error type of the CLI.
//!
//! Every command returns `Result<(), MyS3Error>`. `main.rs` prints the message
//! on stderr and exits with code 1.

use thiserror::Error;

/// All the errors a `mys3` command can report to the user.
#[derive(Debug, Error)]
#[allow(dead_code)] // Variants get used as commands are implemented.
pub enum MyS3Error {
    /// The server did not answer (wrong URL, server down, timeout).
    #[error("cannot reach server at '{0}'")]
    ServerUnreachable(String),

    /// The server refused the access key, the secret key or the signature.
    #[error("authentication refused by the server (check your access key and secret key)")]
    AuthenticationRefused,

    /// The alias given with `--alias` or `alias use` does not exist.
    #[error("alias '{0}' not found (run `mys3 alias list` to see your aliases)")]
    AliasNotFound(String),

    /// No `--alias` was given and no default alias is set.
    #[error("no default alias set (run `mys3 alias set` or `mys3 alias use`)")]
    NoDefaultAlias,

    #[error("bucket '{0}' not found")]
    BucketNotFound(String),

    #[error("bucket '{0}' already exists")]
    BucketAlreadyExists(String),

    #[error("invalid bucket name '{0}'")]
    InvalidBucketName(String),

    #[error("bucket '{0}' is not empty (use --recursive to delete its objects first)")]
    BucketNotEmpty(String),

    #[error("object '{key}' not found in bucket '{bucket}'")]
    ObjectNotFound { bucket: String, key: String },

    /// Overwrite protection on the server side (`upload-file`, `copy-file`, `move-file`).
    #[error("object '{key}' already exists in bucket '{bucket}' (use --overwrite to replace it)")]
    ObjectAlreadyExists { bucket: String, key: String },

    #[error("local file '{0}' not found")]
    LocalFileNotFound(String),

    /// Overwrite protection on the local side (`download-file`).
    #[error("local file '{0}' already exists (use --overwrite to replace it)")]
    LocalFileAlreadyExists(String),

    /// Temporary: returned by the empty handlers until each command is implemented.
    #[error("command '{0}' is not implemented yet")]
    NotImplemented(&'static str),
}
