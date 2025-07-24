import {
  Concept,
  ConceptExpandRow,
  RelatedConcept,
} from "../@types/data-source";
import { createApiClient, API_V2_BASE_URL } from "../config/api";

export const getConceptById = async (id: number): Promise<[Concept]> => {
  const client = createApiClient();

  return client
    .get<[Concept]>(`/concepts/${id}`)
    .then((resp) => {
      return resp.data;
    })
    .catch((err) => {
      throw err;
    });
};

export const getRelatedConcepts = async (
  id: number,
): Promise<[RelatedConcept]> => {
  const client = createApiClient();

  return client
    .get<[RelatedConcept]>(`/concepts/${id}/relationships`)
    .then((resp) => {
      return resp.data;
    })
    .catch((err) => {
      throw err;
    });
};

export const getPhoebeConcepts = async (
  id: number,
): Promise<[RelatedConcept]> => {
  const client = createApiClient();

  return client
    .get<[RelatedConcept]>(`/concepts/${id}/phoebe`)
    .then((resp) => {
      return resp.data;
    })
    .catch((err) => {
      throw err;
    });
};

export const getConceptDefinition = async (id: number): Promise<string> => {
  const client = createApiClient();

  return client
    .get<string>(`/concepts/${id}/definition`)
    .then((resp) => {
      return resp.data;
    })
    .catch((err) => {
      throw err;
    });
};

export const getConceptExpand = async (
  id: number,
): Promise<ConceptExpandRow[]> => {
  const client = createApiClient(API_V2_BASE_URL);

  return client
    .get<{ concepts: ConceptExpandRow[] }>(
      `/concepts/${id}/expand?childlevels=5&parentlevels=0`,
    )
    .then((resp) => {
      if (resp.data.concepts[0].children) {
        return resp.data.concepts[0].children;
      } else {
        return [];
      }
    })
    .catch((err) => {
      throw err;
    });
};
