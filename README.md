# rust-fetcher

Independent Rust workspace for Douyin/TikTok data fetching and live stream processing.

[English](./README.md) | [简体中文](./doc/readme/README_zh.md)

## Overview

`rust-fetcher` is a modular Rust workspace providing both CLI and Desktop interfaces for interacting with Douyin/TikTok services, including live stream message fetching (WebSocket) and instant messaging (IM) processing.

## Project Goals

- **Modular Architecture**: Distributed as a workspace with specialized crates for core logic, UI, and protocol handling.
- **Cross-Platform**: Supports Windows and macOS with native desktop integrations.
- **Embedded Runtimes**: Executes JavaScript signature logic via embedded QuickJS.
- **Type Safety**: Automatically generates Rust types from Protobuf definitions.
- **Standalone**: Zero dependency on external Python or JS environments.

## Workspace Structure

- `crates/cli`: Command-line interface for the fetcher.
- `crates/desktop`: Dioxus-based desktop application with theme support.
- `crates/live`: Core logic for live stream connection, WebSocket handling, and signature verification.
- `crates/im`: Protobuf-based instant messaging client and request builders.
- `crates/service`: Unified service layer and runtime orchestration.
- `crates/common`: Shared utilities, logging, configuration, and HTTP/JS runtimes.
- `xtask`: Custom automation scripts for development workflows (hooks, formatting, linting).

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (Stable)
- Protobuf Compiler (Optional, handled by `protoc-bin-vendored`)

### Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd rust-fetcher
   ```

2. Configuration:
   Copy the example configuration to the project root:
   ```bash
   cp config.yaml.example config.yaml
   ```

### Running the Application

- **Desktop UI**:
  ```bash
  cargo run -p desktop
  ```
- **CLI**:
  ```bash
  cargo run -p cli
  ```

## Development Workflow

This project uses `xtask` to manage common development tasks.

- **Run Hooks (Fmt + Clippy + Check)**:
  ```bash
  cargo xtask hook
  ```
- **Run Tests**:
  ```bash
  cargo test --workspace
  ```

## Configuration

The application reads `config.yaml` for:
- **Authentication**: `live.cookie` is required for session-based fetching.
- **Logging**: Configure levels and persistence directories.
- **Live Settings**: Customize `live.id` and WebSocket parameters.

## Architecture Highlights

- **Protobuf Generation**: `build.rs` in `live` and `im` crates automatically generates Rust code from `.proto` files using `prost`.
- **JS Integration**: Uses `rquickjs` to execute `assets/js/sign.js` for WebSocket handshake signatures.
- **Desktop UI**: Built with [Dioxus](https://dioxuslabs.com/), featuring a modern interface with light/dark theme support and system menu integration.

## License

GNU Affero General Public License v3 - see [LICENSE](LICENSE) for details.
