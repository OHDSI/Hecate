# ============================================
# Stage 1: Initialize database at build time
# ============================================
FROM postgres:15 AS builder

# Set environment variables for initialization
ENV POSTGRES_DB=hecate
ENV POSTGRES_USER=postgres
ENV POSTGRES_PASSWORD=build_temp_password
ENV VOCAB_SCHEMA=vocab_27_feb_2026
ENV PGDATA=/var/lib/postgresql/data

# Copy initialization scripts
COPY 01-init-users-and-schema.sql /docker-entrypoint-initdb.d/
COPY 02-create-tables.sql /docker-entrypoint-initdb.d/
COPY init-db.sh /docker-entrypoint-initdb.d/03-load-data.sh

# Copy CSV data files
COPY data/ /docker-entrypoint-initdb.d/data/

# Make scripts executable
RUN chmod +x /docker-entrypoint-initdb.d/03-load-data.sh

# Initialize the database during build
# Start postgres, let init scripts run, then stop cleanly
RUN set -e; \
    # Initialize the database cluster
    gosu postgres initdb; \
    # Start PostgreSQL in background
    gosu postgres pg_ctl -D "$PGDATA" -o "-c listen_addresses=''" -w start; \
    # Run initialization scripts manually
    gosu postgres psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -v VOCAB_SCHEMA="$VOCAB_SCHEMA" -f /docker-entrypoint-initdb.d/01-init-users-and-schema.sql; \
    gosu postgres psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -v VOCAB_SCHEMA="$VOCAB_SCHEMA" -f /docker-entrypoint-initdb.d/02-create-tables.sql; \
    gosu postgres bash /docker-entrypoint-initdb.d/03-load-data.sh; \
    # Stop PostgreSQL cleanly
    gosu postgres pg_ctl -D "$PGDATA" -m fast -w stop; \
    echo "Database initialization complete!"

# ============================================
# Stage 2: Final runtime image with pre-initialized data
# ============================================
FROM postgres:15

# Set runtime environment variables (POSTGRES_PASSWORD should be set at runtime)
ENV POSTGRES_DB=hecate
ENV POSTGRES_USER=postgres
ENV VOCAB_SCHEMA=vocab_27_feb_2026

# Copy the initialized database from builder stage
COPY --from=builder /var/lib/postgresql/data /var/lib/postgresql/data

# Set proper ownership
RUN chown -R postgres:postgres /var/lib/postgresql/data

# Expose PostgreSQL port
EXPOSE 5432

# Note: The database is already initialized with:
# - Users: hecate_app, hecate_readonly (change passwords in 01-init script)
# - Schema: vocab_27_feb_2026 (or custom via VOCAB_SCHEMA)
# - Tables: All OHDSI vocabulary tables created
# - Data: CSV files loaded
# Container starts immediately without initialization delay!
