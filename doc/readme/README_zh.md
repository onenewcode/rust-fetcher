# rust-fetcher

用于抖音/TikTok 数据抓取和直播流处理的独立 Rust 工作空间。

[English](../../README.md) | [简体中文](./README_zh.md)

## 项目简介

`rust-fetcher` 是一个模块化的 Rust 工作空间，提供命令行（CLI）和桌面（Desktop）界面，用于与抖音/TikTok 服务交互，包括直播消息抓取（WebSocket）和即时通讯（IM）处理。

## 项目目标

- **模块化架构**：作为包含多个专门用于核心逻辑、UI 和协议处理的 crate 的工作空间。
- **跨平台支持**：支持 Windows 和 macOS，具有原生桌面集成。
- **嵌入式运行时**：通过嵌入式 QuickJS 执行 JavaScript 签名逻辑。
- **类型安全**：从 Protobuf 定义自动生成 Rust 类型。
- **独立运行**：零依赖于外部 Python 或 JS 环境。

## 工作空间结构

- `crates/cli`：抓取器的命令行界面。
- `crates/desktop`：基于 Dioxus 的桌面应用程序，支持主题切换。
- `crates/live`：直播连接、WebSocket 处理和签名验证的核心逻辑。
- `crates/im`：基于 Protobuf 的即时通讯客户端和请求构建器。
- `crates/service`：统一的服务层和运行时编排。
- `crates/common`：共享工具、日志记录、配置以及 HTTP/JS 运行时。
- `xtask`：用于开发工作流（格式化、Lint、检查）的自定义自动化脚本。

## 快速入门

### 前置条件

- [Rust](https://rustup.rs/) (Stable)
- Protobuf 编译器（可选，由 `protoc-bin-vendored` 自动处理）

### 安装步骤

1. 克隆仓库：
   ```bash
   git clone <repository-url>
   cd rust-fetcher
   ```

2. 配置：
   将示例配置复制到项目根目录：
   ```bash
   cp config.yaml.example config.yaml
   ```

### 运行程序

- **桌面 UI**：
  ```bash
  cargo run -p desktop
  ```
- **命令行 (CLI)**：
  ```bash
  cargo run -p cli
  ```

## 开发工作流

本项目使用 `xtask` 来管理常见的开发任务。

- **运行挂钩（格式化 + Clippy + 检查）**：
  ```bash
  cargo xtask hook
  ```
- **运行测试**：
  ```bash
  cargo test --workspace
  ```

## 配置说明

应用程序读取 `config.yaml` 进行以下配置：
- **身份验证**：`live.cookie` 是基于会话抓取所必需的。
- **日志记录**：配置日志级别和持久化目录。
- **直播设置**：自定义 `live.id` 和 WebSocket 参数。

## 架构亮点

- **Protobuf 生成**：`live` 和 `im` crate 中的 `build.rs` 会自动使用 `prost` 从 `.proto` 文件生成 Rust 代码。
- **JS 集成**：使用 `rquickjs` 执行 `assets/js/sign.js` 以获取 WebSocket 握手签名。
- **桌面 UI**：使用 [Dioxus](https://dioxuslabs.com/) 构建，具有现代界面、亮/暗主题支持以及系统菜单集成。

## 开源协议

GNU Affero General Public License v3 - 详见 [LICENSE](../../LICENSE)。
