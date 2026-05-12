#!/bin/bash
set -e

echo "========================================="
echo "Loading OHDSI Vocabulary CSV Data"
echo "========================================="

DATA_DIR="/docker-entrypoint-initdb.d/data"
VOCAB_SCHEMA="${VOCAB_SCHEMA:-vocab_27_feb_2026}"

# Set search path
export PGPASSWORD="$POSTGRES_PASSWORD"

# Function to load a CSV file into a table
load_csv() {
    local csv_file=$1
    local table_name=$2
    local schema=$3
    
    if [ -f "$csv_file" ]; then
        echo "Loading $csv_file into ${schema}.${table_name}..."
        
        # Count lines for progress reporting
        local total_lines=$(wc -l < "$csv_file")
        echo "  File contains $total_lines lines"
        
        # Load CSV with tab delimiter and handle special characters
        psql -v ON_ERROR_STOP=0 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
            \timing on
            COPY ${schema}.${table_name} FROM '${csv_file}' WITH (FORMAT CSV, HEADER true, DELIMITER E'\t', QUOTE E'\b', ENCODING 'UTF8');
EOSQL
        
        if [ $? -eq 0 ]; then
            echo "  ✓ Successfully loaded $table_name"
            # Get row count
            local row_count=$(psql -t --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -c "SELECT COUNT(*) FROM ${schema}.${table_name};" | xargs)
            echo "  Row count: $row_count"
        else
            echo "  ✗ Warning: Errors occurred while loading $table_name"
            echo "  Continuing with next file..."
        fi
        echo ""
    else
        echo "  Skipping ${table_name} - file not found: $csv_file"
    fi
}

# Check if data directory exists
if [ ! -d "$DATA_DIR" ]; then
    echo "WARNING: Data directory not found: $DATA_DIR"
    echo "No CSV files to load."
    exit 0
fi

# Check for CSV files
csv_files=$(find "$DATA_DIR" -maxdepth 1 -type f -iname "*.csv" 2>/dev/null | wc -l)

if [ "$csv_files" -eq 0 ]; then
    echo "No CSV files found in $DATA_DIR"
    echo ""
    
    # Check for SQL files as alternative
    if [ -n "$(ls -A $DATA_DIR/*.sql 2>/dev/null)" ]; then
        echo "Found SQL files instead. Processing SQL dumps..."
        for f in $DATA_DIR/*.sql; do
            if [ -f "$f" ]; then
                echo "Executing $f..."
                psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" --set=VOCAB_SCHEMA="$VOCAB_SCHEMA" < "$f"
                echo "Completed $f"
            fi
        done
    else
        echo "WARNING: No data files (CSV or SQL) found to load."
        echo "The database structure has been created but is empty."
        echo "Please refer to postgres/data/README.md for instructions."
    fi
else
    echo "Found $csv_files CSV file(s) in $DATA_DIR"
    echo "Loading into schema: $VOCAB_SCHEMA"
    echo ""
    
    # Load OHDSI vocabulary tables in dependency order
    # Core reference tables first
    load_csv "$DATA_DIR/VOCABULARY.csv" "vocabulary" "$VOCAB_SCHEMA"
    load_csv "$DATA_DIR/DOMAIN.csv" "domain" "$VOCAB_SCHEMA"
    load_csv "$DATA_DIR/CONCEPT_CLASS.csv" "concept_class" "$VOCAB_SCHEMA"
    load_csv "$DATA_DIR/RELATIONSHIP.csv" "relationship" "$VOCAB_SCHEMA"
    
    # Main concept table
    load_csv "$DATA_DIR/CONCEPT.csv" "concept" "$VOCAB_SCHEMA"
    
    # Relationship and synonym tables
    load_csv "$DATA_DIR/CONCEPT_RELATIONSHIP.csv" "concept_relationship" "$VOCAB_SCHEMA"
    load_csv "$DATA_DIR/CONCEPT_SYNONYM.csv" "concept_synonym" "$VOCAB_SCHEMA"
    load_csv "$DATA_DIR/CONCEPT_ANCESTOR.csv" "concept_ancestor" "$VOCAB_SCHEMA"
    
    # Optional tables
    load_csv "$DATA_DIR/DRUG_STRENGTH.csv" "drug_strength" "$VOCAB_SCHEMA"
    
    # Hecate-specific tables
    load_csv "$DATA_DIR/PHOEBE.csv" "phoebe" "$VOCAB_SCHEMA"
    load_csv "$DATA_DIR/phoebe.csv" "phoebe" "$VOCAB_SCHEMA"  # try lowercase
    
    echo "========================================="
    echo "CSV Data Loading Complete"
    echo "========================================="
    
    # Show summary of loaded data
    echo ""
    echo "Table Summary:"
    psql --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
        SELECT 
            schemaname,
            tablename,
            pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size,
            (SELECT COUNT(*) FROM ${VOCAB_SCHEMA}.\${tablename}) AS row_count
        FROM pg_tables
        WHERE schemaname = '${VOCAB_SCHEMA}'
        ORDER BY tablename;
EOSQL
fi

echo ""
echo "========================================="
echo "Database initialization complete!"
echo "========================================="
