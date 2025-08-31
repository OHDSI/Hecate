SELECT c.concept_id       AS concept_id,
       c.concept_name     AS concept_name,
       c.domain_id        AS domain_id,
       c.vocabulary_id    AS vocabulary_id,
       c.concept_class_id AS concept_class_id,
       c.standard_concept AS standard_concept,
       c.concept_code     AS concept_code,
       c.valid_start_date AS valid_start_date,
       c.valid_end_date   AS valid_end_date,
       c.invalid_reason   AS invalid_reason
FROM cdm.concept_relationship AS cr
         JOIN cdm.concept AS c ON cr.concept_id_2 = c.concept_id
         JOIN cdm.relationship AS r ON r.relationship_id = cr.relationship_id
WHERE cr.concept_id_1 = $1
  AND cr.relationship_id = $2
  AND cr.concept_id_1 != cr.concept_id_2
ORDER BY r.relationship_name, c.vocabulary_id, c.concept_name
