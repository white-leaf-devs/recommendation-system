#![allow(clippy::implicit_hasher)]

use num_traits::real::Real;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::{AddAssign, Mul, MulAssign, Sub},
};

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Manhattan,
    Euclidean,
    Minkowski(usize),
    JaccardIndex,
    JaccardDistance,
    CosineSimilarity,
    PearsonCorrelation,
}

pub fn distance<K, V>(lhs: &HashMap<K, V>, rhs: &HashMap<K, V>, method: Method) -> Option<V>
where
    K: Hash + Eq,
    V: Real + AddAssign + Sub + Mul + MulAssign,
{
    match method {
        Method::Manhattan => manhattan_distance(lhs, rhs),
        Method::Euclidean => euclidean_distance(lhs, rhs),
        Method::Minkowski(p) => minkowski_distance(lhs, rhs, p),
        Method::JaccardIndex => jaccard_index(lhs, rhs),
        Method::JaccardDistance => jaccard_distance(lhs, rhs),
        Method::CosineSimilarity => cosine_similarity(lhs, rhs),
        Method::PearsonCorrelation => pearson_correlation(lhs, rhs),
    }
}

pub fn manhattan_distance<K, V>(lhs: &HashMap<K, V>, rhs: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Real + AddAssign + Sub,
{
    let mut dist = None;

    for (key, x) in lhs {
        if let Some(y) = rhs.get(key) {
            *dist.get_or_insert_with(V::zero) += (*y - *x).abs();
        }
    }

    dist
}

pub fn euclidean_distance<K, V>(lhs: &HashMap<K, V>, rhs: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Real + AddAssign + Sub,
{
    let mut dist = None;

    for (key, x) in lhs {
        if let Some(y) = rhs.get(key) {
            *dist.get_or_insert_with(V::zero) += (*y - *x).powi(2);
        }
    }

    dist.map(V::sqrt)
}

pub fn minkowski_distance<K, V>(lhs: &HashMap<K, V>, rhs: &HashMap<K, V>, p: usize) -> Option<V>
where
    K: Hash + Eq,
    V: Real + AddAssign + Sub,
{
    let mut dist = None;

    for (key, x) in lhs {
        if let Some(y) = rhs.get(key) {
            *dist.get_or_insert_with(V::zero) += (*y - *x).abs().powi(p as i32);
        }
    }

    V::from(p)
        .map(|p| dist.map(|v| v.powf(V::one() / p)))
        .flatten()
}

pub fn jaccard_index<K, V>(lhs: &HashMap<K, V>, rhs: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Real + AddAssign + Sub,
{
    let lhs_keys: HashSet<_> = lhs.keys().collect();
    let rhs_keys: HashSet<_> = rhs.keys().collect();

    let inter = lhs_keys.intersection(&rhs_keys).count();
    let union = lhs_keys.union(&rhs_keys).count();

    if union == 0 {
        None
    } else {
        Some(V::from(inter)? / V::from(union)?)
    }
}

pub fn jaccard_distance<K, V>(lhs: &HashMap<K, V>, rhs: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Real + AddAssign + Sub,
{
    Some(V::one() - jaccard_index(lhs, rhs)?)
}

pub fn cosine_similarity<K, V>(lhs: &HashMap<K, V>, rhs: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Real + AddAssign + Sub + Mul + MulAssign,
{
    let mut a_norm = None;
    let mut b_norm = None;
    let mut dot_prod = None;

    for (key, x) in lhs {
        if let Some(y) = rhs.get(key) {
            *a_norm.get_or_insert_with(V::zero) += x.powi(2);
            *b_norm.get_or_insert_with(V::zero) += y.powi(2);
            *dot_prod.get_or_insert_with(V::one) *= (*x) * (*y);
        }
    }

    let norm = (a_norm? * b_norm?).sqrt();

    Some(dot_prod? / norm)
}

pub fn pearson_correlation<K, V>(lhs: &HashMap<K, V>, rhs: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Real + AddAssign + Sub + Mul + MulAssign,
{
    let mut mean_x = None;
    let mut mean_y = None;
    let mut total = 0;

    for (key, x) in lhs {
        if let Some(y) = rhs.get(key) {
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

    for (key, x) in lhs {
        if let Some(y) = rhs.get(key) {
            *cov.get_or_insert_with(V::one) *= (*x - mean_x) * (*y - mean_y);
            *std_dev_a.get_or_insert_with(V::zero) += (*x - mean_x).powi(2);
            *std_dev_b.get_or_insert_with(V::zero) += (*y - mean_y).powi(2);
        }
    }

    let std_dev = (std_dev_a? * std_dev_b?).sqrt();

    Some(cov? / std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::*;
    use common_macros::hash_map;

    #[test]
    fn manhattan_distance_test() {
        let a = hash_map! {
            0 => 1.,
            2 => 2.,
        };

        let b = hash_map! {
            0 => 1.,
            1 => 3.,
            2 => 3.,
        };

        let d = manhattan_distance(&a, &b);

        assert_approx_eq!(1., d.unwrap());
    }

    #[test]
    fn euclidean_distance_test() {
        let a = hash_map! {
            0 => 0.,
            2 => 0.
        };

        let b = hash_map! {
            0 => 2.,
            1 => 1.,
            2 => 2.
        };

        let d = euclidean_distance(&a, &b);

        assert_approx_eq!(8f64.sqrt(), d.unwrap());
    }

    #[test]
    fn minkowski3_distance_test() {
        let a = hash_map! {
            0 => 0.,
            2 => 0.,
        };

        let b = hash_map! {
            0 => 2.,
            1 => 1.,
            2 => 2.,
        };

        let d = minkowski_distance(&a, &b, 3);

        assert_approx_eq!(16f64.powf(1. / 3.), d.unwrap());
    }
}
