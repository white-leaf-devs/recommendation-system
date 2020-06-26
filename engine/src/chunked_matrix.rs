// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::{
    distances::items::{slope_one, AdjCosine},
    error::ErrorKind,
};
use anyhow::Error;
use config::Config;
use controller::{eid, maped_ratings, Controller, Entity, LazyItemChunks};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub trait ChunkedMatrix<'a, C, I>
where
    C: Controller<Item = I>,
    I: Entity,
{
    fn approximate_chunk_size(&self) -> usize;
    fn optimize_chunks_size(&mut self);
    fn calculate_chunk(&mut self, i: usize, j: usize) -> Result<(), Error>;
    fn get_value(&self, id_a: &eid!(I), id_b: &eid!(I)) -> Option<f64>;
}

pub struct SimilarityMatrix<'a, C, U, I>
where
    C: Controller<User = U, Item = I>,
    U: Entity,
    I: Entity,
    eid!(U): Hash + Eq,
{
    config: &'a Config,
    controller: &'a C,

    ver_chunk_size: usize,
    hor_chunk_size: usize,

    adj_cosine: AdjCosine<eid!(U), f64>,

    ver_iter: LazyItemChunks<'a, C, I>,
    hor_iter: LazyItemChunks<'a, C, I>,

    matrix_chunk: HashMap<eid!(I), HashMap<eid!(I), f64>>,
}

impl<'a, C, U, I> SimilarityMatrix<'a, C, U, I>
where
    C: Controller<User = U, Item = I>,
    U: Entity,
    I: Entity,
    eid!(U): Hash + Eq + Default,
{
    pub fn new(controller: &'a C, config: &'a Config, m: usize, n: usize) -> Self {
        Self {
            config,
            controller,
            ver_chunk_size: m,
            hor_chunk_size: n,
            adj_cosine: AdjCosine::new(),
            ver_iter: controller.items_by_chunks(m),
            hor_iter: controller.items_by_chunks(n),
            matrix_chunk: Default::default(),
        }
    }
}

