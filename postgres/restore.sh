#!/bin/bash
set -e

echo "========================================="
echo "First-time restore from dump"
echo "========================================="

# Roles are global objects not captured by pg_dump — create them first
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    DO \$\$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'hecate_app') THEN
            CREATE ROLE hecate_app WITH LOGIN PASSWORD 'hecate_app_password';
        END IF;
    END
    \$\$;
    DO \$\$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'hecate_readonly') THEN
            CREATE ROLE hecate_readonly WITH LOGIN PASSWORD 'hecate_readonly_password';
        END IF;
    END
    \$\$;
EOSQL

echo "Restoring schema + data (takes a few minutes)..."
pg_restore \
    --username "$POSTGRES_USER" \
    --dbname "$POSTGRES_DB" \
    /docker-entrypoint-initdb.d/hecate.pgdump

echo "========================================="
echo "Restore complete!"
echo "========================================="
