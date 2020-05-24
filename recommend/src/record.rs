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

use num_traits::real::Real;
use std::{
    collections::{hash_map::RandomState, HashMap, HashSet},
    default::Default,
    fmt::Debug,
    hash::BuildHasher,
    ops::{AddAssign, Mul, MulAssign, Sub},
};

#[derive(Debug, Clone, Default)]
pub struct Record<V, S = RandomState>
where
    S: BuildHasher,
{
    values: HashMap<u64, V, S>,
}

impl<V> From<HashMap<u64, V>> for Record<V> {
    fn from(map: HashMap<u64, V>) -> Self {
        Self { values: map }
    }
}

impl<V, S> Record<V, S>
where
    V: Default,
    S: BuildHasher + Default,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<V, S> Record<V, S>
where
    S: BuildHasher,
{
    pub fn values(&self) -> &HashMap<u64, V, S> {
        &self.values
    }

    pub fn values_mut(&mut self) -> &mut HashMap<u64, V, S> {
        &mut self.values
    }
}

impl<V, S> Record<V, S>
where
    S: BuildHasher,
    V: Real + AddAssign + MulAssign + Sub + Mul,
{
    pub fn manhattan_distance(&self, rhs: &Self) -> Option<V> {
        let mut dist = None;

        for (key, x) in &self.values {
            if let Some(y) = rhs.values.get(key) {
                *dist.get_or_insert_with(V::zero) += (*y - *x).abs();
            }
        }

        dist
    }

    pub fn euclidean_distance(&self, rhs: &Self) -> Option<V> {
        let mut dist = None;

        for (key, x) in &self.values {
            if let Some(y) = rhs.values.get(key) {
                *dist.get_or_insert_with(V::zero) += (*y - *x).powi(2);
            }
        }

        dist.map(V::sqrt)
    }

    pub fn minkowski_distance(&self, rhs: &Self, p: usize) -> Option<V> {
        let mut dist = None;

        for (key, x) in &self.values {
            if let Some(y) = rhs.values.get(key) {
                *dist.get_or_insert_with(V::zero) += (*y - *x).abs().powi(p as i32);
            }
        }

        V::from(p)
            .map(|p| dist.map(|v| v.powf(V::one() / p)))
            .flatten()
    }

    pub fn jaccard_index(&self, rhs: &Self) -> Option<V> {
        let lhs_keys: HashSet<_> = self.values.keys().collect();
        let rhs_keys: HashSet<_> = rhs.values.keys().collect();

        let inter = lhs_keys.intersection(&rhs_keys).count();
        let union = lhs_keys.union(&rhs_keys).count();

        if union == 0 {
            None
        } else {
            Some(V::from(inter)? / V::from(union)?)
        }
    }

    pub fn jaccard_distance(&self, rhs: &Self) -> Option<V> {
        Some(V::one() - self.jaccard_index(rhs)?)
    }

    pub fn cosine_similarity(&self, rhs: &Self) -> Option<V> {
        let mut a_norm = None;
        let mut b_norm = None;
        let mut dot_prod = None;

        for (key, x) in &self.values {
            if let Some(y) = rhs.values.get(key) {
                *a_norm.get_or_insert_with(V::zero) += x.powi(2);
                *b_norm.get_or_insert_with(V::zero) += y.powi(2);
                *dot_prod.get_or_insert_with(V::one) *= (*x) * (*y);
            }
        }

        let norm = (a_norm? * b_norm?).sqrt();

        Some(dot_prod? / norm)
    }

    pub fn pearson_correlation(&self, rhs: &Self) -> Option<V> {
        let mut mean_x = None;
        let mut mean_y = None;
        let mut total = 0;

        for (key, x) in &self.values {
            if let Some(y) = rhs.values.get(key) {
                *mean_x.get_or_insert_with(V::zero) += *x;
                *mean_y.get_or_insert_with(V::zero) += *y;
                total += 1;
            }
        }

        let mean_x = mean_x? / V::from(total)?;
        let mean_y = mean_y? / V::from(total)?;

        let mut cov = None;
        let mut std_dev_a = None;
        let mut std_dev_b = None;

        for (key, x) in &self.values {
            if let Some(y) = rhs.values.get(key) {
                *cov.get_or_insert_with(V::one) *= (*x - mean_x) * (*y - mean_y);
                *std_dev_a.get_or_insert_with(V::zero) += (*x - mean_x).powi(2);
                *std_dev_b.get_or_insert_with(V::zero) += (*y - mean_y).powi(2);
            }
        }

        let std_dev = (std_dev_a? * std_dev_b?).sqrt();

        Some(cov? / std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::*;

    #[test]
    fn manhattan_distance() {
        let a = Record {
            values: [(0, 1.), (2, 2.)]
                .iter()
                .cloned()
                .collect::<HashMap<u64, f64>>(),
        };

        let b = Record {
            values: [(0, 1.), (1, 3.), (2, 3.)].iter().cloned().collect(),
        };

        let d = b.manhattan_distance(&a);

        assert_approx_eq!(1., d.unwrap());
    }

    #[test]
    fn euclidean_distance() {
        let a = Record {
            values: [(0, 0.), (2, 0.)]
                .iter()
                .cloned()
                .collect::<HashMap<u64, f64>>(),
        };

        let b = Record {
            values: [(0, 2.), (1, 1.), (2, 2.)].iter().cloned().collect(),
        };

        let d = b.euclidean_distance(&a);

        assert_approx_eq!(8f64.sqrt(), d.unwrap());
    }

    #[test]
    fn minkowski3_distance() {
        let a = Record {
            values: [(0, 0.), (2, 0.)]
                .iter()
                .cloned()
                .collect::<HashMap<u64, f64>>(),
        };

        let b = Record {
            values: [(0, 2.), (1, 1.), (2, 2.)].iter().cloned().collect(),
        };

        let d = b.minkowski_distance(&a, 3);

        assert_approx_eq!(16f64.powf(1. / 3.), d.unwrap());
    }
}
