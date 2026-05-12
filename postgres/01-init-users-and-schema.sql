\set ON_ERROR_STOP on

-- Create vocabulary schema
CREATE SCHEMA IF NOT EXISTS :VOCAB_SCHEMA;

-- Create application user
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'hecate_app') THEN
        CREATE ROLE hecate_app WITH LOGIN PASSWORD 'hecate_app_password';
    END IF;
END
$$;

-- Create readonly user
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'hecate_readonly') THEN
        CREATE ROLE hecate_readonly WITH LOGIN PASSWORD 'hecate_readonly_password';
    END IF;
END
$$;

-- Grant app user full access to schema
GRANT USAGE ON SCHEMA :VOCAB_SCHEMA TO hecate_app;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA :VOCAB_SCHEMA TO hecate_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA :VOCAB_SCHEMA GRANT ALL PRIVILEGES ON TABLES TO hecate_app;

-- Grant readonly user select access
GRANT USAGE ON SCHEMA :VOCAB_SCHEMA TO hecate_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA :VOCAB_SCHEMA TO hecate_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA :VOCAB_SCHEMA GRANT SELECT ON TABLES TO hecate_readonly;
