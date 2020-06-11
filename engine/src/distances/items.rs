use num_traits::float::Float;
use std::{
    collections::HashMap,
    hash::Hash,
    ops::{AddAssign, Mul, Sub},
};

pub fn pre_adjusted_cosine<U, K, V>(vecs: &HashMap<U, HashMap<K, V>>) -> HashMap<U, V>
where
    U: Hash + Eq + Clone,
    K: Hash + Eq,
    V: Float + AddAssign,
{
    let mut means = HashMap::new();
    for (id, vec) in vecs {
        let mut mean = None;
        let mut n = 0;

        for x in vec.values() {
            *mean.get_or_insert_with(V::zero) += *x;
            n += 1;
        }

        if let Some(mean) = mean {
            let mean = mean / V::from(n).unwrap();
            means.insert(id.to_owned(), mean);
        }
    }

    means
}

pub fn post_adjusted_cosine<U, K, V>(
    means: &HashMap<U, V>,
    vecs: &HashMap<U, HashMap<K, V>>,
    a: &K,
    b: &K,
) -> Option<V>
where
    U: Hash + Eq,
    K: Hash + Eq,
    V: Float + AddAssign + Sub + Mul + std::fmt::Display,
{
    let mut cov = None;
    let mut dev_a = None;
    let mut dev_b = None;

    for (id, vec) in vecs {
        let mean = if let Some(mean) = means.get(id) {
            *mean
        } else {
            continue;
        };

        match (vec.get(a), vec.get(b)) {
            (Some(val_a), Some(val_b)) => {
                *cov.get_or_insert_with(V::zero) += (*val_a - mean) * (*val_b - mean);
                *dev_a.get_or_insert_with(V::zero) += (*val_a - mean).powi(2);
                *dev_b.get_or_insert_with(V::zero) += (*val_b - mean).powi(2);

                println!("({} - {mean})({} - {mean}) +", val_a, val_b, mean = mean);
            }
            _ => continue,
        }
    }

    let num = cov?;
    let dem = dev_a?.sqrt() * dev_b?.sqrt();

    let res = num / dem;
    if res.is_nan() || res.is_infinite() {
        None
    } else {
        Some(res)
    }
}

pub fn adjusted_cosine<U, K, V>(vecs: &HashMap<U, HashMap<K, V>>, a: &K, b: &K) -> Option<V>
where
    U: Hash + Eq,
    K: Hash + Eq,
    V: Float + AddAssign + Sub + Mul,
{
    let mut cov = None;
    let mut dev_a = None;
    let mut dev_b = None;

    for vec in vecs.values() {
        let mut mean = None;
        let mut n = 0;
        for x in vec.values() {
            *mean.get_or_insert_with(V::zero) += *x;
            n += 1;
        }

        let mean = if let Some(mean) = mean {
            mean / V::from(n)?
        } else {
            continue;
        };

        match (vec.get(a), vec.get(b)) {
            (Some(val_a), Some(val_b)) => {
                *cov.get_or_insert_with(V::zero) += (*val_a - mean) * (*val_b - mean);
                *dev_a.get_or_insert_with(V::zero) += (*val_a - mean).powi(2);
                *dev_b.get_or_insert_with(V::zero) += (*val_b - mean).powi(2);
            }
            _ => continue,
        }
    }

    let num = cov?;
    let dem = dev_a?.sqrt() * dev_b?.sqrt();

    let res = num / dem;
    if res.is_nan() || res.is_infinite() {
        None
    } else {
        Some(res)
    }
}
