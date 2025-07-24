SELECT r.relationship_name AS relationship_id,
       c.concept_id        AS concept_id,
       c.concept_name      AS concept_name,
       c.vocabulary_id     AS vocabulary_id
FROM cdm.concept_relationship AS cr
         JOIN cdm.concept AS c ON cr.concept_id_2 = c.concept_id
         JOIN cdm.relationship AS r ON r.relationship_id = cr.relationship_id
WHERE cr.concept_id_1 = $1 ORDER BY r.relationship_name, c.vocabulary_id, c.concept_name
