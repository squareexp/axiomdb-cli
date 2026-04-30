# 🌌 AxiomDB CLI (`axiomdb-cli`)

*The terminal tool you didn't know you needed, but now can't live without.*

Traditional database management UIs are clunky and highkey mid. The **AxiomDB CLI** is here to give you total control over your multi-branch Postgres databases right from your terminal. It's fast, aesthetic, and fits perfectly into your developer workflow. 

## ⚡️ Why it Matters

Clicking through dashboards is an L when you're in the zone. We built this CLI so you can spin up databases, branch your schema, and pull Prisma URLs without ever leaving your terminal. It's giving main character energy for your DevOps.

| The Old Way (L) | The Axiom Way (W) |
| :--- | :--- |
| Going to a website to grab a connection string | `axm gen tk <project-id>` |
| Sharing a broken dev database | `axm branches create feat-auth` |
| Wondering if your DB is down | `axm monitoring stream <project-id>` |

## 📦 Installation

Grab the package straight from NPM. It's globally available and installs in seconds.

```bash
npm install -g axiomdb-cli
```

## 🎮 How to Use It

We kept the commands short and sweet. You can use `axiom` or just `axm` if you're lazy (valid).

### The Basics
```bash
axm login                      # Authenticate your session
axm whoami                     # Check your vibe (who's logged in)
axm projects list              # See all your database projects
axm branches list <id>         # Check your branches
```

### The Based Features
```bash
axm monitoring stream <id>     # Live telemetry streaming right in your terminal
axm gen tk <id>                # Drops Prisma-ready URLs straight to your clipboard
axm secrets generate           # Generates a fresh, secure crypto token
```

### The Vibe Check (Illustration)

```text
          ●●●●            
        ●●●●●●●●            A X I O M  D B
       ●●●●●●●●●●           Database control plane
      ●●●●●●●●●●●●        
     ●●●●●●●●●●●●●●         ▸ Multi-branch Postgres
      ●●●●●●●●●●●●          ▸ Prisma-ready connections
       ●●●●●●●●●●           ▸ Real-time monitoring
        ●●●●●●●●          
          ●●●●              v0.1.0
```

## 🐛 Something broke? 

If the CLI throws an error or acts out of pocket, we need to know.
1. Run your command again with the `--verbose` flag (if applicable) to catch the receipts.
2. Open an issue on our [GitHub Repo](https://github.com/squareexp/axiomdb-cli/issues).
3. Drop the logs and let us know your OS (macOS/Linux/Windows). We'll patch it ASAP.
