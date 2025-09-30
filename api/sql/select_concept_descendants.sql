SELECT DISTINCT c.concept_id,
                c.concept_name,
                c.domain_id,
                c.vocabulary_id,
                c.concept_class_id,
                c.standard_concept,
                c.concept_code,
                c.invalid_reason,
                c.valid_start_date,
                c.valid_end_date,
                0 as record_count
FROM vocab_27_AUG_25.concept_ancestor a,
     vocab_27_AUG_25.concept c
WHERE a.ancestor_concept_id = $1
  AND c.concept_id = a.descendant_concept_id
  AND a.descendant_concept_id != a.ancestor_concept_id
  AND min_levels_of_separation = $2
ORDER BY c.concept_name
