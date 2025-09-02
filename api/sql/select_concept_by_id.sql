SELECT concept_id,
       concept_name,
       domain_id,
       vocabulary_id,
       concept_class_id,
       standard_concept,
       concept_code,
       invalid_reason,
       valid_start_date,
       valid_end_date
FROM vocab_27_AUG_25.concept
WHERE concept_id = $1
