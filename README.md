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

The image comes from `ghcr.io/herve-beziat/minio`. MinIO no longer publishes its Docker images (Docker Hub and quay.io refuse the download), so the project keeps an unchanged copy of the release it uses, `RELEASE.2025-09-07T16-13-09Z`.

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

Rules shared by the commands:

- Commands that talk to a server accept `--alias <ALIAS>`. Without it, the default alias is used.
- `--output <text|JSON>`, where available, selects human-readable text (default) or JSON.
- Destructive commands ask for a confirmation. `--force` (or `-f`) skips it.
- Errors are printed on `stderr` and the exit code is `1`.

### 1. Alias Commands

#### `alias set`
Creates or updates an alias: a server URL and its keys saved under a name. The first alias created becomes the default one.

- **Syntax**: `mys3 alias set <ALIAS_NAME> <URL> <ACCESS_KEY> [SECRET_KEY] [OPTIONS]`
- **Arguments and options**:
  - `<ALIAS_NAME>`: Name of the alias.
  - `<URL>`: URL of the server (e.g., `http://localhost:9000`).
  - `<ACCESS_KEY>`: S3 access key / username.
  - `[SECRET_KEY]`: S3 secret key / password. When omitted, it is asked with hidden input.
  - `--default`: *(Optional)* Make this alias the default one.
  - `--region <REGION>`: *(Optional)* Region of the server (default: `us-east-1`).
- **Example**:
  ```bash
  mys3 alias set local http://localhost:9000 admin admin12345
  mys3 alias set prod https://s3.example.com my-access-key --default --region eu-west-1
  ```

#### `alias list`
Displays all configured aliases with their URL and region, and indicates which one is the default. Keys are never displayed.

- **Syntax**: `mys3 alias list [OPTIONS]`
- **Arguments and options**:
  - `--output <text|JSON>`: *(Optional)* Output format.
- **Example**:
  ```bash
  mys3 alias list
  ```

#### `alias use`
Sets the alias used by default for all commands.

- **Syntax**: `mys3 alias use <ALIAS_NAME>`
- **Arguments and options**:
  - `<ALIAS_NAME>`: Name of an existing alias.
- **Example**:
  ```bash
  mys3 alias use local
  ```

#### `alias remove`
Deletes a saved alias. If it was the default one, there is no default alias until `alias use` is run.

- **Syntax**: `mys3 alias remove <ALIAS_NAME> [OPTIONS]`
- **Arguments and options**:
  - `<ALIAS_NAME>`: Name of the alias to delete.
  - `--force`, `-f`: *(Optional)* Skip the confirmation.
- **Example**:
  ```bash
  mys3 alias remove local
  ```

---

### 2. Bucket Commands

#### `list-buckets`
Lists all buckets available on the server. A server without buckets is not an error.

- **Syntax**: `mys3 list-buckets [OPTIONS]`
- **Arguments and options**:
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias instead of the default.
  - `--output <text|JSON>`: *(Optional)* Output format.
- **Example**:
  ```bash
  mys3 list-buckets
  mys3 list-buckets --alias prod --output JSON
  ```

#### `create-bucket`
Creates a new bucket.

- **Syntax**: `mys3 create-bucket <BUCKET_NAME> [OPTIONS]`
- **Arguments and options**:
  - `<BUCKET_NAME>`: Name of the bucket to create (must adhere to S3 naming conventions: 3-63 characters, lowercase letters, digits, dots and hyphens).
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--region <REGION>`: *(Optional)* Region of the bucket (default: the region of the alias).
- **Example**:
  ```bash
  mys3 create-bucket my-bucket
  mys3 create-bucket my-eu-bucket --region eu-west-1
  ```

#### `bucket-info`
Displays the details of a bucket: name, creation date, object count and total size.

- **Syntax**: `mys3 bucket-info <BUCKET_NAME> [OPTIONS]`
- **Arguments and options**:
  - `<BUCKET_NAME>`: Target bucket name.
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--output <text|JSON>`: *(Optional)* Output format.
- **Example**:
  ```bash
  mys3 bucket-info my-bucket
  ```

#### `delete-bucket`
Deletes a bucket, after a confirmation. A bucket that still contains objects is refused unless `--recursive` is given.

