// Copyright (C) 2020 Kevin Del Castillo Ramírez
//
// This file is part of recommend.
//
// recommend is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// recommend is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with recommend.  If not, see <http://www.gnu.org/licenses/>.

use crate::{record::Record, Distance};
use num_traits::real::Real;
use std::{
    borrow::Borrow,
    collections::hash_map::RandomState,
    collections::HashMap,
    hash::{BuildHasher, Hash, Hasher},
    ops::{AddAssign, Mul, MulAssign, Sub},
};

#[derive(Debug, Clone, Default)]
pub struct Table<I, K, V, S = RandomState>
where
    I: Hash + Eq,
    K: Hash + Eq,
    S: BuildHasher,
{
    hash_builder: S,
    keys: HashMap<u64, K>,
    records: HashMap<I, Record<V, S>>,
}

pub(crate) fn make_hash<K>(hash_builder: &impl BuildHasher, val: &K) -> u64
where
    K: Hash,
{
    let mut state = hash_builder.build_hasher();
    val.hash(&mut state);
    state.finish()
}

impl<I, K, V, S> Table<I, K, V, S>
where
    I: Hash + Eq + Default,
    K: Hash + Eq + Default + Clone,
    V: Default,
    S: BuildHasher + Default,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_keys(keys: &[K]) -> Self {
        let hash_builder = S::default();

        let mut with_hashes = Vec::new();
        for key in keys {
            let hash = make_hash(&hash_builder, &key);
            with_hashes.push((hash, key.clone()));
        }

        Self {
            hash_builder,
            keys: with_hashes.into_iter().collect(),
            ..Default::default()
        }
    }

    pub fn hash_key(&self, key: &K) -> u64 {
        make_hash(&self.hash_builder, key)
    }
}

impl<I, K, V, S> Table<I, K, V, S>
where
    I: Hash + Eq,
    K: Hash + Eq,
    S: BuildHasher,
{
    pub fn insert(&mut self, key: I, record: Record<V, S>) {
        self.records.insert(key, record);
    }

    pub fn record<Q: ?Sized>(&self, key: &Q) -> Option<&Record<V, S>>
    where
        I: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.records.get(key)
    }

    pub fn record_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut Record<V, S>>
    where
        I: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.records.get_mut(key)
    }
}

impl<'a, I, K, V, S> Table<I, K, V, S>
where
    I: Hash + Eq,
    K: Hash + Eq,
    S: BuildHasher,
    V: Real + AddAssign + MulAssign + 'a,
    &'a V: Sub<Output = V> + Mul<Output = V>,
{
    pub fn distance_between<P: ?Sized, Q: ?Sized>(
        &'a self,
        a: &P,
        b: &Q,
        method: Distance,
    ) -> Option<V>
    where
        I: Borrow<Q>,
        I: Borrow<P>,
        P: Hash + Eq,
        Q: Hash + Eq,
    {
        let a = a.borrow();
        let b = b.borrow();

        match method {
            Distance::Manhattan => self.record(a)?.manhattan_distance(self.record(b)?),
            Distance::Euclidean => self.record(a)?.euclidean_distance(self.record(b)?),
            Distance::Minkowski(p) => self.record(a)?.minkowski_distance(self.record(b)?, p),
        }
    }
}
