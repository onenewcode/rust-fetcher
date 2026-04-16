# rust-fetcher

Independent Rust version of the Douyin live streaming crawler tool.

## Project Goals

- Exist as a standalone Rust project that can be migrated, maintained, and initialized with Git independently.
- Do not depend on Python source code, JS resources, or `.proto` files from the parent repository.
- Automatically generate Rust protobuf code based on `proto/douyin.proto` during build.
- Execute `assets/js/sign.js` via embedded QuickJS at runtime.
- Prioritize establishing HTTP and WebSocket login session chains using specified user Cookies.

## Directory Structure

- `src/app.rs`: Startup orchestration and runtime validation.
- `src/logging.rs`: tracing initialization and log persistence.
- `src/fetcher/`: Fetching orchestration, message parsing, and network sessions.
- `proto/douyin.proto`: Protocol source; Rust types are automatically generated during build.
- `assets/js/sign.js`: WebSocket signature script.
- `config.yaml.example`: Configuration template.
- `build.rs`: Entry point for automatic protobuf generation.
- `docs/architecture.md`: Architecture and maintenance instructions.

## How to Run

1. Enter the project directory: `cd rust-fetcher`
2. Copy `config.yaml.example` to `config.yaml`.
3. Execute: `cargo run --quiet`

The program will default to reading `config.yaml` from the current project root, loading signature logic from `assets/js/sign.js`, and appending logs to `logs/rust-fetcher.log`.

## Login Session Configuration

- When `auth.enabled=true`, the complete `auth.cookie_string` exported from the browser must be provided.
- This Cookie string is used for:
  - Fetching the live page and parsing `room_id`.
  - Obtaining supplementary runtime Cookies, such as `ttwid`.
  - Initiating the WebSocket handshake.
- If `client.user_unique_id` is empty, the program will prioritize using the `uid_tt` from the Cookie.
- To align with the actual WebSocket node hit by the browser, the default address can be overridden via `client.ws_host`.

## Logging Configuration

- By default, output is sent to both the console and the local file `logs/rust-fetcher.log`.
- The log file location can be modified via `logging.directory` and `logging.file_name`.
- The default log level can be specified via `logging.level`.
- If the `RUST_LOG` environment variable is set, its priority is higher than `logging.level`.

## Automatic protobuf Generation

- The protocol file is fixed as `proto/douyin.proto`.
- During build, `build.rs` automatically calls `prost-build`.
- The project uses `protoc-bin-vendored` to provide `protoc`, so pre-installation on the system is not required.
- Do not manually modify the generated Rust protobuf code; to modify the protocol, edit the `.proto` file directly and rebuild.

## Dependency Source Description

- `proto/douyin.proto` is copied from the protocol file in the original Python implementation of the parent repository.
- `assets/js/sign.js` is copied from the signature script in the original Python implementation of the parent repository.
- The current Rust version depends only on these copies within this project and no longer reads files from the parent repository.

## Git Boundary

`rust-fetcher/` is designed as a nested repository that can be independently `git init`.  
If you want to migrate it as a whole, simply copy the entire `rust-fetcher/` directory.
