SELECT p.relationship_id AS relationship_id,
       c.concept_id      AS concept_id,
       c.concept_name    AS concept_name,
       c.vocabulary_id   AS vocabulary_id
FROM vocab_27_AUG_25.phoebe AS p
         JOIN vocab_27_AUG_25.concept AS c ON p.concept_id_2 = c.concept_id
WHERE p.concept_id_1 = $1
ORDER BY relationship_id, vocabulary_id, concept_name
