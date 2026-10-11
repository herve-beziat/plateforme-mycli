//! Command-line interface definition: the root `mys3` command and the list of
//! all its commands. The arguments of each command live in `commands/`.

use clap::{Parser, Subcommand, ValueEnum};

use crate::commands;

/// Root command.
#[derive(Debug, Parser)]
#[command(
    name = "mys3",
    version,
    about = "A command-line client for S3-compatible servers"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// All the commands of `mys3`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// List all buckets on the server
    ListBuckets(commands::list_buckets::Args),

    /// Create a bucket
    CreateBucket(commands::create_bucket::Args),

    /// Delete a bucket
    DeleteBucket(commands::delete_bucket::Args),

    /// Show bucket details
    BucketInfo(commands::bucket_info::Args),

    /// Upload a local file to a bucket
    UploadFile(commands::upload_file::Args),

    /// List the objects of a bucket
    ListObjects(commands::list_objects::Args),

    /// Download an object to the local disk
    DownloadFile(commands::download_file::Args),

    /// Delete an object
    DeleteFile(commands::delete_file::Args),

    /// Show object details
    ObjectInfo(commands::object_info::Args),

    /// Copy an object between buckets of the same server
    CopyFile(commands::copy_file::Args),

    /// Move an object between buckets of the same server
    MoveFile(commands::move_file::Args),

    /// Synchronise a local folder to a bucket
    Sync(commands::sync::Args),

    /// Manage server aliases
    #[command(subcommand)]
    Alias(AliasCommand),
}

/// Subcommands of `mys3 alias`.
#[derive(Debug, Subcommand)]
pub enum AliasCommand {
    /// Create or update an alias
    Set(commands::alias_set::Args),

    /// Switch the default alias
    Use(commands::alias_use::Args),

    /// List the aliases
    List(commands::alias_list::Args),

    /// Delete an alias
    Remove(commands::alias_remove::Args),
}

/// Output format of the commands that accept `--output`.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text
    Text,
    /// Machine-readable JSON
    #[value(name = "JSON")]
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// Checks that the whole command tree is valid (no duplicated or conflicting arguments).
    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
