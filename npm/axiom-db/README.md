# AxiomDB CLI

> Multi-branch Postgres database management from your terminal — built in Rust.

```
        ●●●●
      ●●●●●●●●          A X I O M  D B
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
npm install -g axiom-db
```

## Quick start

```bash
axiom login                      # Log in (interactive)
axiom projects list              # List all projects
axiom projects create            # Create a new project (provisions a real DB on VPS)
axiom projects use <id>          # Set active project context
axiom branches list              # List branches (uses active project)
axiom branches create            # Create a branch (max 10 per project)
axiom monitoring summary         # CPU / RAM / Postgres health
axiom monitoring stream          # Live metric stream (SSE)
axiom gen tk                     # Prisma-ready DATABASE_URL + DIRECT_URL
axiom secrets generate           # Generate a cryptographic secret
axiom audit list                 # View audit event log
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
| macOS Apple Silicon | `@axiom-db/cli-darwin-arm64` |
| macOS Intel | `@axiom-db/cli-darwin-x64` |
| Linux x64 | `@axiom-db/cli-linux-x64` |
| Linux ARM64 | `@axiom-db/cli-linux-arm64` |
| Windows x64 | `@axiom-db/cli-win32-x64` |

## Build from source

```bash
git clone https://github.com/squareexp/axiom-db
cd axiom-db
cargo build --release
./target/release/axiom-db --help
```

## License

MIT — © Square Exp
