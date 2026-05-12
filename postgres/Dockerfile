# ============================================
# Stage 1: Load data and dump at build time
# ============================================
FROM postgres:15 AS builder

ENV POSTGRES_DB=hecate
ENV POSTGRES_USER=postgres
ENV POSTGRES_PASSWORD=build_temp_password
ENV VOCAB_SCHEMA=vocab_27_feb_2026
ENV PGDATA=/var/lib/postgresql/data

COPY 01-init-users-and-schema.sql /docker-entrypoint-initdb.d/
COPY 02-create-tables.sql /docker-entrypoint-initdb.d/
COPY init-db.sh /docker-entrypoint-initdb.d/03-load-data.sh
COPY data/ /docker-entrypoint-initdb.d/data/

RUN chmod +x /docker-entrypoint-initdb.d/03-load-data.sh

RUN set -e; \
    gosu postgres initdb; \
    gosu postgres pg_ctl -D "$PGDATA" -o "-c listen_addresses=''" -w start; \
    gosu postgres psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "postgres" -c "CREATE DATABASE $POSTGRES_DB;"; \
    gosu postgres psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -v VOCAB_SCHEMA="$VOCAB_SCHEMA" -f /docker-entrypoint-initdb.d/01-init-users-and-schema.sql; \
    gosu postgres psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -v VOCAB_SCHEMA="$VOCAB_SCHEMA" -f /docker-entrypoint-initdb.d/02-create-tables.sql; \
    gosu postgres bash /docker-entrypoint-initdb.d/03-load-data.sh; \
    echo "Dumping database..."; \
    gosu postgres pg_dump -Fc --username "$POSTGRES_USER" "$POSTGRES_DB" -f /tmp/hecate.pgdump; \
    gosu postgres pg_ctl -D "$PGDATA" -m fast -w stop; \
    echo "Dump size: $(du -sh /tmp/hecate.pgdump)"

# ============================================
# Stage 2: Lean runtime image with dump only
# ============================================
FROM postgres:15

ENV POSTGRES_DB=hecate
ENV POSTGRES_USER=postgres
ENV VOCAB_SCHEMA=vocab_27_feb_2026

# Compressed dump (~1-2GB vs ~10GB raw PGDATA)
COPY --from=builder /tmp/hecate.pgdump /docker-entrypoint-initdb.d/hecate.pgdump

# Runs once on first docker run when PGDATA is empty
COPY restore.sh /docker-entrypoint-initdb.d/01-restore.sh
RUN chmod +x /docker-entrypoint-initdb.d/01-restore.sh

EXPOSE 5432