- **Syntax**: `mys3 delete-bucket <BUCKET_NAME> [OPTIONS]`
- **Arguments and options**:
  - `<BUCKET_NAME>`: Bucket to delete.
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--force`, `-f`: *(Optional)* Skip the confirmation.
  - `--recursive`: *(Optional)* Delete the objects of the bucket first.
- **Example**:
  ```bash
  mys3 delete-bucket my-bucket
  mys3 delete-bucket my-bucket --recursive --force
  ```

---

### 3. Object Commands

#### `upload-file`
Uploads a local file to a bucket. An object that already exists is not replaced unless `--overwrite` is given.

- **Syntax**: `mys3 upload-file <FILE_PATH> <BUCKET_NAME> [OPTIONS]`
- **Arguments and options**:
  - `<FILE_PATH>`: Path of the local file to upload.
  - `<BUCKET_NAME>`: Destination bucket name.
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--key <KEY>`: *(Optional)* Object key in the bucket (defaults to the file name).
  - `--overwrite`: *(Optional)* Replace the object if it already exists.
- **Example**:
  ```bash
  mys3 upload-file ./report.pdf my-bucket
  mys3 upload-file ./report.pdf my-bucket --key archive/2026-report.pdf
  ```

#### `list-objects`
Lists the objects stored in a bucket.

- **Syntax**: `mys3 list-objects <BUCKET_NAME> [OPTIONS]`
- **Arguments and options**:
  - `<BUCKET_NAME>`: Bucket to query.
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--prefix <PREFIX>`: *(Optional)* Only list the keys starting with this prefix.
  - `--output <text|JSON>`: *(Optional)* Output format.
- **Example**:
  ```bash
  mys3 list-objects my-bucket
  mys3 list-objects my-bucket --prefix archive/
  ```

#### `object-info`
Displays the details of an object: name, size, last modified date, content type and ETag.

- **Syntax**: `mys3 object-info <BUCKET_NAME> <OBJECT_KEY> [OPTIONS]`
- **Arguments and options**:
  - `<BUCKET_NAME>`: Target bucket.
  - `<OBJECT_KEY>`: Key of the object.
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--output <text|JSON>`: *(Optional)* Output format.
- **Example**:
  ```bash
  mys3 object-info my-bucket report.pdf
  ```
- **Output**:
  ```
  Name:          report.pdf
  Size:          1024 bytes
  Last modified: 2026-10-08T08:21:06Z
  Content type:  application/pdf
  ETag:          5d41402abc4b2a76b9719d911017c592
  ```
  With `--output JSON`, the keys are `name`, `size`, `last_modified`, `content_type` and `etag`.
- **Errors**: the command fails (exit code `1`) if the bucket or the object does not exist.
- **Note**: the ETag is the MD5 of the content only for single-part, unencrypted uploads.

#### `download-file`
Downloads an object to the local disk. A local file that already exists is not replaced unless `--overwrite` is given.

- **Syntax**: `mys3 download-file <BUCKET_NAME> <OBJECT_KEY> [OPTIONS]`
- **Arguments and options**:
  - `<BUCKET_NAME>`: Target bucket.
  - `<OBJECT_KEY>`: Key of the object to download.
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--output <PATH>`: *(Optional)* Local destination path (defaults to the object name in the current directory).
  - `--overwrite`: *(Optional)* Replace the local file if it already exists.
- **Example**:
  ```bash
  mys3 download-file my-bucket report.pdf
  mys3 download-file my-bucket report.pdf --output ./downloads/report.pdf
  ```

#### `copy-file`
Copies an object to another bucket, or to another key of the same bucket, on the same server.

- **Syntax**: `mys3 copy-file <SOURCE_BUCKET> <OBJECT_KEY> <DESTINATION_BUCKET> [OPTIONS]`
- **Arguments and options**:
  - `<SOURCE_BUCKET>`: Bucket that contains the object.
  - `<OBJECT_KEY>`: Key of the object to copy.
  - `<DESTINATION_BUCKET>`: Bucket that receives the copy.
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--key <NEW_KEY>`: *(Optional)* Key in the destination bucket (defaults to the same key).
  - `--overwrite`: *(Optional)* Replace the object if it already exists at the destination.
- **Example**:
  ```bash
  mys3 copy-file my-bucket report.pdf backup-bucket
  mys3 copy-file my-bucket report.pdf my-bucket --key backups/report.pdf
  ```

