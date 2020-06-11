use num_traits::float::Float;
use std::{
    collections::{HashSet, HashMap},
    hash::Hash,
    ops::{AddAssign, Mul, Sub},
};

#[derive(Debug, Copy, Clone)]
pub enum Method {
    AdjCosine,
    SlopeOne,
}

pub fn adjusted_cosine_means<U, K, V>(vecs: &HashMap<U, HashMap<K, V>>) -> HashMap<U, V>
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

pub fn fast_adjusted_cosine<U, K, V>(
    means: &HashMap<U, V>,
    vecs: &HashMap<U, HashMap<K, V>>,
    users_a: &HashSet<U>,
    users_b: &HashSet<U>,
    a: &K,
    b: &K
) -> Option<V>
where
    U: Hash + Eq,
    K: Hash + Eq,
    V: Float + AddAssign + Sub + Mul,
{
    let mut cov = None;
    let mut dev_a = None;
    let mut dev_b = None;

    let common_users = users_a.intersection(users_b);

    for common_user in common_users {
        let val_a = vecs[common_user][a];
        let val_b = vecs[common_user][b];
        let mean = means[common_user];
        *cov.get_or_insert_with(V::zero) += (val_a-mean) * (val_b-mean);
        *dev_a.get_or_insert_with(V::zero) += (val_a - mean).powi(2);
        *dev_b.get_or_insert_with(V::zero) += (val_b - mean).powi(2);
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

pub fn slow_adjusted_cosine<U, K, V>(vecs: &HashMap<U, HashMap<K, V>>, a: &K, b: &K) -> Option<V>
where
    U: Hash + Eq + Clone,
    K: Hash + Eq,
    V: Float + AddAssign + Sub + Mul,
{
    let means = adjusted_cosine_means(vecs);
    fast_adjusted_cosine(&means, vecs, a, b)
}
