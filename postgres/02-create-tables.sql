-- ================================================
-- OHDSI Vocabulary Table Definitions
-- Creates the standard OHDSI CDM vocabulary tables
-- ================================================

\set ON_ERROR_STOP on

\echo 'Creating OHDSI vocabulary tables in schema: ' :VOCAB_SCHEMA

-- Set search path to vocabulary schema
SET search_path TO :VOCAB_SCHEMA;

-- CONCEPT table
CREATE TABLE IF NOT EXISTS concept (
    concept_id          INTEGER       NOT NULL,
    concept_name        VARCHAR(255)  NOT NULL,
    domain_id           VARCHAR(20)   NOT NULL,
    vocabulary_id       VARCHAR(20)   NOT NULL,
    concept_class_id    VARCHAR(20)   NOT NULL,
    standard_concept    VARCHAR(1),
    concept_code        VARCHAR(50)   NOT NULL,
    valid_start_date    DATE          NOT NULL,
    valid_end_date      DATE          NOT NULL,
    invalid_reason      VARCHAR(1),
    CONSTRAINT pk_concept PRIMARY KEY (concept_id)
);

-- VOCABULARY table
CREATE TABLE IF NOT EXISTS vocabulary (
    vocabulary_id           VARCHAR(20)   NOT NULL,
    vocabulary_name         VARCHAR(255)  NOT NULL,
    vocabulary_reference    VARCHAR(255),
    vocabulary_version      VARCHAR(255),
    vocabulary_concept_id   INTEGER       NOT NULL,
    CONSTRAINT pk_vocabulary PRIMARY KEY (vocabulary_id)
);

-- DOMAIN table
CREATE TABLE IF NOT EXISTS domain (
    domain_id           VARCHAR(20)   NOT NULL,
    domain_name         VARCHAR(255)  NOT NULL,
    domain_concept_id   INTEGER       NOT NULL,
    CONSTRAINT pk_domain PRIMARY KEY (domain_id)
);

-- CONCEPT_CLASS table
CREATE TABLE IF NOT EXISTS concept_class (
    concept_class_id          VARCHAR(20)   NOT NULL,
    concept_class_name        VARCHAR(255)  NOT NULL,
    concept_class_concept_id  INTEGER       NOT NULL,
    CONSTRAINT pk_concept_class PRIMARY KEY (concept_class_id)
);

-- CONCEPT_RELATIONSHIP table
CREATE TABLE IF NOT EXISTS concept_relationship (
    concept_id_1      INTEGER       NOT NULL,
    concept_id_2      INTEGER       NOT NULL,
    relationship_id   VARCHAR(20)   NOT NULL,
    valid_start_date  DATE          NOT NULL,
    valid_end_date    DATE          NOT NULL,
    invalid_reason    VARCHAR(1),
    CONSTRAINT pk_concept_relationship PRIMARY KEY (concept_id_1, concept_id_2, relationship_id)
);

-- RELATIONSHIP table
CREATE TABLE IF NOT EXISTS relationship (
    relationship_id            VARCHAR(20)   NOT NULL,
    relationship_name          VARCHAR(255)  NOT NULL,
    is_hierarchical            VARCHAR(1)    NOT NULL,
    defines_ancestry           VARCHAR(1)    NOT NULL,
    reverse_relationship_id    VARCHAR(20)   NOT NULL,
    relationship_concept_id    INTEGER       NOT NULL,
    CONSTRAINT pk_relationship PRIMARY KEY (relationship_id)
);

-- CONCEPT_SYNONYM table
CREATE TABLE IF NOT EXISTS concept_synonym (
    concept_id            INTEGER       NOT NULL,
    concept_synonym_name  VARCHAR(1000) NOT NULL,
    language_concept_id   INTEGER       NOT NULL
);

-- CONCEPT_ANCESTOR table
CREATE TABLE IF NOT EXISTS concept_ancestor (
    ancestor_concept_id       INTEGER  NOT NULL,
    descendant_concept_id     INTEGER  NOT NULL,
    min_levels_of_separation  INTEGER  NOT NULL,
    max_levels_of_separation  INTEGER  NOT NULL,
    CONSTRAINT pk_concept_ancestor PRIMARY KEY (ancestor_concept_id, descendant_concept_id)
);

-- DRUG_STRENGTH table (optional, but commonly used)
CREATE TABLE IF NOT EXISTS drug_strength (
    drug_concept_id             INTEGER,
    ingredient_concept_id       INTEGER,
    amount_value                NUMERIC,
    amount_unit_concept_id      INTEGER,
    numerator_value             NUMERIC,
    numerator_unit_concept_id   INTEGER,
    denominator_value           NUMERIC,
    denominator_unit_concept_id INTEGER,
    box_size                    INTEGER,
    valid_start_date            DATE     NOT NULL,
    valid_end_date              DATE     NOT NULL,
    invalid_reason              VARCHAR(1)
);

-- PHOEBE table (Hecate-specific for concept recommendations)
CREATE TABLE IF NOT EXISTS phoebe (
    concept_id_1    INTEGER,
    concept_id_2    INTEGER,
    relationship_id VARCHAR(20)
);

\echo 'OHDSI vocabulary tables created successfully'
