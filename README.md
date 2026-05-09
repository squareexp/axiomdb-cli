# AxiomDB CLI

Fast terminal control for AxiomDB projects, branches, Prisma URLs, network access, metrics, backups, audit, and secrets.

## Install

```bash
npm install -g axiomdb-cli
```

The npm package exposes `axm` for speed and keeps `axiom` for compatibility.

## Sign in

```bash
axm login
axm -li
```

The CLI opens Square IdP in your browser and uses OAuth 2.0 with PKCE. If the loopback callback cannot complete, paste the authorization code or the full redirect URL back into the terminal.

```text
◒  Complete sign-in in browser…
◇  Paste the authorization code or full redirect URL:
◇  AxiomDB OAuth complete
```

## Projects by name

```bash
axm projects list
axm projects use "Square Experience"
axm projects use square_experience-main
axm projects current
```

IDs still work, but names and app slugs are the normal path. If a name is duplicated, the CLI asks you to pick the right project.

## Branches

```bash
axm branches list
axm branches create --name feature-auth --lifespan 7d
axm branches urls --name feature-auth
axm branches delete feature-auth
```

Branch URL output is always Prisma-ready:

```env
DATABASE_URL="postgresql://...@db.squareexp.com:6432/<branch-db>?sslmode=require"
DIRECT_URL="postgresql://...@db.squareexp.com:5432/<branch-db>?sslmode=require"
```

## Network access

If your machine cannot reach `db.squareexp.com`, allow the current IP:

```bash
axm network allow --current
axm -ne add --current
```

Other useful network commands:

```bash
axm network list
axm network allow 203.0.113.10/32 --ports both --label "Office VPN"
axm network revoke <rule-id>
axm network public-mode restricted
axm network public-mode public_runtime
```

Keep `restricted` as the default. Use public modes only when you mean it.

## Dashboard

```bash
axm dashboard
axm -da
```

The TUI shows projects, metrics, and recent audit events without leaving the terminal.

## Shortcuts

```bash
axm -li
axm -pr -ls
axm -br -url --name feature-auth
axm -ne add --current
axm -da
axm -g -tk --branch feature-auth
```

## Diagnostics

```bash
axm whoami
axm monitoring summary
axm audit list --limit 20
```
