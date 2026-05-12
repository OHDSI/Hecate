# Hecate PostgreSQL Docker Image

Pre-loaded OHDSI vocabulary database (schema `vocab_27_feb_2026`).

## How it works

**Build time** — loads 5.6 GB of OHDSI vocabulary CSVs into PostgreSQL, then dumps the result as a compressed `hecate.pgdump` (~1.1 GB). Final image is ~1.6 GB.

**First `docker run`** — restores from dump into a fresh volume (~3–5 min). Subsequent starts are instant.

## Build

```bash
docker build -t hecate-postgres .
```

## Run

```bash
docker rm -f hecate-postgres 2>/dev/null; docker run -d --name hecate-postgres -p 5433:5432 -e POSTGRES_PASSWORD=yourpassword hecate-postgres
```

> Uses port **5433** to avoid conflicts with a local PostgreSQL on 5432.

Wait for ready (first run takes ~3–5 min):

```bash
docker logs -f hecate-postgres 2>&1 | grep "ready to accept connections"
```

Ready when that line appears **twice** (second = restore complete).

## Test

```bash
PGPASSWORD=yourpassword psql -h localhost -p 5433 -U postgres -d hecate -c "SELECT COUNT(*) FROM vocab_27_feb_2026.concept;"
```

Expected: **7 527 642 rows**

## Users

| User | Password | Access |
|------|----------|--------|
| `postgres` | set via `POSTGRES_PASSWORD` | superuser |
| `hecate_app` | `hecate_app_password` | read/write on `vocab_27_feb_2026` |
| `hecate_readonly` | `hecate_readonly_password` | read-only on `vocab_27_feb_2026` |
