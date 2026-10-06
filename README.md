# mys3

A high-performance Rust command-line interface (CLI) for managing S3-compatible object storage systems (such as MinIO and AWS S3).

`mys3` provides complete commands to manage buckets, upload, download, and synchronize objects, and configure multi-endpoint aliases with AWS Signature Version 4 (SigV4) authentication.

---

## Features

- **Bucket Operations**: Create, list, inspect, and remove S3 buckets.
- **Object Operations**: Upload, download, list, inspect, copy, move, and delete objects.
- **Synchronization**: Sync local directories with remote S3 buckets.
- **Multi-Server Aliases**: Manage connection profiles (`alias set`, `alias use`, `alias list`, `alias remove`) for seamless switching between environments.
- **S3 / MinIO Compatibility**: Compliant with standard S3 REST APIs using AWS SigV4 request signing.

---

## Prerequisites

Before getting started, make sure you have installed:

- **[Rust](https://www.rust-lang.org/tools/install)** (2024 edition / stable toolchain)
- **[Docker](https://docs.docker.com/get-docker/) & [Docker Compose](https://docs.docker.com/compose/)** (for running the local MinIO instance)

---

## Quick Start

### 1. Clone & Setup Environment

```bash
git clone https://github.com/herve-beziat/plateforme-mycli.git
cd plateforme-mycli

# Copy example environment configuration
cp .env.example .env
```

### 2. Start the MinIO Storage Backend

Launch the local MinIO service in the background:

```bash
docker compose up -d
```

- **S3 API endpoint**: `http://localhost:9000`
- **MinIO Web Console**: `http://localhost:9001` (Credentials defined in `.env`: `admin` / `admin12345`)

To check MinIO container logs or stop the service:

```bash
# View logs
docker compose logs -f minio

# Stop container
docker compose down
```

### 3. Build `mys3`

Build the CLI binary with Cargo:

```bash
cargo build --release
```

The compiled binary is located at `target/release/mys3`. You can also execute it via `cargo run -- <command>` during development.

---

## Alias Workflow

`mys3` uses an alias management system to store endpoint URLs and credentials. This allows you to work across multiple S3 environments (local MinIO, staging, AWS production) without repeatedly specifying credentials or secrets in commands.

### How the Alias Workflow Works:
1. **Define an Alias (`alias set`)**: Saves an endpoint URL, access key, and secret key under a friendly name (e.g., `local`, `prod`). The first alias created is automatically set as the active default.
2. **List Configured Aliases (`alias list`)**: Displays all saved aliases with their endpoints and highlights the currently active alias.
3. **Switch Active Alias (`alias use`)**: Changes which alias is used by default for all subsequent S3 commands.
4. **Remove an Alias (`alias remove`)**: Deletes an alias from the configuration file.

---

## Command Reference

Every command supports `--help` (or `-h`) to view usage details and available flags.

### 1. Alias Commands

#### `alias set`
Saves or updates connection credentials for a given S3 endpoint.

- **Syntax**: `mys3 alias set <ALIAS_NAME> --endpoint <URL> --access-key <KEY> --secret-key <SECRET>`
- **Options**:
  - `<ALIAS_NAME>`: Unique name for this configuration profile.
  - `--endpoint`, `-e`: Base URL of the S3 service (e.g., `http://localhost:9000`).
  - `--access-key`, `-a`: S3 access key / username.
  - `--secret-key`, `-s`: S3 secret key / password.
- **Example**:
  ```bash
  mys3 alias set local \
    --endpoint http://localhost:9000 \
    --access-key admin \
    --secret-key admin12345
  ```

#### `alias list`
Displays all configured aliases, their endpoints, and indicates which alias is currently active.

- **Syntax**: `mys3 alias list`
- **Example**:
  ```bash
  mys3 alias list
  ```

#### `alias use`
Sets the active alias used by default for all commands.

- **Syntax**: `mys3 alias use <ALIAS_NAME>`
- **Example**:
  ```bash
  mys3 alias use local
  ```

#### `alias remove`
Deletes a saved alias profile.

- **Syntax**: `mys3 alias remove <ALIAS_NAME>`
- **Example**:
  ```bash
  mys3 alias remove local
  ```

---

### 2. Bucket Commands

#### `list-buckets`
Lists all buckets available on the current S3 server.

- **Syntax**: `mys3 list-buckets [OPTIONS]`
- **Options**:
  - `--alias`: *(Optional)* Run against a specific alias instead of the default.
- **Example**:
  ```bash
  mys3 list-buckets
  ```

#### `create-bucket`
Creates a new S3 bucket.

- **Syntax**: `mys3 create-bucket <BUCKET_NAME> [OPTIONS]`
- **Options**:
  - `<BUCKET_NAME>`: Name of the bucket to create (must adhere to S3 naming conventions).
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 create-bucket my-bucket
  ```

#### `bucket-info`
Displays metadata and statistics for a bucket (creation date, object count, total size).

- **Syntax**: `mys3 bucket-info <BUCKET_NAME> [OPTIONS]`
- **Options**:
  - `<BUCKET_NAME>`: Target bucket name.
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 bucket-info my-bucket
  ```

#### `delete-bucket`
Deletes an existing bucket. Note: The bucket must be empty before deletion.

- **Syntax**: `mys3 delete-bucket <BUCKET_NAME> [OPTIONS]`
- **Options**:
  - `<BUCKET_NAME>`: Target bucket to remove.
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 delete-bucket my-bucket
  ```

---

### 3. Object Commands

#### `upload-file`
Uploads a local file to a bucket.

- **Syntax**: `mys3 upload-file <BUCKET_NAME> <LOCAL_PATH> [OPTIONS]`
- **Options**:
  - `<BUCKET_NAME>`: Destination bucket name.
  - `<LOCAL_PATH>`: Path to local file on disk.
  - `--key`, `-k`: *(Optional)* Custom object key in S3 (defaults to local file name).
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 upload-file my-bucket ./report.pdf
  mys3 upload-file my-bucket ./report.pdf --key archive/2026-report.pdf
  ```

#### `list-objects`
Lists objects stored inside a bucket.

- **Syntax**: `mys3 list-objects <BUCKET_NAME> [OPTIONS]`
- **Options**:
  - `<BUCKET_NAME>`: Bucket to query.
  - `--prefix`, `-p`: *(Optional)* Filter objects starting with a specific prefix/folder path.
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 list-objects my-bucket
  mys3 list-objects my-bucket --prefix archive/
  ```

#### `object-info`
Retrieves detailed metadata for a specific object (size, ETag, last modified date, Content-Type).

- **Syntax**: `mys3 object-info <BUCKET_NAME> <OBJECT_KEY> [OPTIONS]`
- **Options**:
  - `<BUCKET_NAME>`: Target bucket.
  - `<OBJECT_KEY>`: Key/path of the object.
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 object-info my-bucket report.pdf
  ```

#### `download-file`
Downloads an object from a bucket to the local filesystem.

- **Syntax**: `mys3 download-file <BUCKET_NAME> <OBJECT_KEY> <LOCAL_PATH> [OPTIONS]`
- **Options**:
  - `<BUCKET_NAME>`: Target bucket.
  - `<OBJECT_KEY>`: Object key to download.
  - `<LOCAL_PATH>`: Destination path on local filesystem.
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 download-file my-bucket report.pdf ./downloads/report.pdf
  ```

#### `copy-file`
Copies an object from one location to another within or across buckets.

- **Syntax**: `mys3 copy-file <SRC_BUCKET/SRC_KEY> <DEST_BUCKET/DEST_KEY> [OPTIONS]`
- **Options**:
  - `<SRC_BUCKET/SRC_KEY>`: Source bucket and object key.
  - `<DEST_BUCKET/DEST_KEY>`: Target bucket and object key.
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 copy-file my-bucket/report.pdf my-bucket/backups/report.pdf
  ```

#### `move-file`
Moves or renames an object in S3 (copies then removes the source object).

- **Syntax**: `mys3 move-file <SRC_BUCKET/SRC_KEY> <DEST_BUCKET/DEST_KEY> [OPTIONS]`
- **Options**:
  - `<SRC_BUCKET/SRC_KEY>`: Source object.
  - `<DEST_BUCKET/DEST_KEY>`: Destination object.
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 move-file my-bucket/report.pdf archive-bucket/old-report.pdf
  ```

#### `delete-file`
Deletes a single object from a bucket.

- **Syntax**: `mys3 delete-file <BUCKET_NAME> <OBJECT_KEY> [OPTIONS]`
- **Options**:
  - `<BUCKET_NAME>`: Target bucket.
  - `<OBJECT_KEY>`: Object key to delete.
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 delete-file my-bucket old-report.pdf
  ```

---

### 4. Synchronization Command

#### `sync`
Synchronizes a local directory with a remote bucket (or a remote bucket prefix). Uploads missing or updated files.

- **Syntax**: `mys3 sync <LOCAL_DIRECTORY> <BUCKET_NAME>[/<PREFIX>] [OPTIONS]`
- **Options**:
  - `<LOCAL_DIRECTORY>`: Path to local folder.
  - `<BUCKET_NAME>[/<PREFIX>]`: Target bucket and optional directory prefix.
  - `--delete`: *(Optional)* Delete remote files that do not exist in the local directory.
  - `--alias`: *(Optional)* Run against a specific alias.
- **Example**:
  ```bash
  mys3 sync ./data my-bucket/backup-data
  mys3 sync ./data my-bucket/backup-data --delete
  ```

---

## Development & Testing

```bash
# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test
```

For contribution rules, branch conventions, and PR workflow, see [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

This project is licensed under the terms of the MIT License.