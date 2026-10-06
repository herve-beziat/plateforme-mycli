//! One module per command. Each module declares the arguments of its command
//! (`Args`) and its handler (`run`).

pub mod alias_list;
pub mod alias_remove;
pub mod alias_set;
pub mod alias_use;
pub mod bucket_info;
pub mod copy_file;
pub mod create_bucket;
pub mod delete_bucket;
pub mod delete_file;
pub mod download_file;
pub mod list_buckets;
pub mod list_objects;
pub mod move_file;
pub mod object_info;
pub mod sync;
pub mod upload_file;
