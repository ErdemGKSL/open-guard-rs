# 🛡️ Open Guard

**Open Guard** is a high-performance, open-source security and protection bot for Discord, built with Rust. It is designed to keep your community safe with lightning-fast response times and robust modular protection.

[![Release](https://img.shields.io/github/v/release/ErdemGKSL/open-guard-rs?label=latest%20release&color=blue)](https://github.com/ErdemGKSL/open-guard-rs/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## ✨ Key Features

*   **⚡ Blazing Fast**: Built in Rust for maximum performance and minimal memory footprint.
*   **🛡️ Advanced Protection**: Modular guard system to protect your server from malicious actions.
*   **📂 Easy Scale**: Full support for autosharding, ready to protect thousands of servers.
*   **💻 Modern Commands**: Native support for Slash Commands and subcommands.
*   **⚙️ Highly Configurable**: Database-backed settings that can be customized for every server.

---

## 🚀 Getting Started (Quickest Way)

You don't need to be a developer to run Open Guard! We provide pre-compiled binaries for all major platforms.

1.  **Download Open Guard**: Go to the [Latest Release](https://github.com/ErdemGKSL/open-guard-rs/releases/latest) and download the binary for your operating system (Windows, Linux, or macOS).
2.  **Setup Environment**: Create a file named `.env` in the same folder as the binary:
    ```env
    DISCORD_TOKEN=your_token_here
    DATABASE_URL=postgres://user:pass@localhost/open_guard
    ```
3.  **Run it**:
    *   **Windows**: Double-click `open-guard-rs-windows-x86_64.exe`
    *   **Linux/macOS**: 
        ```bash
        chmod +x open-guard-rs-linux-x86_64
        ./open-guard-rs-linux-x86_64
        ```

---

## 🛠️ For Developers

If you want to contribute or build from source, follow these steps:

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (Stable)
- [PostgreSQL](https://www.postgresql.org/)

### Installation
```bash
git clone https://github.com/ErdemGKSL/open-guard-rs.git
cd open-guard-rs
cargo build --release
```

### Running with Commands
```bash
# Start the bot
cargo run --release

# Publish Slash Commands (First time only)
cargo run --release -- --publish
```

---

## 📁 Project Overview

*   **`src/modules/`**: Contains all protection and utility modules.
*   **`src/db/`**: Handles database entities and automated migrations.
*   **`src/services/`**: Core engine logic and event management.

## 🛡️ License

This project is licensed under the **MIT License**. See the `LICENSE` file for details.

---
*Created with ❤️ by the Open Guard Community.*
