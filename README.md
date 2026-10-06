# mys3

A high-performance Rust command-line interface (CLI) for managing S3-compatible object storage systems (such as MinIO and AWS S3).

`mys3` provides intuitive commands to manage buckets, upload, download, and synchronize objects, and configure multi-endpoint aliases with AWS Signature Version 4 (SigV4) authentication.

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

The compiled binary will be located at `target/release/mys3`. Alternatively, run directly via Cargo during development:

```bash
cargo run -- --help
```

---

## Configuration & Aliases

`mys3` uses aliases to store endpoint configurations and credentials securely.

### Configure your first alias

Set up a local MinIO alias named `local`:

```bash
mys3 alias set local \
  --endpoint http://localhost:9000 \
  --access-key admin \
  --secret-key admin12345
```

### List configured aliases

```bash
mys3 alias list
```

### Switch default active alias

```bash
mys3 alias use local
```

### Remove an alias

```bash
mys3 alias remove local
```

---

## Usage Examples

### Bucket Management

```bash
# List all buckets
mys3 list-buckets

# Create a new bucket
mys3 create-bucket my-bucket

# Inspect bucket metadata & stats
mys3 bucket-info my-bucket

# Delete an empty bucket
mys3 delete-bucket my-bucket
```

### Object Management

```bash
# Upload a file
mys3 upload-file my-bucket document.pdf

# List objects in a bucket
mys3 list-objects my-bucket

# Inspect an object (metadata, size, ETag)
mys3 object-info my-bucket document.pdf

# Download a file
mys3 download-file my-bucket document.pdf ./downloads/document.pdf

# Copy an object
mys3 copy-file my-bucket/document.pdf my-bucket/document-backup.pdf

# Move or rename an object
mys3 move-file my-bucket/document-backup.pdf my-bucket/archive.pdf

# Delete an object
mys3 delete-file my-bucket archive.pdf
```

### Directory Synchronization

Synchronize a local folder with an S3 bucket:

```bash
mys3 sync ./data my-bucket/backup-data
```

---

## Development & Testing

```bash
# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Run unit and integration tests
cargo test
```

For contribution guidelines and branching workflow, see [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

This project is licensed under the terms of the MIT License.