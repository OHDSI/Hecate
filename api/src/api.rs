use crate::concept_graph::ConceptExpander;
use crate::domain::{Concept, SearchResponse};
use crate::embeddings::fetch_embeddings;
use crate::errors::PgError;
use crate::umls::get_umls_definition_from_nlm;
use crate::utils::deserialize_string_or_vec;
use crate::validation;
use crate::{StateWrapper, db};
use actix_web::web::{Data, Json, Query};
use actix_web::{Error, HttpResponse, get, post, web};
use log::info;
use qdrant_client::qdrant::condition::ConditionOneOf;
use qdrant_client::qdrant::point_id::PointIdOptions;
use qdrant_client::qdrant::{
    Condition, Filter, GetPointsBuilder, PointId, RecommendInputBuilder, RetrievedPoint,
    ScoredPoint, ScrollPointsBuilder, SearchPointsBuilder,
};
use qdrant_client::{Qdrant, qdrant};
use serde::Deserialize;
use std::collections::HashMap;

pub const CONCEPT_COLLECTION: &str = "meddra";
pub const SYNONYMS_COLLECTION: &str = "synonyms";

#[derive(Deserialize)]
struct Parameters {
    q: String,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    vocabulary_id: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    exclude_vocabulary_id: Option<Vec<String>>,
    standard_concept: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    domain_id: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    concept_class_id: Option<Vec<String>>,
    limit: Option<u64>,
}

#[derive(Deserialize)]
struct ConceptSetValidationRequest {
    concept_set: String,
}

#[derive(Deserialize)]
struct ExpandParams {
    childlevels: Option<i32>,
    parentlevels: Option<i32>,
}

