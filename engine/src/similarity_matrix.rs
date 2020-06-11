use crate::distances::items::{adjusted_cosine_means, fast_adjusted_cosine};
use controller::{Controller, Entity, LazyItemChunks};
use std::collections::{HashMap, HashSet};

pub struct SimilarityMatrix<'a, C, U, I>
where
    C: Controller<U, I>,
{
    controller: &'a C,

    ver_chunk_size: usize,
    hor_chunk_size: usize,
    threshold: usize,

    ver_iter: LazyItemChunks<'a, U, I>,
    hor_iter: LazyItemChunks<'a, U, I>,
}

impl<'a, C, U, I> SimilarityMatrix<'a, C, U, I>
where
    C: Controller<U, I>,
{
    pub fn new(controller: &'a C, m: usize, n: usize, threshold: usize) -> Self {
        Self {
            controller,
            ver_chunk_size: m,
            hor_chunk_size: n,
            threshold,
            ver_iter: controller.items_by_chunks(m),
            hor_iter: controller.items_by_chunks(n),
        }
    }

    fn approximate_chunk_size(&self) -> usize {
        todo!("Implement for each controller a 'counter' method")
    }

    pub fn optimize_chunks(&mut self) {
        while self.approximate_chunk_size() > self.threshold {
            self.ver_chunk_size /= 2;
            self.hor_chunk_size /= 2;

            self.ver_iter = self.controller.items_by_chunks(self.ver_chunk_size);
            self.hor_iter = self.controller.items_by_chunks(self.hor_chunk_size);
        }
    }

    pub fn get_chunk(&mut self, i: usize, j: usize) -> Option<HashMap<String, HashMap<String, f64>>>
    where
        I: Entity,
    {
        let ver_items = self.ver_iter.nth(i)?;
        let hor_items = self.hor_iter.nth(j)?;

        let ver_items_users = self.controller.users_who_rated(&ver_items).ok()?;
        let hor_items_users = self.controller.users_who_rated(&hor_items).ok()?;

        let all_users = ver_items_users
            .into_iter()
            .fold(HashSet::new(), |whole_set, (_, users)| {
                whole_set.union(&users).cloned().collect()
            });

        let all_users = hor_items_users
            .into_iter()
            .fold(all_users, |whole_set, (_, users)| {
                whole_set.union(&users).cloned().collect()
            });

        let all_users: Vec<_> = all_users.into_iter().collect();
        let all_partial_users = self.controller.create_partial_users(&all_users).ok()?;

        let maped_ratings = self.controller.maped_ratings_by(&all_partial_users).ok()?;
        let means = adjusted_cosine_means(&maped_ratings);

        let mut matrix = HashMap::new();
        for item_a in &ver_items {
            for item_b in &hor_items {
                let item_a = item_a.get_id();
                let item_b = item_b.get_id();

                if item_a == item_b {
                    matrix
                        .entry(item_a)
                        .or_insert_with(HashMap::new)
                        .insert(item_b, 1.0);
                    continue;
                }

                if let Some(similarity) =
                    fast_adjusted_cosine(&means, &maped_ratings, &item_a, &item_b)
                {
                    matrix
                        .entry(item_a)
                        .or_insert_with(HashMap::new)
                        .insert(item_b, similarity);
                }
            }
        }

        Some(matrix)
    }
}
