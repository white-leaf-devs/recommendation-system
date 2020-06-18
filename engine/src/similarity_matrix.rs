use crate::{distances::items::AdjCosine, error::ErrorKind};
use anyhow::Error;
use config::Config;
use controller::{Controller, Entity, LazyItemChunks, MapedRatings};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub struct SimilarityMatrix<'a, C, User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId>,
    UserId: Hash + Eq,
    C: Controller<User, UserId, Item, ItemId>,
{
    config: &'a Config,
    controller: &'a C,

    ver_chunk_size: usize,
    hor_chunk_size: usize,

    adj_cosine: AdjCosine<UserId, f64>,

    ver_iter: LazyItemChunks<'a, User, UserId, Item, ItemId>,
    hor_iter: LazyItemChunks<'a, User, UserId, Item, ItemId>,
}

impl<'a, C, User, UserId, Item, ItemId> SimilarityMatrix<'a, C, User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId>,
    UserId: Hash + Eq,
    C: Controller<User, UserId, Item, ItemId>,
{
    pub fn new(controller: &'a C, config: &'a Config, m: usize, n: usize) -> Self
    where
        UserId: Default,
    {
        Self {
            config,
            controller,
            ver_chunk_size: m,
            hor_chunk_size: n,
            adj_cosine: AdjCosine::new(),
            ver_iter: controller.items_by_chunks(m),
            hor_iter: controller.items_by_chunks(n),
        }
    }

    fn approximate_chunk_size(&self) -> usize {
        todo!("Implement for each controller a 'counter' method for ratings")
    }

    pub fn optimize_chunks(&mut self) {
        if !self.config.sim_matrix.allow_chunk_optimization {
            return;
        }

        let threshold = self.config.sim_matrix.chunk_size_threshold;
        let original_size = self.approximate_chunk_size();
        let target_size = (original_size as f64 * threshold) as usize;

        while self.approximate_chunk_size() > target_size {
            self.ver_chunk_size /= 2;
            self.hor_chunk_size /= 2;

            self.ver_iter = self.controller.items_by_chunks(self.ver_chunk_size);
            self.hor_iter = self.controller.items_by_chunks(self.hor_chunk_size);
        }
    }

    pub fn get_chunk(
        &mut self,
        i: usize,
        j: usize,
    ) -> Result<HashMap<ItemId, HashMap<ItemId, f64>>, Error>
    where
        UserId: Hash + Eq + Clone + Default,
        ItemId: Hash + Eq + Clone,
    {
        let ver_items = self
            .ver_iter
            .nth(i)
            .ok_or_else(|| ErrorKind::IndexOutOfBound)?;

        let hor_items = self
            .hor_iter
            .nth(j)
            .ok_or_else(|| ErrorKind::IndexOutOfBound)?;

        let ver_items_users: MapedRatings<ItemId, UserId> = self
            .controller
            .users_who_rated(&ver_items)?
            .into_iter()
            .filter(|(_, ratings)| !ratings.is_empty())
            .collect();

        let hor_items_users: MapedRatings<ItemId, UserId> = self
            .controller
            .users_who_rated(&hor_items)?
            .into_iter()
            .filter(|(_, ratings)| !ratings.is_empty())
            .collect();

        let all_users_iter = ver_items_users.values().chain(hor_items_users.values());
        let mut all_users = HashSet::new();

        for users in all_users_iter {
            for user in users.keys() {
                all_users.insert(user.clone());
            }
        }

        // Shrink some means by their usage frequency
        self.adj_cosine.shrink_means();

        // Collect all the users that doesn't have a calculated mean
        let all_users: Vec<_> = all_users
            .into_iter()
            .filter(|user_id| !self.adj_cosine.has_mean_for(user_id))
            .collect();
        let all_partial_users = self.controller.create_partial_users(&all_users)?;

        let partial_users_chunk_size = self.config.sim_matrix.partial_users_chunk_size;
        for partial_users_chunk in all_partial_users.chunks(partial_users_chunk_size) {
            let mean_chunk = self.controller.get_means(partial_users_chunk);
            self.adj_cosine.add_new_means(&mean_chunk);
        }

        let mut matrix = HashMap::new();
        for (item_a, item_a_ratings) in ver_items_users.into_iter() {
            for (item_b, item_b_ratings) in hor_items_users.iter() {
                if matrix.contains_key(item_b) {
                    continue;
                }

                if let Ok(similarity) = self.adj_cosine.calculate(&item_a_ratings, item_b_ratings) {
                    matrix
                        .entry(item_a.clone())
                        .or_insert_with(HashMap::new)
                        .insert(item_b.clone(), similarity);
                }
            }

            matrix
                .entry(item_a.clone())
                .or_insert_with(HashMap::new)
                .insert(item_a, 1.0);
        }

        Ok(matrix)
    }
}