async fn process_search_results(
    search_results: Vec<SearchResponse>,
    concept_map: &mut HashMap<i32, (Concept, f64)>,
    parameters: &Parameters,
    state: &Data<StateWrapper>,
    limit: usize,
) -> Result<(), Error> {
    // First pass: collect all concepts with their best scores
    let mut all_concepts: HashMap<i32, (Concept, f64)> = HashMap::new();

    for sr in search_results {
        for concept in &sr.concepts {
            if concept
                .standard_concept
                .as_ref()
                .is_some_and(|sc| sc.eq_ignore_ascii_case("S"))
            {
                let filtered_concepts = filter_and_enrich_concepts(
                    vec![concept.clone()],
                    parameters,
                    &state.concept_record_counts,
                );
                if !filtered_concepts.is_empty() {
                    let filtered_concept = &filtered_concepts[0];
                    let score = sr.score.unwrap();

                    // Keep the highest score for each concept_id
                    if let Some((_, existing_score)) = all_concepts.get(&filtered_concept.concept_id) {
                        if score > *existing_score {
                            all_concepts.insert(
                                filtered_concept.concept_id,
                                (filtered_concept.clone(), score),
                            );
                        }
                    } else {
                        all_concepts.insert(
                            filtered_concept.concept_id,
                            (filtered_concept.clone(), score),
                        );
                    }
                }
            } else {
                let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
                let standard_concepts = db::map_to_standard(&pg_client, concept.concept_id).await?;
                let filtered_standard_concepts = filter_and_enrich_concepts(
                    standard_concepts,
                    parameters,
                    &state.concept_record_counts,
                );
                for std_concept in filtered_standard_concepts {
                    let score = sr.score.unwrap();

                    // Keep the highest score for each concept_id
                    if let Some((_, existing_score)) = all_concepts.get(&std_concept.concept_id) {
                        if score > *existing_score {
                            all_concepts.insert(std_concept.concept_id, (std_concept, score));
                        }
                    } else {
                        all_concepts.insert(std_concept.concept_id, (std_concept, score));
                    }
                }
            }
        }
    }

    // Second pass: merge with existing concept_map, keeping best scores
    for (concept_id, (concept, score)) in all_concepts {
        if let Some((_, existing_score)) = concept_map.get(&concept_id) {
            if score > *existing_score {
                concept_map.insert(concept_id, (concept, score));
            }
        } else {
            concept_map.insert(concept_id, (concept, score));
        }
    }

    // Third pass: if over limit, keep only top N by score
    if concept_map.len() > limit {
        let mut concepts_vec: Vec<(i32, (Concept, f64))> = concept_map.drain().collect();
        concepts_vec.sort_by(|a, b| {
            b.1.1.partial_cmp(&a.1.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        concepts_vec.truncate(limit);
        *concept_map = concepts_vec.into_iter().collect();
    }

    Ok(())
}

#[get("/api/search_standard")]
async fn search_standard(
    parameters: Query<Parameters>,
    state: Data<StateWrapper>,
) -> Result<Json<Vec<SearchResponse>>, Error> {
    let limit = parameters.limit.unwrap_or(25) as usize;
    let mut query_string = format!("q={}", parameters.q);
    if let Some(exclude_vocab_ids) = &parameters.exclude_vocabulary_id {
        let exclude_vocab_str = exclude_vocab_ids.join(",");
        query_string.push_str(&format!("&exclude_vocabulary_id={}", exclude_vocab_str));
    }
    let resp = search(
        Query::from_query(&query_string)?,
        state.clone(),
        CONCEPT_COLLECTION,
    )
    .await;

    // Additional search call with synonyms collection
    let synonyms_resp = search(
        Query::from_query(&query_string)?,
        state.clone(),
        SYNONYMS_COLLECTION,
    )
    .await;

    let mut concept_map: HashMap<i32, (Concept, f64)> = HashMap::new();
    let main_search_results = resp?;
    let synonyms_search_results = synonyms_resp?;

    // Process main search results
    process_search_results(
        main_search_results,
        &mut concept_map,
        &parameters,
        &state,
        limit,
    )
    .await?;

    // Process synonyms search results
    process_search_results(
        synonyms_search_results,
        &mut concept_map,
        &parameters,
        &state,
        limit,
    )
    .await?;

    let mut grouped_concepts: HashMap<String, (f64, Vec<Concept>)> = HashMap::new();

    for (concept, score) in concept_map.into_values() {
        let name_lower = concept.concept_name.to_lowercase();

        if let Some((existing_score, concepts)) = grouped_concepts.get_mut(&name_lower) {
            if score > *existing_score {
                *existing_score = score;
            }
            concepts.push(concept);
        } else {
            grouped_concepts.insert(name_lower, (score, vec![concept]));
        }
    }

    let mut search_responses: Vec<SearchResponse> = grouped_concepts
        .into_iter()
        .map(|(name_lower, (score, concepts))| SearchResponse {
            concept_name: concepts[0].concept_name.clone(),
            concept_name_lower: name_lower,
            score: Some(score),
            concepts,
        })
        .collect();

    search_responses.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(Json(search_responses))
}

#[get("/api/search")]
async fn search_api(
    parameters: Query<Parameters>,
    state: Data<StateWrapper>,
) -> Result<Json<Vec<SearchResponse>>, Error> {
    Ok(Json(search(parameters, state, CONCEPT_COLLECTION).await?))
}

async fn search(
    parameters: Query<Parameters>,
    state: Data<StateWrapper>,
    collection_name: &str,
) -> Result<Vec<SearchResponse>, Error> {
    let client = &state.qdrant_client;
    let input = parameters.q.trim();
    let lowercase_input = input.to_lowercase();
    info!("Received search request for {:?}", &input);
    let opt_existing = if collection_name == CONCEPT_COLLECTION {
        state.concept_index.get(lowercase_input.as_str())
    } else {
        None
    };
    let mut to_return: Vec<SearchResponse> = Vec::new();
    let mut ids: Vec<String> = Vec::new();
    if let Some(existing) = opt_existing {
        existing.iter().for_each(|x| ids.push(x.to_string()));
    } else {
        info!("Nothing found in search index");
        let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
        let numeric_id = input.parse::<i32>();
        let concepts = match numeric_id {
            Ok(_) => db::get_concept_name_by_number(&pg_client, numeric_id.unwrap()).await?,
            Err(_) => db::get_concept_name_by_string(&pg_client, input.to_string()).await?,
        };

        if !concepts.is_empty() {
            for c in concepts {
                let lower = c.to_lowercase();
                info!("{}", lower);
                let res = if collection_name == CONCEPT_COLLECTION {
                    state.concept_index.get(lower.as_str())
                } else {
                    None
                };
                if let Some(item) = res {
                    item.iter().for_each(|x| ids.push(x.to_string()))
                } else {
                    let results: Vec<RetrievedPoint> =
                        find_by_concept_name_lower(client, lower, collection_name).await;
                    results.iter().for_each(|x| {
                        if let PointIdOptions::Uuid(id) =
                            x.clone().id.unwrap().point_id_options.unwrap()
                        {
                            ids.push(id.to_string());
                        }
                    });
                }
            }
        } else {
            let limit = parameters.limit.unwrap_or(100);
            // Request more results from qdrant to account for filtering
            let search_limit = 250;
            let recommendations =
                recommend(input.to_string(), client, search_limit, collection_name).await;
            for sp in recommendations {
                let mut concept: SearchResponse = SearchResponse::from(sp);
                // Apply filters after retrieval due to performance issues with filtering in qdrant
                concept.concepts = filter_and_enrich_concepts(
                    concept.concepts,
                    &parameters,
                    &state.concept_record_counts,
                );
                if concept.concepts.is_empty() {
                    continue;
                }
                // case desensification
                let mut contains_case_insensitive_exact_match = false;
                to_return = to_return
                    .into_iter()
                    .map(|mut every| {
                        if every.concept_name_lower.eq(&concept.concept_name_lower) {
                            every.append_concepts(&mut concept.concepts);
                            contains_case_insensitive_exact_match = true;
                            every
                        } else {
                            every
                        }
                    })
                    .collect();
                if !contains_case_insensitive_exact_match {
                    to_return.push(concept);
                }
            }
            // Sort by score descending and apply limit
            to_return.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if to_return.len() > limit as usize {
                to_return.truncate(limit as usize);
            }
            return Ok(to_return);
        }
    }
    let mut points: Vec<PointId> = Vec::new();
    let mut recs = RecommendInputBuilder::default();
    for id in ids {
        points.push(PointId::from(id.as_str()));
        recs = recs.add_positive(PointId::from(id.as_str()));
    }
    create_response_from_vector_db_ids(
        client,
        to_return,
        recs,
        points,
        &parameters,
        collection_name,
        &state.concept_record_counts,
    )
    .await
}

#[get("/api/concepts/{id}")]
async fn get_concept_by_id(
    path: web::Path<i32>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    info!("Get concept {}", &id);
    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let mut concept = db::get_concept_by_id(&pg_client, id).await?;
    concept.record_count = state.concept_record_counts.get(&id).copied().unwrap_or(0);
    Ok(HttpResponse::Ok().json([concept]))
}

#[get("/api/concepts/{id}/relationships")]
async fn get_concept_relationships(
    path: web::Path<i32>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    info!("Get concept {} relationships", &id);
    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let mut concepts = db::get_concept_relationships(&pg_client, id).await?;

    // Enrich with record counts
    for concept in &mut concepts {
        concept.record_count = state
            .concept_record_counts
            .get(&concept.concept_id)
            .copied()
            .unwrap_or(0);
    }

    Ok(HttpResponse::Ok().json(concepts))
}

#[get("/api/concepts/{id}/phoebe")]
async fn get_concept_phoebe(
    path: web::Path<i32>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    info!("Get concept {} phoebe", &id);
    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let mut concepts = db::get_concept_phoebe(&pg_client, id).await?;

    // Enrich with record counts
    for concept in &mut concepts {
        concept.record_count = state
            .concept_record_counts
            .get(&concept.concept_id)
            .copied()
            .unwrap_or(0);
    }

    Ok(HttpResponse::Ok().json(concepts))
}

#[get("/api/concepts/{id}/definition")]
async fn get_concept_definition(
    path: web::Path<i32>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    info!("Get concept {} definition", &id);
    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let concept = db::get_concept_by_id(&pg_client, id).await?;
    let def = get_umls_definition_from_nlm(concept.concept_name)
        .await
        .unwrap()
        .unwrap_or("No definition available".parse()?);
    Ok(HttpResponse::Ok().json(def))
}

#[get("/api/concepts/{id}/expand")]
async fn get_concept_expand(
    path: web::Path<i32>,
    params: Query<ExpandParams>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    info!(
        "Get concept {} expand with params: childlevels={:?}, parentlevels={:?}",
        &id, params.childlevels, params.parentlevels
    );

    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let expand_response = ConceptExpander::expand(
        &pg_client,
        &state.expand_cache,
        &state.children_cache,
        &state.parents_cache,
        &state.concept_record_counts,
        id,
        params.childlevels,
        params.parentlevels,
    )
    .await?;

    Ok(HttpResponse::Ok().json(expand_response))
}

async fn create_response_from_vector_db_ids(
    client: &Qdrant,
    mut to_return: Vec<SearchResponse>,
    _recs: RecommendInputBuilder,
    points: Vec<PointId>,
    parameters: &Parameters,
    collection_name: &str,
    record_counts: &HashMap<i32, i64>,
) -> Result<Vec<SearchResponse>, Error> {
    let search_result = retrieve_point_from_db(client, points, collection_name).await;
    let limit = parameters.limit.unwrap_or(100);

    // Extract vector from the first matched point to use for search
    let search_vector = if let Some(first_point) = search_result.first() {
        if let Some(vectors) = &first_point.vectors {
            match vectors.vectors_options.as_ref() {
                Some(qdrant::vectors_output::VectorsOptions::Vector(vector)) => {
                    Some(vector.data.clone())
                }
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    // Use Search API instead of Query/Recommend API for consistent results
    let neighbours = if let Some(vector) = search_vector {
        client
            .search_points(
                SearchPointsBuilder::new(collection_name, vector, 500)
                    .with_payload(true)
                    .score_threshold(0.50),
            )
            .await
            .unwrap()
            .result
    } else {
        Vec::new()
    };

    // Collect the point IDs from search_result to exclude them from neighbours
    // (they're already being returned and would cause duplicates)
    let search_result_ids: std::collections::HashSet<String> = search_result
        .iter()
        .filter_map(|p| {
            p.id.as_ref().and_then(|id| {
                if let Some(PointIdOptions::Uuid(uuid)) = &id.point_id_options {
                    Some(uuid.clone())
                } else {
                    None
                }
            })
        })
        .collect();

    // Add search_result items first (these are the exact matches)
    for retrieved_point in search_result {
        let mut concept = SearchResponse::from(retrieved_point);
        concept.concepts = filter_and_enrich_concepts(concept.concepts, parameters, record_counts);
        if concept.concepts.is_empty() {
            continue;
        }
        let mut didwehit = false;
        to_return = to_return
            .into_iter()
            .map(|mut every| {
                if every.concept_name_lower.eq(&concept.concept_name_lower) {
                    every.append_concepts(&mut concept.concepts);
                    didwehit = true;
                    every
                } else {
                    every
                }
            })
            .collect();
        if !didwehit {
            to_return.push(concept);
        }
    }

    // Add neighbours, but exclude items that were already in search_result
    for scored_point in neighbours {
        // Skip if this point was already added from search_result
        if let Some(id) = &scored_point.id {
            if let Some(PointIdOptions::Uuid(uuid)) = &id.point_id_options {
                if search_result_ids.contains(uuid) {
                    continue;
                }
            }
        }
        let mut concept = SearchResponse::from(scored_point);
        // Apply filters after retrieval due to performance issues with filtering in qdrant
        concept.concepts = filter_and_enrich_concepts(concept.concepts, parameters, record_counts);
        if concept.concepts.is_empty() {
            continue;
        }
        let mut didwehit = false;
        to_return = to_return
            .into_iter()
            .map(|mut every| {
                if every.concept_name_lower.eq(&concept.concept_name_lower) {
                    every.append_concepts(&mut concept.concepts);
                    didwehit = true;
                    every
                } else {
                    every
                }
            })
            .collect();
        if !didwehit {
            to_return.push(concept);
        }
    }

    // Sort by score descending and apply limit
    to_return.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if to_return.len() > limit as usize {
        to_return.truncate(limit as usize);
    }

    Ok(to_return)
}

async fn find_by_concept_name_lower(
    client: &Qdrant,
    concept_name_lower: String,
    collection: &str,
) -> Vec<RetrievedPoint> {
    client
        .scroll(
            ScrollPointsBuilder::new(collection).filter(Filter::must([Condition {
                condition_one_of: Some(ConditionOneOf::Field(qdrant::FieldCondition {
                    key: "concept_name_lower".to_string(),
                    r#match: Some(qdrant::Match {
                        match_value: Some(concept_name_lower.to_string().into()),
                    }),
                    range: None,
                    geo_bounding_box: None,
                    geo_radius: None,
                    values_count: None,
                    geo_polygon: None,
                    datetime_range: None,
                    is_empty: None,
                    is_null: None,
                })),
            }])),
        )
        .await
        .unwrap()
        .result
}

async fn retrieve_point_from_db(
    client: &Qdrant,
    points: Vec<PointId>,
    collection: &str,
) -> Vec<RetrievedPoint> {
    client
        .get_points(
            GetPointsBuilder::new(collection, points)
                .with_vectors(true)
                .with_payload(true),
        )
        .await
        .unwrap()
        .result
}

async fn recommend(
    input: String,
    client: &Qdrant,
    limit: u64,
    collection_name: &str,
) -> Vec<ScoredPoint> {
    let vector = fetch_embeddings(input).await.unwrap().embedding;
    client
        .search_points(SearchPointsBuilder::new(collection_name, vector, limit).with_payload(true))
        .await
        .unwrap()
        .result
}

fn filter_and_enrich_concepts(
    concepts: Vec<Concept>,
    parameters: &Parameters,
    record_counts: &HashMap<i32, i64>,
) -> Vec<Concept> {
    concepts
        .into_iter()
        .filter(|concept| {
            // Filter by vocabulary_id
            if let Some(vocab_ids) = &parameters.vocabulary_id
                && !vocab_ids
                    .iter()
                    .any(|id| id.eq_ignore_ascii_case(&concept.vocabulary_id))
            {
                return false;
            }

            // Exclude by vocabulary_id (substring match), wil cause ICD to filter ICD-N and RxNorm to filter also RxNorm Extension
            if let Some(exclude_vocab_ids) = &parameters.exclude_vocabulary_id
                && exclude_vocab_ids.iter().any(|id| {
                    concept
                        .vocabulary_id
                        .to_lowercase()
                        .contains(&id.to_lowercase())
                })
            {
                return false;
            }

            // Filter by standard_concept
            if let Some(std_concept) = &parameters.standard_concept {
                match concept.standard_concept.as_ref() {
                    Some(sc) if sc == std_concept => {}
                    None if std_concept.is_empty() => {}
                    _ => return false,
                }
            }

            // Filter by domain_id
            if let Some(domain_ids) = &parameters.domain_id
                && !domain_ids
                    .iter()
                    .any(|id| id.eq_ignore_ascii_case(&concept.domain_id))
            {
                return false;
            }

            // Filter by concept_class_id
            if let Some(class_ids) = &parameters.concept_class_id
                && !class_ids
                    .iter()
                    .any(|id| id.eq_ignore_ascii_case(&concept.concept_class_id))
            {
                return false;
            }

            true
        })
        .map(|mut concept| {
            // Enrich with record count
            concept.record_count = record_counts.get(&concept.concept_id).copied().unwrap_or(0);
            concept
        })
        .collect()
}

#[post("/api/conceptsets/analyze")]
async fn analyze_concept_set(
    request: Json<ConceptSetValidationRequest>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    info!("Received concept set analysis request");
    let concept_set = &request.concept_set;

    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;

    let analysis_result = validation::analyze_concept_set(
        concept_set,
        &pg_client,
        Some(&state.qdrant_client),
        Some(&state.concept_index),
        Some(&state.concept_record_counts),
    )
    .await
    .unwrap_or_else(|e| {
        let mut error_result = validation::ValidationResult::new();
        error_result.add_error(format!("Database error during analysis: {}", e));
        error_result
    });

    Ok(HttpResponse::Ok().json(analysis_result.to_json()))
}
