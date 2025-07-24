SELECT concept_name
FROM cdm.concept
WHERE concept_id = $1
   OR concept_code = $2
