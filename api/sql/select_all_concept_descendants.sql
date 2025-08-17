SELECT descendant_concept_id AS concept_id
FROM cdm.concept_ancestor
WHERE ancestor_concept_id = $1
  AND min_levels_of_separation > 0
