//! Entry point of `mys3`: parses the command line, runs the matching command
//! and turns any error into a message on stderr with exit code 1.

mod cli;
#[allow(dead_code)] // used by the commands (#19 onwards)
mod client;
mod commands;
#[allow(dead_code)] // used by the commands (#15 onwards)
mod config;
mod error;
mod signer;

use std::process::ExitCode;

use clap::Parser;

use cli::{AliasCommand, Cli, Command};
use error::MyS3Error;

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // Help and version go to stdout (success); usage errors go to stderr.
            let _ = err.print();
            return if err.use_stderr() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            };
        }
    };

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::from(1)
        }
    }
}

/// Calls the handler of the command given on the command line.
fn run(cli: Cli) -> Result<(), MyS3Error> {
    match cli.command {
        Command::ListBuckets(args) => commands::list_buckets::run(args),
        Command::CreateBucket(args) => commands::create_bucket::run(args),
        Command::DeleteBucket(args) => commands::delete_bucket::run(args),
        Command::BucketInfo(args) => commands::bucket_info::run(args),
        Command::UploadFile(args) => commands::upload_file::run(args),
        Command::ListObjects(args) => commands::list_objects::run(args),
        Command::DownloadFile(args) => commands::download_file::run(args),
        Command::DeleteFile(args) => commands::delete_file::run(args),
        Command::ObjectInfo(args) => commands::object_info::run(args),
        Command::CopyFile(args) => commands::copy_file::run(args),
        Command::MoveFile(args) => commands::move_file::run(args),
        Command::Sync(args) => commands::sync::run(args),
        Command::Alias(alias) => match alias {
            AliasCommand::Set(args) => commands::alias_set::run(args),
            AliasCommand::Use(args) => commands::alias_use::run(args),
            AliasCommand::List(args) => commands::alias_list::run(args),
            AliasCommand::Remove(args) => commands::alias_remove::run(args),
        },
    }
}
