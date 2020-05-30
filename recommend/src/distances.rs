#![allow(clippy::implicit_hasher)]

use num_traits::float::Float;
use std::{
    collections::{hash_map::Iter as MapIter, HashMap, HashSet},
    hash::Hash,
    ops::{AddAssign, Mul, MulAssign, Sub},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Method {
    Manhattan,
    Euclidean,
    Minkowski(usize),
    JaccardIndex,
    JaccardDistance,
    CosineSimilarity,
    PearsonCorrelation,
    PearsonApproximation,
}

impl Method {
    pub fn is_similarity(&self) -> bool {
        match self {
            Method::Manhattan
            | Method::Euclidean
            | Method::Minkowski(_)
            | Method::JaccardDistance => false,

            Method::JaccardIndex
            | Method::CosineSimilarity
            | Method::PearsonCorrelation
            | Method::PearsonApproximation => true,
        }
    }
}

#[derive(Debug)]
pub struct CommonKeyIterator<'a, K, V>
where
    K: Hash + Eq,
{
    shortest: MapIter<'a, K, V>,
    longest: &'a HashMap<K, V>,
}

impl<'a, K, V> CommonKeyIterator<'a, K, V>
where
    K: Hash + Eq,
{
    // Creating a common key iterator is kinda interesting since it'll
    // decide which map is going to be iterated based on it's length.
    // Basically if one of them is empty, it'll choose it as the main
    // iterator, therefore ending the iteration early.
    pub fn new(a: &'a HashMap<K, V>, b: &'a HashMap<K, V>) -> Self {
        let (shortest, longest) = if a.len() > b.len() { (b, a) } else { (a, b) };

        Self {
            shortest: shortest.iter(),
            longest,
        }
    }
}

impl<'a, K, V> Iterator for CommonKeyIterator<'a, K, V>
where
    K: Hash + Eq,
{
    type Item = (&'a V, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let mut a_val = self.shortest.next()?;

        loop {
            if let Some(b_val) = self.longest.get(a_val.0) {
                break Some((a_val.1, b_val));
            } else {
                a_val = self.shortest.next()?;
            }
        }
    }
}

pub fn distance<K, V>(a: &HashMap<K, V>, b: &HashMap<K, V>, method: Method) -> Option<V>
where
    K: Hash + Eq,
    V: Float + AddAssign + Sub + Mul + MulAssign,
{
    match method {
        Method::Manhattan => manhattan_distance(a, b),
        Method::Euclidean => euclidean_distance(a, b),
        Method::Minkowski(p) => minkowski_distance(a, b, p),
        Method::JaccardIndex => jaccard_index(a, b),
        Method::JaccardDistance => jaccard_distance(a, b),
        Method::CosineSimilarity => cosine_similarity(a, b),
        Method::PearsonCorrelation => pearson_correlation(a, b),
        Method::PearsonApproximation => pearson_approximation(a, b),
    }
}

pub fn manhattan_distance<K, V>(a: &HashMap<K, V>, b: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Float + AddAssign + Sub,
{
    let mut dist = None;
    for (x, y) in CommonKeyIterator::new(a, b) {
        *dist.get_or_insert_with(V::zero) += (*y - *x).abs();
    }

    dist
}

pub fn euclidean_distance<K, V>(a: &HashMap<K, V>, b: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Float + AddAssign + Sub,
{
    let mut dist = None;
    for (x, y) in CommonKeyIterator::new(a, b) {
        *dist.get_or_insert_with(V::zero) += (*y - *x).powi(2);
    }

    dist.map(V::sqrt)
}

pub fn minkowski_distance<K, V>(a: &HashMap<K, V>, b: &HashMap<K, V>, p: usize) -> Option<V>
where
    K: Hash + Eq,
    V: Float + AddAssign + Sub,
{
    if p == 0 {
        panic!("Received p = 0 for minkowski distance!");
    }

    let mut dist = None;
    for (x, y) in CommonKeyIterator::new(a, b) {
        *dist.get_or_insert_with(V::zero) += (*y - *x).abs().powi(p as i32);
    }

    let exp = V::one() / V::from(p)?;
    dist.map(|dist| dist.powf(exp))
}

pub fn jaccard_index<K, V>(a: &HashMap<K, V>, b: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Float + AddAssign + Sub,
{
    match (a.is_empty(), b.is_empty()) {
        // Both are empty, cannot compute the index
        (true, true) => None,

        // One of them is empty, the result is zero
        (true, _) | (_, true) => Some(V::zero()),

        // Both have at least one element, proceed
        _ => {
            let a_keys: HashSet<_> = a.keys().collect();
            let b_keys: HashSet<_> = b.keys().collect();

            let union = a_keys.union(&b_keys).count();
            let inter = a_keys.intersection(&b_keys).count();

            Some(V::from(inter)? / V::from(union)?)
        }
    }
}

pub fn jaccard_distance<K, V>(a: &HashMap<K, V>, b: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Float + AddAssign + Sub,
{
    Some(V::one() - jaccard_index(a, b)?)
}

pub fn cosine_similarity<K, V>(a: &HashMap<K, V>, b: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Float + AddAssign + Sub + Mul,
{
    let mut a_norm = None;
    let mut b_norm = None;
    let mut dot_prod = None;

    for (x, y) in CommonKeyIterator::new(a, b) {
        *a_norm.get_or_insert_with(V::zero) += x.powi(2);
        *b_norm.get_or_insert_with(V::zero) += y.powi(2);
        *dot_prod.get_or_insert_with(V::zero) += (*x) * (*y);
    }

    let dot_prod = dot_prod?;
    let a_norm = a_norm?;
    let b_norm = b_norm?;

    let cos_sim = dot_prod / (a_norm.sqrt() * b_norm.sqrt());
    if cos_sim.is_nan() || cos_sim.is_infinite() {
        None
    } else {
        Some(cos_sim)
    }
}

pub fn pearson_correlation<K, V>(a: &HashMap<K, V>, b: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Float + AddAssign + Sub + Mul,
{
    let mut mean_x = None;
    let mut mean_y = None;
    let mut n = 0;

    for (x, y) in CommonKeyIterator::new(a, b) {
        *mean_x.get_or_insert_with(V::zero) += *x;
        *mean_y.get_or_insert_with(V::zero) += *y;
        n += 1;
    }

    let n = V::from(n)?;
    let mean_x = mean_x? / n;
    let mean_y = mean_y? / n;

    let mut cov = None;
    let mut std_dev_a = None;
    let mut std_dev_b = None;

    for (x, y) in CommonKeyIterator::new(a, b) {
        *cov.get_or_insert_with(V::zero) += (*x - mean_x) * (*y - mean_y);
        *std_dev_a.get_or_insert_with(V::zero) += (*x - mean_x).powi(2);
        *std_dev_b.get_or_insert_with(V::zero) += (*y - mean_y).powi(2);
    }

    let cov = cov?;
    let std_dev = std_dev_a?.sqrt() * std_dev_b?.sqrt();

    let pearson = cov / std_dev;
    if pearson.is_nan() || pearson.is_infinite() {
        None
    } else {
        Some(pearson)
    }
}

fn pearson_approximation<K, V>(a: &HashMap<K, V>, b: &HashMap<K, V>) -> Option<V>
where
    K: Hash + Eq,
    V: Float + AddAssign + Sub + Mul,
{
    let mut sum_x = None;
    let mut sum_y = None;
    let mut sum_x_sq = None;
    let mut sum_y_sq = None;
    let mut dot_prod = None;
    let mut n = 0;

    for (x, y) in CommonKeyIterator::new(a, b) {
        *sum_x.get_or_insert_with(V::zero) += *x;
        *sum_y.get_or_insert_with(V::zero) += *y;
        *sum_x_sq.get_or_insert_with(V::zero) += x.powi(2);
        *sum_y_sq.get_or_insert_with(V::zero) += y.powi(2);
        *dot_prod.get_or_insert_with(V::zero) += (*x) * (*y);
        n += 1;
    }

    let n = V::from(n)?;
    let num = dot_prod? - (sum_x? * sum_y?) / n;

    let dem_x = sum_x_sq? - sum_x?.powi(2) / n;
    let dem_y = sum_y_sq? - sum_y?.powi(2) / n;
    let dem = dem_x.sqrt() * dem_y.sqrt();

    let pearson = num / dem;
    if pearson.is_nan() || pearson.is_infinite() {
        None
    } else {
        Some(pearson)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::*;
    use common_macros::hash_map;

    #[test]
    fn common_key_iterator() {
        let a = hash_map! {
            0 => 0.,
            2 => 0.,
            3 => 0.,
            5 => 0.,
        };

        let b = hash_map! {
            0 => 2.,
            1 => 1.,
            2 => 2.,
            5 => 2.,
        };

        let mut iter = CommonKeyIterator::new(&a, &b);

        assert_eq!(iter.next(), Some((&0., &2.)));
        assert_eq!(iter.next(), Some((&0., &2.)));
        assert_eq!(iter.next(), Some((&0., &2.)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn invalid_distances_should_be_none() {
        let a = hash_map! {
            0 => 1.,
            2 => 2.,
            3 => 2.,
        };

        let b = hash_map! {
            4 => 1.,
            5 => 2.,
            6 => 2.,
        };

        assert!(manhattan_distance(&a, &b).is_none());
        assert!(euclidean_distance(&a, &b).is_none());
        assert!(minkowski_distance(&a, &b, 1).is_none());
        assert!(minkowski_distance(&a, &b, 2).is_none());
        assert!(minkowski_distance(&a, &b, 3).is_none());
        assert!(cosine_similarity(&a, &b).is_none());
        assert!(pearson_correlation(&a, &b).is_none());
        assert!(pearson_approximation(&a, &b).is_none());
    }

    #[test]
    fn manhattan_distance_ok() {
        let a = hash_map! {
            0 => 1.,
            2 => 2.,
            3 => 2.,
        };

        let b = hash_map! {
            0 => 1.,
            1 => 3.,
            2 => 3.,
            3 => 4.,
        };

        assert_approx_eq!(3., manhattan_distance(&a, &b).unwrap());
    }

    #[test]
    fn euclidean_distance_ok() {
        let a = hash_map! {
            0 => 0.,
            2 => 1.,
            3 => 2.,
        };

        let b = hash_map! {
            0 => 2.,
            1 => 1.,
            2 => 2.,
            3 => 4.,
        };

        assert_approx_eq!(3., euclidean_distance(&a, &b).unwrap());
    }

    #[test]
    fn minkowski_distance_test() {
        let a = hash_map! {
            0 => 0.,
            2 => 1.,
            3 => 2.,
        };

        let b = hash_map! {
            0 => 2.,
            1 => 1.,
            2 => 3.,
            3 => 5.
        };

        assert_approx_eq!(
            manhattan_distance(&a, &b).unwrap(),
            minkowski_distance(&a, &b, 1).unwrap()
        );

        assert_approx_eq!(
            euclidean_distance(&a, &b).unwrap(),
            minkowski_distance(&a, &b, 2).unwrap()
        );
    }

    #[test]
    fn cosine_similarity_all_zeros_should_be_none() {
        let a = hash_map! {
            0 => 0.,
            1 => 0.,
            2 => 0.,
            3 => 0.,
            4 => 1.,
        };

        let b = hash_map! {
            0 => 1.,
            1 => 1.,
            2 => 1.,
            3 => 1.,
        };

        assert!(cosine_similarity(&a, &b).is_none());
    }
}
