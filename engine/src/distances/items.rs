use crate::error::ErrorKind;
use crate::utils::common_keys_iter;
use controller::{MapedRatings, Ratings};
use num_traits::float::Float;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap, HashSet},
    hash::Hash,
    ops::{Add, AddAssign, Div, Mul, Sub},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Method {
    AdjCosine,
    SlopeOne,
}

type Means<UserId, Value> = HashMap<UserId, Value>;
type MinHeap<T> = BinaryHeap<Reverse<T>>;

#[derive(Debug, Clone, Default)]
pub struct MeanUsage<UserId>(UserId, u32);

impl<UserId> MeanUsage<UserId> {
    pub fn freq(&self) -> u32 {
        self.1
    }
}

impl<UserId> PartialEq for MeanUsage<UserId> {
    fn eq(&self, other: &Self) -> bool {
        self.freq().eq(&other.freq())
    }
}

impl<UserId> Eq for MeanUsage<UserId> {}

impl<UserId> PartialOrd for MeanUsage<UserId> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.freq().partial_cmp(&other.freq())
    }
}

impl<UserId> Ord for MeanUsage<UserId> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.freq().cmp(&other.freq())
    }
}

#[derive(Debug, Clone, Default)]
pub struct AdjCosine<UserId, Value>
where
    UserId: Hash + Eq,
{
    means_freq: MinHeap<MeanUsage<UserId>>,
    means: HashMap<UserId, Value>,
}

impl<UserId, Value> AdjCosine<UserId, Value>
where
    UserId: Hash + Eq,
{
    const THRESHOLD: usize = 1024;

    pub fn new() -> Self
    where
        UserId: Default,
        Value: Default,
    {
        Default::default()
    }

    pub fn has_mean_for(&self, user_id: &UserId) -> bool {
        self.means.contains_key(user_id)
    }

    pub fn shrink_means(&mut self) {
        while self.means.len() > Self::THRESHOLD {
            let Reverse(MeanUsage(uid, _)) = self.means_freq.pop().unwrap();
            self.means.remove(&uid);
        }
    }

    pub fn update_means<ItemId>(&mut self, maped_ratings: &MapedRatings<UserId, ItemId, Value>)
    where
        UserId: Clone,
        Value: Float + AddAssign,
    {
        for (id, ratings) in maped_ratings {
            let mut mean = None;
            let mut n = 0;

            for r in ratings.values() {
                *mean.get_or_insert_with(Value::zero) += *r;
                n += 1;
            }

            if let Some(mean) = mean {
                let mean = mean / Value::from(n).unwrap();
                self.means.insert(id.to_owned(), mean);
                self.means_freq.push(Reverse(MeanUsage(id.to_owned(), 0)));
            }
        }
    }

    pub fn calculate(
        &self,
        item_a_ratings: &Ratings<UserId, Value>,
        item_b_ratings: &Ratings<UserId, Value>,
    ) -> Result<Value, ErrorKind>
    where
        Value: Float + AddAssign + Sub,
    {
        let mut cov = None;
        let mut dev_a = None;
        let mut dev_b = None;

        for (user_id, (val_a, val_b)) in common_keys_iter(item_a_ratings, item_b_ratings) {
            let mean = if let Some(mean) = self.means.get(user_id) {
                *mean
            } else {
                continue;
            };

            *cov.get_or_insert_with(Value::zero) += (*val_a - mean) * (*val_b - mean);
            *dev_a.get_or_insert_with(Value::zero) += (*val_a - mean).powi(2);
            *dev_b.get_or_insert_with(Value::zero) += (*val_b - mean).powi(2);
        }

        let num = cov.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
        let dev_a = dev_a.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
        let dev_b = dev_b.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
        let dem = dev_a.sqrt() * dev_b.sqrt();

        let res = num / dem;
        if res.is_nan() {
            Err(ErrorKind::IndeterminateForm)
        } else if res.is_infinite() {
            Err(ErrorKind::DivisionByZero)
        } else {
            Ok(res)
        }
    }
}

