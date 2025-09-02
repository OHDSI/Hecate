SELECT concept_name
FROM vocab_27_AUG_25.concept
WHERE concept_id = $1
   OR concept_code = $2
