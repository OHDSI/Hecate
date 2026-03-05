SELECT concept_name
FROM {VOCAB_SCHEMA}.concept
WHERE concept_id = $1
   OR concept_code = $2
