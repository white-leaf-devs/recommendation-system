use crate::distances::items::{adjusted_cosine_means, fast_adjusted_cosine, AdjCosine};
use controller::{Controller, Entity, ItemsUsers, LazyItemChunks, MapedRatings};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

pub struct SimilarityMatrix<'a, C, User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId>,
    C: Controller<User, UserId, Item, ItemId>,
{
    controller: &'a C,

    ver_chunk_size: usize,
    hor_chunk_size: usize,
    threshold: usize,

    ver_iter: LazyItemChunks<'a, User, UserId, Item, ItemId>,
    hor_iter: LazyItemChunks<'a, User, UserId, Item, ItemId>,
}

impl<'a, C, User, UserId, Item, ItemId> SimilarityMatrix<'a, C, User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId>,
    C: Controller<User, UserId, Item, ItemId>,
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
        todo!("Implement for each controller a 'counter' method for ratings")
    }

    pub fn optimize_chunks(&mut self) {
        while self.approximate_chunk_size() > self.threshold {
            self.ver_chunk_size /= 2;
            self.hor_chunk_size /= 2;

            self.ver_iter = self.controller.items_by_chunks(self.ver_chunk_size);
            self.hor_iter = self.controller.items_by_chunks(self.hor_chunk_size);
        }
    }

    pub fn get_chunk(&mut self, i: usize, j: usize) -> Option<HashMap<ItemId, HashMap<ItemId, f64>>>
    where
        UserId: Hash + Eq + Clone + Default,
        ItemId: Hash + Eq + Clone,
    {
        let ver_items = self.ver_iter.nth(i)?;
        let hor_items = self.hor_iter.nth(j)?;

        let ver_items_users: MapedRatings<ItemId, UserId> = self
            .controller
            .users_who_rated(&ver_items)
            .ok()?
            .into_iter()
            .filter(|(_, ratings)| !ratings.is_empty())
            .collect();

        let hor_items_users: MapedRatings<ItemId, UserId> = self
            .controller
            .users_who_rated(&hor_items)
            .ok()?
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

        let all_users: Vec<_> = all_users.into_iter().collect();
        let all_partial_users = self.controller.create_partial_users(&all_users).ok()?;
        let maped_ratings = self.controller.maped_ratings_by(&all_partial_users).ok()?;

        let mut adj_cosine = AdjCosine::new();
        adj_cosine.update_means(&maped_ratings);

        let mut matrix = HashMap::new();
        for (item_a, users_a) in ver_items_users.into_iter() {
            for (item_b, users_b) in hor_items_users.iter() {
                if matrix.contains_key(item_b) {
                    continue;
                }

                if let Ok(similarity) = adj_cosine.calculate(&users_a, users_b) {
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

        Some(matrix)
    }
}
