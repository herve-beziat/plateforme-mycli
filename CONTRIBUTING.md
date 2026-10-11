# Contributing to mys3

Thank you for contributing to **mys3**! This document outlines the standards, conventions, and workflow to keep our codebase clean, consistent, and collaborative.

---

## 1. Git Workflow & Branch Strategy

We follow a Git flow model where all development happens via Pull Requests into `develop`:

- **`main`**: Production-ready, stable releases. Protected branch. Direct pushes are forbidden.
- **`develop`**: Integration branch for ongoing work. Protected branch. Direct pushes are forbidden.
- **Feature / Chore / Bugfix branches**: Branched off `develop` and merged back into `develop` via Pull Requests.

### Branch Naming Conventions

Use lowercase branch names with hyphens, prefixed by the type of change:

| Prefix | Description | Example |
| :--- | :--- | :--- |
| `feat/` | New user-facing feature or subcommand | `feat/bucket-list`, `feat/upload-file` |
| `fix/` | Bug fixes | `fix/sigv4-empty-payload`, `fix/connection-timeout` |
| `chore/` | Maintenance, dependencies, scaffolding | `chore/update-dependencies`, `chore/setup-minio` |
| `docs/` | Documentation additions or changes | `docs/add-contributing-guidelines`, `docs/readme-usage` |
| `test/` | Adding or updating tests | `test/backup-restore-harness` |
| `refactor/` | Code refactoring without behavioral changes | `refactor/s3-client-helpers` |

Example:
```bash
git checkout develop
git pull origin develop
git checkout -b feat/list-buckets
```

---

## 2. Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. Each commit message should be clear, concise, and structured as follows:

```
<type>(<scope>): <short imperative description>
```

- **Type**: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `style`.
- **Scope** *(optional)*: `bucket`, `object`, `alias`, `sigv4`, `config`, `cli`, etc.
- **Description**: Use lowercase, present tense, imperative tone (e.g. "add support for...", not "added" or "adds"). No trailing period.

### Examples:
- `feat(bucket): implement list-buckets command`
- `fix(sigv4): correct canonical request header sorting`
- `docs: update command examples in README`
- `chore: move minio credentials to .env file`

---

## 3. Pull Request (PR) Process

1. **Keep branches updated**: Always rebase or merge the latest `develop` into your branch before creating a PR.
2. **Target `develop`**: All pull requests must target the `develop` branch (`--base develop`).
3. **Link issues**: Always link the relevant GitHub issue in the PR description using keywords like `Closes #<issue_number>` or `Fixes #<issue_number>`.
4. **Code Quality Checks**: Before submitting your PR, ensure the code builds and passes linting and formatting:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```
5. **Code Review**: At least one review is required before merging. Do not merge your own PR without team approval when reviews are requested.

### PR Description Template

```markdown
Closes #<issue_number>

## Summary
Brief description of the changes made and the motivation.

## Changes
- List key changes
- Highlight any breaking changes or new dependencies

## Verification
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] Tested manually against MinIO
```

---

## 4. Development Setup

### Prerequisites
- **Rust**: Latest stable toolchain (2024 edition).
- **Docker & Docker Compose**: For local MinIO S3 backend.
- **GitHub CLI (`gh`)**: Recommended for managing issues and PRs.

### Local Environment
1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```
2. Start the local MinIO instance:
   ```bash
   docker compose up -d
   ```
   MinIO will be accessible at:
   - S3 API: `http://localhost:9000`
   - Web Console: `http://localhost:9001` (Credentials from `.env`)

3. Build and test the CLI:
   ```bash
   cargo build
   cargo run -- --help
   ```

---

## 5. Architecture & Code Style

- **CLI Framework**: Built using `clap` (derive feature). Every command resides in `src/commands/<command_name>.rs`.
- **Error Handling**: Use the shared `MyS3Error` enum (powered by `thiserror`). Print user-friendly errors to `stderr` and exit with status code `1`.
- **Formatting**: Format code before committing using `cargo fmt`.
- **Security**: Never hardcode credentials, access keys, or secrets in the repository. Use `.env` and environment variables.
