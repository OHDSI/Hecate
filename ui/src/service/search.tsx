import { ConceptRow, SearchResponse } from "../@types/data-source";
import { createApiClient } from "../config/api";
import axios from "axios";

export const autocomplete = async (q: string): Promise<[string]> => {
  const client = axios.create({
    baseURL: "https://hecate.pantheon-hds.com/api",
  });

  return client
    .get<[string]>(`/autocomplete?q=${q}`)
    .then((resp) => {
      return resp.data;
    })
    .catch((err) => {
      throw err;
    });
};

export function expandStandardConceptAbbreviation(
  abbreviation: string | undefined,
) {
  if (abbreviation === "S") {
    return "Standard";
  } else if (abbreviation === "C") {
    return "Classification";
  } else {
    return "Non-standard";
  }
}

export function expandInvalidReasonAbbreviation(
  abbreviation: string | undefined,
) {
  if (abbreviation === "D") {
    return "Deprecated";
  } else if (abbreviation === "U") {
    return "Upgraded";
  } else {
    return "Valid";
  }
}

export const search = async (q: string): Promise<ConceptRow[]> => {
  if (!q || q.length === 0) {
    return [];
  }
  const client = createApiClient();

  const encodedQuery = encodeURIComponent(q);
  return client
    .get<SearchResponse[]>(`/search?q=${encodedQuery}&limit=250`)
    .then((resp) => {
      return resp.data
        .filter((resp) => resp.concepts)
        .map((resp) => {
          if (resp.concepts.length === 1) {
            return {
              concept_id: resp.concepts[0].concept_id,
              concept_name: resp.concepts[0].concept_name,
              domain_id: [resp.concepts[0].domain_id],
              vocabulary_id: [resp.concepts[0].vocabulary_id],
              concept_class_id: [resp.concepts[0].concept_class_id],
              concept_code: resp.concepts[0].concept_code,
              standard_concept: [
                expandStandardConceptAbbreviation(
                  resp.concepts[0].standard_concept,
                ),
              ],
              invalid_reason: [
                expandInvalidReasonAbbreviation(
                  resp.concepts[0].invalid_reason,
                ),
              ],
              score: resp.score,
              children: undefined,
            };
          } else {
            console.log("in the else");
            const children = resp.concepts.map((c) => {
              return {
                concept_id: c.concept_id,
                concept_name: c.concept_name,
                domain_id: [c.domain_id],
                vocabulary_id: [c.vocabulary_id],
                concept_class_id: [c.concept_class_id],
                concept_code: c.concept_code,
                standard_concept: [
                  expandStandardConceptAbbreviation(c.standard_concept),
                ],
                invalid_reason: [
                  expandInvalidReasonAbbreviation(c.invalid_reason),
                ],
                score: resp.score,
                children: undefined,
              };
            });
            return {
              concept_name: resp.concept_name,
              domain_id: [...new Set(children.map((c) => c.domain_id[0]))],
              vocabulary_id: [
                ...new Set(children.map((c) => c.vocabulary_id[0])),
              ],
              concept_class_id: [
                ...new Set(children.map((c) => c.concept_class_id[0])),
              ],
              standard_concept: [
                ...new Set(children.map((c) => c.standard_concept[0])),
              ],
              invalid_reason: [
                ...new Set(children.map((c) => c.invalid_reason[0])),
              ],
              score: resp.score,
              children: children,
            };
          }
        });
    })
    .catch((err) => {
      throw err;
    });
};