impl<'a, C, U, I> ChunkedMatrix<'a, C, I> for SimilarityMatrix<'a, C, U, I>
where
    C: Controller<User = U, Item = I>,
    U: Entity,
    I: Entity,
    eid!(U): Hash + Eq + Clone + Default,
    eid!(I): Hash + Eq + Clone,
{
    fn approximate_chunk_size(&self) -> usize {
        todo!("Implement for each controller a 'counter' method for ratings")
    }

    fn optimize_chunks_size(&mut self) {
        if !self.config.matrix.allow_chunk_optimization {
            return;
        }

        let threshold = self.config.matrix.chunk_size_threshold;
        let original_size = self.approximate_chunk_size();
        let target_size = (original_size as f64 * threshold) as usize;

        while self.approximate_chunk_size() > target_size {
            self.ver_chunk_size /= 2;
            self.hor_chunk_size /= 2;

            self.ver_iter = self.controller.items_by_chunks(self.ver_chunk_size);
            self.hor_iter = self.controller.items_by_chunks(self.hor_chunk_size);
        }
    }

    fn calculate_chunk(&mut self, i: usize, j: usize) -> Result<(), Error> {
        let ver_items = self
            .ver_iter
            .nth(i)
            .ok_or_else(|| ErrorKind::IndexOutOfBound)?;

        let hor_items = self
            .hor_iter
            .nth(j)
            .ok_or_else(|| ErrorKind::IndexOutOfBound)?;

        let ver_items_users: maped_ratings!(I => U) = self
            .controller
            .users_who_rated(&ver_items)?
            .into_iter()
            .filter(|(_, ratings)| !ratings.is_empty())
            .collect();

        let hor_items_users: maped_ratings!(I => U) = self
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

        let partial_users_chunk_size = self.config.matrix.partial_users_chunk_size;
        for partial_users_chunk in all_partial_users.chunks(partial_users_chunk_size) {
            let mean_chunk = self.controller.means_for(partial_users_chunk)?;
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

        self.matrix_chunk = matrix;

        Ok(())
    }

    fn get_value(&self, id_a: &eid!(I), id_b: &eid!(I)) -> Option<f64> {
        if let Some(row_a) = self.matrix_chunk.get(id_a) {
            let maybe_val = row_a.get(id_b);
            if let Some(val) = maybe_val {
                return Some(*val);
            }
        }

        if let Some(row_b) = self.matrix_chunk.get(id_b) {
            let maybe_val = row_b.get(id_a);
            if let Some(val) = maybe_val {
                return Some(*val);
            }
        }

        None
    }
}

pub struct DeviationMatrix<'a, C, I>
where
    C: Controller<Item = I>,
    I: Entity,
{
    config: &'a Config,
    controller: &'a C,

    ver_chunk_size: usize,
    hor_chunk_size: usize,

    ver_iter: LazyItemChunks<'a, C, I>,
    hor_iter: LazyItemChunks<'a, C, I>,

    matrix_chunk: HashMap<eid!(I), HashMap<eid!(I), f64>>,
}

impl<'a, C, I> DeviationMatrix<'a, C, I>
where
    C: Controller<Item = I>,
    I: Entity,
{
    pub fn new(controller: &'a C, config: &'a Config, m: usize, n: usize) -> Self {
        Self {
            config,
            controller,
            ver_chunk_size: m,
            hor_chunk_size: n,
            ver_iter: controller.items_by_chunks(m),
            hor_iter: controller.items_by_chunks(n),
            matrix_chunk: Default::default(),
        }
    }
}

impl<'a, C, U, I> ChunkedMatrix<'a, C, I> for DeviationMatrix<'a, C, I>
where
    C: Controller<User = U, Item = I>,
    U: Entity,
    I: Entity,
    eid!(U): Hash + Eq,
    eid!(I): Hash + Eq + Clone,
{
    fn approximate_chunk_size(&self) -> usize {
        todo!("Implement for each controller a 'counter' method for ratings")
    }

    fn optimize_chunks_size(&mut self) {
        if !self.config.matrix.allow_chunk_optimization {
            return;
        }

        let threshold = self.config.matrix.chunk_size_threshold;
        let original_size = self.approximate_chunk_size();
        let target_size = (original_size as f64 * threshold) as usize;

        while self.approximate_chunk_size() > target_size {
            self.ver_chunk_size /= 2;
            self.hor_chunk_size /= 2;

            self.ver_iter = self.controller.items_by_chunks(self.ver_chunk_size);
            self.hor_iter = self.controller.items_by_chunks(self.hor_chunk_size);
        }
    }

    fn calculate_chunk(&mut self, i: usize, j: usize) -> Result<(), Error> {
        let ver_items = self
            .ver_iter
            .nth(i)
            .ok_or_else(|| ErrorKind::IndexOutOfBound)?;

        let hor_items = self
            .hor_iter
            .nth(j)
            .ok_or_else(|| ErrorKind::IndexOutOfBound)?;

        let ver_items_users: maped_ratings!(I => U) = self
            .controller
            .users_who_rated(&ver_items)?
            .into_iter()
            .filter(|(_, ratings)| !ratings.is_empty())
            .collect();

        let hor_items_users: maped_ratings!(I => U) = self
            .controller
            .users_who_rated(&hor_items)?
            .into_iter()
            .filter(|(_, ratings)| !ratings.is_empty())
            .collect();

        let mut matrix = HashMap::new();
        for (item_a, item_a_ratings) in ver_items_users.into_iter() {
            for (item_b, item_b_ratings) in hor_items_users.iter() {
                if matrix.contains_key(item_b) {
                    continue;
                }

                if let Ok((dev, _)) = slope_one(&item_a_ratings, item_b_ratings) {
                    matrix
                        .entry(item_a.clone())
                        .or_insert_with(HashMap::new)
                        .insert(item_b.clone(), dev);
                }
            }

            matrix
                .entry(item_a.clone())
                .or_insert_with(HashMap::new)
                .insert(item_a, 0.0);
        }

        self.matrix_chunk = matrix;

        Ok(())
    }

    fn get_value(&self, id_a: &eid!(I), id_b: &eid!(I)) -> Option<f64> {
        if let Some(row_a) = self.matrix_chunk.get(id_a) {
            let maybe_val = row_a.get(id_b);
            if let Some(val) = maybe_val {
                return Some(*val);
            }
        }

        if let Some(row_b) = self.matrix_chunk.get(id_b) {
            let maybe_val = row_b.get(id_a);
            if let Some(val) = maybe_val {
                return Some(-val);
            }
        }

        None
    }
}
