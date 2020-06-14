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
pub struct MeanUsage<UserId>(UserId, u32, usize);

impl<UserId> MeanUsage<UserId> {
    pub fn freq(&self) -> u32 {
        self.1
    }

    pub fn size(&self) -> usize {
        self.2
    }
}

impl<UserId> PartialEq for MeanUsage<UserId> {
    fn eq(&self, other: &Self) -> bool {
        self.freq().eq(&other.freq()) && self.size().eq(&other.size())
    }
}

impl<UserId> Eq for MeanUsage<UserId> {}

impl<UserId> PartialOrd for MeanUsage<UserId> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.freq()
            .partial_cmp(&other.freq())
            .and_then(|ord| match ord {
                Ordering::Equal => self.size().partial_cmp(&other.size()),
                _ => Some(ord),
            })
    }
}

impl<UserId> Ord for MeanUsage<UserId> {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.freq().cmp(&other.freq());
        match ord {
            Ordering::Equal => self.size().cmp(&other.size()),
            _ => ord,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AdjCosine<UserId, Value>
where
    UserId: Hash + Eq,
{
    // The value is a tuple of (usage, size)
    mfreq: HashMap<UserId, (u32, usize)>,
    means: HashMap<UserId, Value>,
}

impl<UserId, Value> AdjCosine<UserId, Value>
where
    UserId: Hash + Eq,
{
    const THRESHOLD: usize = 1048576;

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

    pub fn shrink_means(&mut self)
    where
        UserId: Clone,
    {
        if self.means.len() < Self::THRESHOLD {
            return;
        }

        let mut min_heap: MinHeap<_> = self
            .mfreq
            .iter()
            .map(|(user_id, (usage, size))| Reverse(MeanUsage(user_id.to_owned(), *usage, *size)))
            .collect();

        while self.means.len() > Self::THRESHOLD {
            let Reverse(MeanUsage(uid, _, _)) = min_heap.pop().unwrap();
            self.means.remove(&uid);
            self.mfreq.remove(&uid);
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
                self.mfreq.insert(id.to_owned(), (0, ratings.len()));
            }
        }
    }

    pub fn calculate(
        &mut self,
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
                let (freq, _) = self
                    .mfreq
                    .get_mut(user_id)
                    .expect("Broken invariant: mfreq doesn't contain an already stored mean");

                *freq += 1;
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
