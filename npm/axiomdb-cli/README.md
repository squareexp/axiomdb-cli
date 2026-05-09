# AxiomDB CLI

Fast terminal control for AxiomDB projects, branches, Prisma URLs, network access, metrics, backups, audit, and secrets.

## Install

```bash
npm install -g axiomdb-cli
```

The package exposes `axm` for speed and keeps `axiom` for compatibility.

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

## Daily flow

```bash
axm projects list
axm projects use "Square Experience"
axm branches list
axm branches create --name feature-auth --lifespan 7d
axm branches urls --name feature-auth
axm network allow --current
axm dashboard
```

Branch URLs are Prisma-ready:

```env
DATABASE_URL="postgresql://...@db.squareexp.com:6432/<branch-db>?sslmode=require"
DIRECT_URL="postgresql://...@db.squareexp.com:5432/<branch-db>?sslmode=require"
```

## Shortcuts

```bash
axm -li
axm -pr -ls
axm -br -url --name feature-auth
axm -ne add --current
axm -da
axm -g -tk --branch feature-auth
```
