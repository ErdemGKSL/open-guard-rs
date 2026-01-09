# Discord Guard Bot (Rust)

A high-performance, scalable Discord guard bot implemented in Rust using the **serenity** and **poise** frameworks. Designed for sharding, reliability, and modularity.

## 🚀 Features

- **Sharding**: Ready for large-scale operation with autosharding support.
- **Dynamic Prefixing**: Database-backed prefix handling with high-performance **lock-free caching** (using [Papaya](https://github.com/ibraheemdev/papaya)).
- **Modular Architecture**: Clean separation of concerns between core services, database logic, and feature modules.
- **Database Integrated**: Powered by **PostgreSQL** and **SeaORM** with automated migrations.
- **Slash Commands**: Full support for slash commands, including subcommands and autocomplete.
- **CLI Command Publisher**: Built-in CLI tool to publish slash commands globally or to specific guilds.

## 🛠️ Tech Stack

- **Language**: Rust (Stable)
- **Discord Framework**: [serenity](https://github.com/serenity-rs/serenity) + [poise](https://github.com/serenity-rs/poise)
- **Database/ORM**: [SeaORM](https://www.sea-ql.org/SeaORM/) + PostgreSQL
- **Async Runtime**: [tokio](https://tokio.rs/)
- **Caching**: [Papaya](https://github.com/ibraheemdev/papaya) (Concurrent Hash Table)
- **Configuration**: [dotenvy](https://github.com/allan2/dotenvy)
- **Error Handling**: [anyhow](https://github.com/dtolnay/anyhow)
- **CLI Parsing**: [clap](https://github.com/clap-rs/clap)

## 📁 Project Structure

```text
src/
├── db/               # Database entities and migrations
│   ├── entities/     # SeaORM entity models
│   └── migrations/   # Database schema migrations
├── modules/          # Feature-specific modules (commands & events)
│   └── hello/        # Example feature module
├── services/         # Shared business logic and managers
│   ├── event_manager.rs # Main event handling logic
│   └── prefix.rs     # Dynamic prefix and caching logic
└── main.rs           # Entry point and bot initialization
```

## ⚙️ Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (Stable)
- [PostgreSQL](https://www.postgresql.org/)

### Setup

1. **Clone the repository**:
   ```bash
   git clone <repository_url>
   cd open-guard-rs
   ```

2. **Configure environment variables**:
   ```bash
   cp .env.example .env
   ```
   Edit `.env` and provide your `DISCORD_TOKEN` and `DATABASE_URL`.

3. **Database Migrations**:
   Migrations run automatically when the bot starts.

### Running the Bot

```bash
# Direct run
cargo run

# With command publishing (Global)
cargo run -- --publish

# With command publishing (Specific Guild)
cargo run -- --publish <guild_id>
```

## 📜 Usage

### CLI Commands

| Flag | Description |
|------|-------------|
| `--publish` | Publishes all registered slash commands globally to Discord. |
| `--publish <guild_id>` | Publishes slash commands to a specific guild (instant update). |

## 🛡️ License

Distributed under the MIT License. See `LICENSE` for more information.
