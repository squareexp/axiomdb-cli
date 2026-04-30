# PulsarDB CLI

> Multi-branch Postgres database management from your terminal — built in Rust.

```
        ●●●●
      ●●●●●●●●          P U L S A R  D B
     ●●●●●●●●●●         Database control plane
    ●●●●●●●●●●●●
   ●●●●●●●●●●●●●●     ▸ Multi-branch Postgres
    ●●●●●●●●●●●●     ▸ Prisma-ready connections
     ●●●●●●●●●●      ▸ Real-time monitoring
      ●●●●●●●●
        ●●●●
```

## Installation

```bash
npm install -g pulsardb
```

## Quick start

```bash
pulsardb login                      # Log in (interactive)
pulsardb projects list              # List all projects
pulsardb projects create            # Create a new project (provisions a real DB on VPS)
pulsardb projects use <id>          # Set active project context
pulsardb branches list              # List branches (uses active project)
pulsardb branches create            # Create a branch (max 10 per project)
pulsardb monitoring summary         # CPU / RAM / Postgres health
pulsardb monitoring stream          # Live metric stream (SSE)
pulsardb gen tk                     # Prisma-ready DATABASE_URL + DIRECT_URL
pulsardb secrets generate           # Generate a cryptographic secret
pulsardb audit list                 # View audit event log
```

## Commands

| Command | Alias | Description |
|---|---|---|
| `login` | | Log in (interactive or --email/--password) |
| `logout` | | Clear local session |
| `whoami` | | Show current session |
| `projects list` | `p -l` | List all projects |
| `projects create` | | Provision a new project + database |
| `projects use <id>` | `project use` | Set active project context |
| `projects get` | | Show project details |
| `projects current` | | Show active project |
| `projects unset` | | Clear active project |
| `branches list` | `br -l` | List branches |
| `branches create` | | Create a branch (max 10) |
| `branches delete` | `br rm` | Delete a branch |
| `monitoring summary` | `mon` | Health snapshot |
| `monitoring stream` | | Live SSE metric stream |
| `tables list` | `tb` | List database tables |
| `backups list` | `bk` | Show backup catalog |
| `backups restore` | | Queue a restore job |
| `secrets generate` | `sec` | Generate a secret |
| `audit list` | | View audit events |
| `jobs get <id>` | | Poll a provisioning job |
| `gen tk` | | Prisma DATABASE_URL + DIRECT_URL |

## Platform support

| Platform | Package |
|---|---|
| macOS Apple Silicon | `@pulsardb/cli-darwin-arm64` |
| macOS Intel | `@pulsardb/cli-darwin-x64` |
| Linux x64 | `@pulsardb/cli-linux-x64` |
| Linux ARM64 | `@pulsardb/cli-linux-arm64` |
| Windows x64 | `@pulsardb/cli-win32-x64` |

## Build from source

```bash
git clone https://github.com/squareexp/pulsardb
cd pulsardb
cargo build --release
./target/release/pulsardb --help
```

## License

MIT — © Square Exp
