FROM postgres:15

# Copy initialization scripts in order (executed alphabetically)
# 01 - Create users, roles, and schema
COPY 01-init-users-and-schema.sql /docker-entrypoint-initdb.d/

# 02 - Create OHDSI vocabulary table structure
COPY 02-create-tables.sql /docker-entrypoint-initdb.d/

# 03 - Load CSV data (must run after tables are created)
COPY init-db.sh /docker-entrypoint-initdb.d/03-load-data.sh

# Copy data directory containing CSV/SQL files
# These will be loaded by the init-db.sh script
COPY data/ /docker-entrypoint-initdb.d/data/

# Set default environment variables
ENV POSTGRES_DB=hecate
ENV POSTGRES_USER=postgres
ENV VOCAB_SCHEMA=vocab_27_feb_2026

# Expose PostgreSQL port
EXPOSE 5432

# The postgres:15 image already has an entrypoint that handles initialization
# Scripts in /docker-entrypoint-initdb.d/ are executed in alphabetical order on first run:
# 1. 01-init-users-and-schema.sql - Creates users, schemas, privileges
# 2. 02-create-tables.sql - Creates OHDSI vocabulary tables
# 3. 03-load-data.sh - Loads CSV files into tables