#### `move-file`
Moves or renames an object: it is copied, then the source object is deleted. There is no confirmation.

- **Syntax**: `mys3 move-file <SOURCE_BUCKET> <OBJECT_KEY> <DESTINATION_BUCKET> [OPTIONS]`
- **Arguments and options**:
  - `<SOURCE_BUCKET>`: Bucket that contains the object.
  - `<OBJECT_KEY>`: Key of the object to move.
  - `<DESTINATION_BUCKET>`: Bucket that receives the object.
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--key <NEW_KEY>`: *(Optional)* Key in the destination bucket (defaults to the same key).
  - `--overwrite`: *(Optional)* Replace the object if it already exists at the destination.
- **Example**:
  ```bash
  mys3 move-file my-bucket report.pdf archive-bucket
  mys3 move-file my-bucket report.pdf my-bucket --key old-report.pdf
  ```

#### `delete-file`
Deletes a single object from a bucket, after a confirmation.

- **Syntax**: `mys3 delete-file <BUCKET_NAME> <OBJECT_KEY> [OPTIONS]`
- **Arguments and options**:
  - `<BUCKET_NAME>`: Target bucket.
  - `<OBJECT_KEY>`: Key of the object to delete.
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--force`, `-f`: *(Optional)* Skip the confirmation.
- **Example**:
  ```bash
  mys3 delete-file my-bucket old-report.pdf
  ```

---

### 4. Synchronization Command

#### `sync`
Synchronizes a local folder to a bucket, one way: the contents of the folder, subfolders included, are uploaded when they are missing or different in the bucket.

- **Syntax**: `mys3 sync <LOCAL_FOLDER> <BUCKET_NAME> [OPTIONS]`
- **Arguments and options**:
  - `<LOCAL_FOLDER>`: Path of the local folder.
  - `<BUCKET_NAME>`: Destination bucket.
  - `--alias <ALIAS>`: *(Optional)* Run against a specific alias.
  - `--prefix <PREFIX>`: *(Optional)* Destination prefix in the bucket.
  - `--dry-run`: *(Optional)* Show what would be done without doing it.
  - `--delete`: *(Optional)* Also remove from the bucket the objects that no longer exist locally, after a confirmation.
  - `--force`, `-f`: *(Optional)* Skip that confirmation.
- **Example**:
  ```bash
  mys3 sync ./data my-bucket
  mys3 sync ./data my-bucket --prefix backup-data
  mys3 sync ./data my-bucket --prefix backup-data --delete --dry-run
  ```

---

## Development & Testing

```bash
# Check formatting
cargo fmt --check

# Run linter
cargo clippy --all-targets -- -D warnings

# Run tests
cargo test
```

### Functional tests against MinIO

Some tests talk to a real server. They are marked `#[ignore]`, so `cargo test` skips them. To run them, start MinIO:

```bash
docker compose up -d

# Only the tests that need MinIO
cargo test -- --ignored

# Every test
cargo test -- --include-ignored
```

The tests read the keys and the port of MinIO from `.env` (`MINIO_ROOT_USER`, `MINIO_ROOT_PASSWORD`, `MINIO_PORT`). To test against another server, set `MYS3_ACCESS_KEY`, `MYS3_SECRET_KEY` and `MYS3_TEST_URL`: they take priority over `.env`.

### Writing a functional test

A functional test runs the real `mys3` binary and checks its output and its exit code. It lives in `tests/test_<command>.rs` and uses the helpers of `tests/common`:

- `TestEnv` gives the binary a temporary home directory, so a test never reads or writes your real `~/.mys3/config.json`.
- `TempBucket` is a bucket with a unique name on the test server, deleted with its objects at the end of the test.

`tests/common/mod.rs` starts with an example, and `tests/test_harness.rs` contains working tests.

### Continuous integration

On every pull request to `develop` or `main`, and on every push to `develop`, GitHub Actions runs `.github/workflows/ci.yml`:

1. `cargo fmt --check`
2. `cargo clippy --all-targets -- -D warnings`
3. MinIO is started with `docker-compose.yml` and the keys of `.env.example`
4. `cargo test -- --include-ignored`, so every test runs, including the ones that need MinIO

The result appears in the checks of the pull request. Run the same commands locally before pushing to avoid a red CI.

For contribution rules, branch conventions, and PR workflow, see [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

This project is licensed under the terms of the MIT License.