pub fn adjusted_cosine_means<UserId, ItemId, Value>(
    vecs: &MapedRatings<UserId, ItemId, Value>,
) -> Means<&UserId, Value>
where
    UserId: Hash + Eq + Clone,
    ItemId: Hash + Eq,
    Value: Float + AddAssign,
{
    let mut means = Means::new();
    for (id, vec) in vecs {
        let mut mean = None;
        let mut n = 0;

        for x in vec.values() {
            *mean.get_or_insert_with(Value::zero) += *x;
            n += 1;
        }

        if let Some(mean) = mean {
            let mean = mean / Value::from(n).unwrap();
            means.insert(id, mean);
        }
    }

    means
}

pub fn fast_adjusted_cosine<UserId, ItemId, Value>(
    means: &Means<&UserId, Value>,
    vecs: &MapedRatings<UserId, ItemId, Value>,
    users_a: &HashSet<UserId>,
    users_b: &HashSet<UserId>,
    a: &ItemId,
    b: &ItemId,
) -> Result<Value, ErrorKind>
where
    UserId: Hash + Eq,
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub + Mul,
{
    let mut cov = None;
    let mut dev_a = None;
    let mut dev_b = None;

    for common_user in users_a.intersection(users_b) {
        if vecs.get(common_user).is_none() {
            continue;
        }

        match (
            vecs[common_user].get(a),
            vecs[common_user].get(b),
            means.get(common_user),
        ) {
            (Some(val_a), Some(val_b), Some(mean)) => {
                *cov.get_or_insert_with(Value::zero) += (*val_a - *mean) * (*val_b - *mean);
                *dev_a.get_or_insert_with(Value::zero) += (*val_a - *mean).powi(2);
                *dev_b.get_or_insert_with(Value::zero) += (*val_b - *mean).powi(2);
            }
            _ => continue,
        }
    }

    let num = cov.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let dev_a = dev_a.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let dev_b = dev_b.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let dem = dev_a.sqrt() * dev_b.sqrt();

    let res = num / dem;
    if res.is_nan() {
        Err(ErrorKind::IndeterminateForm)
    } else if res.is_infinite() {
        Err(ErrorKind::DivisionByZero)
    } else {
        Ok(res)
    }
}

pub fn slow_adjusted_cosine<UserId, ItemId, Value>(
    vecs: &MapedRatings<UserId, ItemId, Value>,
    users_a: &HashSet<UserId>,
    users_b: &HashSet<UserId>,
    a: &ItemId,
    b: &ItemId,
) -> Result<Value, ErrorKind>
where
    UserId: Hash + Eq + Clone,
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub + Mul,
{
    let means = adjusted_cosine_means(vecs);
    fast_adjusted_cosine(&means, vecs, users_a, users_b, a, b)
}

pub fn normalize_user_ratings<ItemId, Value>(
    ratings: &Ratings<ItemId, Value>,
    min_rating: Value,
    max_rating: Value,
) -> Result<Ratings<&ItemId, Value>, ErrorKind>
where
    ItemId: Hash + Eq,
    Value: Float + Sub + Mul + Div,
{
    if (max_rating - min_rating).is_zero() {
        return Err(ErrorKind::DivisionByZero);
    }

    let mut normalized_ratings = Ratings::new();
    for (id, value) in ratings {
        let two = Value::from(2.0).ok_or_else(|| ErrorKind::ConvertType)?;
        let normalized = (two * (*value) - min_rating - max_rating) / (max_rating - min_rating);
        normalized_ratings.insert(id, normalized);
    }

    Ok(normalized_ratings)
}

pub fn denormalize_user_rating<Value>(
    normalized_rating: Value,
    min_rating: Value,
    max_rating: Value,
) -> Result<Value, ErrorKind>
where
    Value: Float + Sub + Add + Div + Mul,
{
    let one = Value::one();
    let two = Value::from(2.0).ok_or_else(|| ErrorKind::ConvertType)?;

    Ok((one / two) * ((normalized_rating + one) * (max_rating - min_rating)) + min_rating)
}
