use crate::db::{get_concept_by_id, get_concept_children_cached, get_concept_parents_cached};
use crate::domain::{Concept, ExpandCacheKey, ExpandConcept, ExpandResponse};
use crate::errors::PgError;
use deadpool_postgres::Client;
use log::info;
use moka::future::Cache;
use std::collections::HashMap;
use std::future::Future;
use std::time::Instant;

pub struct ConceptExpander;

impl ConceptExpander {
    #[allow(clippy::too_many_arguments)]
    pub async fn expand(
        client: &Client,
        expand_cache: &Cache<ExpandCacheKey, ExpandResponse>,
        children_cache: &Cache<i32, Vec<Concept>>,
        parents_cache: &Cache<i32, Vec<Concept>>,
        record_counts: &HashMap<i32, i64>,
        concept_id: i32,
        child_levels: Option<i32>,
        parent_levels: Option<i32>,
    ) -> Result<ExpandResponse, PgError> {
        let start = Instant::now();
        let cache_key = ExpandCacheKey::new(concept_id, child_levels, parent_levels);

        // Check cache first
        if let Some(cached_response) = expand_cache.get(&cache_key).await {
            return Ok(cached_response);
        }

        // Perform expansion
        let mut concept = get_concept_by_id(client, concept_id).await?;
        concept.record_count = record_counts.get(&concept_id).copied().unwrap_or(0);
        let mut expand_concept = ExpandConcept::from(concept);

        // Attach children if child_levels is specified and > 0
        if let Some(levels) = child_levels
            && levels > 0
        {
            expand_concept = Self::attach_children(
                client,
                children_cache,
                record_counts,
                expand_concept,
                levels,
            )
            .await?;
        }

        // If parent_levels is specified and > 0, build the parent hierarchy
        let concept_trees = if let Some(levels) = parent_levels {
            if levels > 0 {
                Self::build_parent_hierarchy(
                    client,
                    parents_cache,
                    record_counts,
                    expand_concept,
                    levels,
                )
                .await?
            } else {
                vec![expand_concept]
            }
        } else {
            vec![expand_concept]
        };

        let count = Self::count_concepts(&concept_trees);

        let response = ExpandResponse {
            concepts: concept_trees,
            childlevel: child_levels,
            parentlevel: parent_levels,
            count,
        };

        // Cache slow operations > 1000ms
        let elapsed = start.elapsed();
        if elapsed.as_millis() > 1000 {
            info!(
                "Caching concept {} expansion (took {}ms): ",
                concept_id,
                elapsed.as_millis()
            );
            expand_cache.insert(cache_key, response.clone()).await;
        }

        Ok(response)
    }

    async fn attach_children(
        client: &Client,
        children_cache: &Cache<i32, Vec<Concept>>,
        record_counts: &HashMap<i32, i64>,
        mut concept: ExpandConcept,
        max_levels: i32,
    ) -> Result<ExpandConcept, PgError> {
        concept.children = Self::get_children_recursive(
            client,
            children_cache,
            record_counts,
            vec![concept.clone()],
            1,
            max_levels,
        )
        .await?;
        if !concept.children.is_empty() {
            Ok(concept.children[0].clone())
        } else {
            Ok(concept)
        }
    }

    fn get_children_recursive<'a>(
        client: &'a Client,
        children_cache: &'a Cache<i32, Vec<Concept>>,
        record_counts: &'a HashMap<i32, i64>,
        concepts: Vec<ExpandConcept>,
        current_level: i32,
        max_levels: i32,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<ExpandConcept>, PgError>> + 'a>> {
        Box::pin(async move {
            let mut result = Vec::new();

            for mut concept in concepts {
                concept.level = Some(current_level);

                if current_level < max_levels {
                    let children =
                        get_concept_children_cached(client, children_cache, concept.concept_id)
                            .await?;
                    let mut expand_children = Vec::new();

                    for mut child in children {
                        child.record_count =
                            record_counts.get(&child.concept_id).copied().unwrap_or(0);
                        let expand_child = ExpandConcept::from(child);
                        expand_children.push(expand_child);
                    }

                    concept.children = Self::get_children_recursive(
                        client,
                        children_cache,
                        record_counts,
                        expand_children,
                        current_level + 1,
                        max_levels,
                    )
                    .await?;
                }

                result.push(concept);
            }

            Ok(result)
        })
    }

    async fn build_parent_hierarchy(
        client: &Client,
        parents_cache: &Cache<i32, Vec<Concept>>,
        record_counts: &HashMap<i32, i64>,
        concept: ExpandConcept,
        max_levels: i32,
    ) -> Result<Vec<ExpandConcept>, PgError> {
        Self::get_parents_recursive(
            client,
            parents_cache,
            record_counts,
            concept,
            1,
            max_levels,
            true,
        )
        .await
    }

    fn get_parents_recursive<'a>(
        client: &'a Client,
        parents_cache: &'a Cache<i32, Vec<Concept>>,
        record_counts: &'a HashMap<i32, i64>,
        concept: ExpandConcept,
        current_level: i32,
        max_levels: i32,
        append_child: bool,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<ExpandConcept>, PgError>> + 'a>> {
        Box::pin(async move {
            if max_levels == -1 || current_level < max_levels {
                let parent_concepts =
                    get_concept_parents_cached(client, parents_cache, concept.concept_id).await?;
                let parents = parent_concepts
                    .into_iter()
                    .filter(|parent| parent.vocabulary_id == concept.vocabulary_id)
                    .collect::<Vec<_>>();

                if !parents.is_empty() {
                    let mut parent_expand_concepts = Vec::new();

                    for mut parent in parents {
                        parent.record_count =
                            record_counts.get(&parent.concept_id).copied().unwrap_or(0);
                        let mut parent_expand = ExpandConcept::from(parent);
                        if append_child {
                            parent_expand.children.push(concept.clone());
                            parent_expand.level = Some(current_level);
                        }
                        parent_expand_concepts.push(parent_expand);
                    }

                    let mut top_concepts = Vec::new();
                    for parent in parent_expand_concepts {
                        let tops = Self::get_parents_recursive(
                            client,
                            parents_cache,
                            record_counts,
                            parent,
                            current_level + 1,
                            max_levels,
                            append_child,
                        )
                        .await?;
                        top_concepts.extend(tops);
                    }

                    if !top_concepts.is_empty() {
                        return Ok(top_concepts);
                    }
                }
            }

            Ok(vec![concept])
        })
    }

    fn count_concepts(concept_trees: &[ExpandConcept]) -> i32 {
        concept_trees
            .iter()
            .map(Self::count_concept_recursive)
            .sum()
    }

    fn count_concept_recursive(concept: &ExpandConcept) -> i32 {
        if concept.children.is_empty() {
            1
        } else {
            1 + concept
                .children
                .iter()
                .map(Self::count_concept_recursive)
                .sum::<i32>()
        }
    }
}
