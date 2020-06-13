#![allow(clippy::implicit_hasher)]

use crate::error::ErrorKind;
use crate::utils::common_keys_iter;
use controller::Ratings;
use num_traits::float::Float;
use std::{
    collections::HashSet,
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

    pub fn is_distance(&self) -> bool {
        !self.is_similarity()
    }
}

pub fn distance<ItemId, Value>(
    a: &Ratings<ItemId, Value>,
    b: &Ratings<ItemId, Value>,
    method: Method,
) -> Result<Value, ErrorKind>
where
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub + Mul + MulAssign,
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

pub fn manhattan_distance<ItemId, Value>(
    a: &Ratings<ItemId, Value>,
    b: &Ratings<ItemId, Value>,
) -> Result<Value, ErrorKind>
where
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub,
{
    let mut dist = None;
    for (x, y) in common_keys_iter(a, b) {
        *dist.get_or_insert_with(Value::zero) += (*y - *x).abs();
    }

    dist.ok_or(ErrorKind::NoMatchingRatings)
}

pub fn euclidean_distance<ItemId, Value>(
    a: &Ratings<ItemId, Value>,
    b: &Ratings<ItemId, Value>,
) -> Result<Value, ErrorKind>
where
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub,
{
    let mut dist = None;
    for (x, y) in common_keys_iter(a, b) {
        *dist.get_or_insert_with(Value::zero) += (*y - *x).powi(2);
    }

    dist.map(Value::sqrt).ok_or(ErrorKind::NoMatchingRatings)
}

pub fn minkowski_distance<ItemId, Value>(
    a: &Ratings<ItemId, Value>,
    b: &Ratings<ItemId, Value>,
    p: usize,
) -> Result<Value, ErrorKind>
where
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub,
{
    if p == 0 {
        panic!("Received p = 0 for minkowski distance!");
    }

    let mut dist = None;
    for (x, y) in common_keys_iter(a, b) {
        *dist.get_or_insert_with(Value::zero) += (*y - *x).abs().powi(p as i32);
    }

    let exp = Value::one() / Value::from(p).ok_or(ErrorKind::ConvertType)?;
    dist.map(|dist| dist.powf(exp))
        .ok_or(ErrorKind::NoMatchingRatings)
}

pub fn jaccard_index<ItemId, Value>(
    a: &Ratings<ItemId, Value>,
    b: &Ratings<ItemId, Value>,
) -> Result<Value, ErrorKind>
where
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub,
{
    match (a.is_empty(), b.is_empty()) {
        // Both are empty, cannot compute the index
        (true, true) => Err(ErrorKind::EmptyRatings),

        // One of them is empty, the result is zero
        (true, _) | (_, true) => Ok(Value::zero()),

        // Both have at least one element, proceed
        _ => {
            let a_keys: HashSet<_> = a.keys().collect();
            let b_keys: HashSet<_> = b.keys().collect();

            let union = a_keys.union(&b_keys).count();
            let inter = a_keys.intersection(&b_keys).count();

            let inter = Value::from(inter).ok_or(ErrorKind::ConvertType)?;
            let union = Value::from(union).ok_or(ErrorKind::ConvertType)?;

            Ok(inter / union)
        }
    }
}

pub fn jaccard_distance<ItemId, Value>(
    a: &Ratings<ItemId, Value>,
    b: &Ratings<ItemId, Value>,
) -> Result<Value, ErrorKind>
where
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub,
{
    Ok(Value::one() - jaccard_index(a, b)?)
}

pub fn cosine_similarity<ItemId, Value>(
    a: &Ratings<ItemId, Value>,
    b: &Ratings<ItemId, Value>,
) -> Result<Value, ErrorKind>
where
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub + Mul,
{
    let mut a_norm = None;
    let mut b_norm = None;
    let mut dot_prod = None;

    for (x, y) in common_keys_iter(a, b) {
        *a_norm.get_or_insert_with(Value::zero) += x.powi(2);
        *b_norm.get_or_insert_with(Value::zero) += y.powi(2);
        *dot_prod.get_or_insert_with(Value::zero) += (*x) * (*y);
    }

    let dot_prod = dot_prod.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let a_norm = a_norm.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let b_norm = b_norm.ok_or_else(|| ErrorKind::NoMatchingRatings)?;

    let cos_sim = dot_prod / (a_norm.sqrt() * b_norm.sqrt());
    if cos_sim.is_nan() {
        Err(ErrorKind::IndeterminateForm)
    } else if cos_sim.is_infinite() {
        Err(ErrorKind::DivisionByZero)
    } else {
        Ok(cos_sim)
    }
}

pub fn pearson_correlation<ItemId, Value>(
    a: &Ratings<ItemId, Value>,
    b: &Ratings<ItemId, Value>,
) -> Result<Value, ErrorKind>
where
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub + Mul,
{
    let mut mean_x = None;
    let mut mean_y = None;
    let mut n = 0;

    for (x, y) in common_keys_iter(a, b) {
        *mean_x.get_or_insert_with(Value::zero) += *x;
        *mean_y.get_or_insert_with(Value::zero) += *y;
        n += 1;
    }

    let n = Value::from(n).ok_or_else(|| ErrorKind::ConvertType)?;
    let mean_x = mean_x.ok_or_else(|| ErrorKind::NoMatchingRatings)? / n;
    let mean_y = mean_y.ok_or_else(|| ErrorKind::NoMatchingRatings)? / n;

    let mut cov = None;
    let mut std_dev_a = None;
    let mut std_dev_b = None;

    for (x, y) in common_keys_iter(a, b) {
        *cov.get_or_insert_with(Value::zero) += (*x - mean_x) * (*y - mean_y);
        *std_dev_a.get_or_insert_with(Value::zero) += (*x - mean_x).powi(2);
        *std_dev_b.get_or_insert_with(Value::zero) += (*y - mean_y).powi(2);
    }

    let cov = cov.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let std_dev_a = std_dev_a.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let std_dev_b = std_dev_b.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let std_dev = std_dev_a.sqrt() * std_dev_b.sqrt();

    let pearson = cov / std_dev;
    if pearson.is_nan() {
        Err(ErrorKind::IndeterminateForm)
    } else if pearson.is_infinite() {
        Err(ErrorKind::DivisionByZero)
    } else {
        Ok(pearson)
    }
}

pub fn pearson_approximation<ItemId, Value>(
    a: &Ratings<ItemId, Value>,
    b: &Ratings<ItemId, Value>,
) -> Result<Value, ErrorKind>
where
    ItemId: Hash + Eq,
    Value: Float + AddAssign + Sub + Mul,
{
    let mut sum_x = None;
    let mut sum_y = None;
    let mut sum_x_sq = None;
    let mut sum_y_sq = None;
    let mut dot_prod = None;
    let mut n = 0;

    for (x, y) in common_keys_iter(a, b) {
        *sum_x.get_or_insert_with(Value::zero) += *x;
        *sum_y.get_or_insert_with(Value::zero) += *y;
        *sum_x_sq.get_or_insert_with(Value::zero) += x.powi(2);
        *sum_y_sq.get_or_insert_with(Value::zero) += y.powi(2);
        *dot_prod.get_or_insert_with(Value::zero) += (*x) * (*y);
        n += 1;
    }

    let n = Value::from(n).ok_or_else(|| ErrorKind::ConvertType)?;
    let dot_prod = dot_prod.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let sum_x = sum_x.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let sum_y = sum_y.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let num = dot_prod - (sum_x * sum_y) / n;

    let sum_x_sq = sum_x_sq.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let sum_y_sq = sum_y_sq.ok_or_else(|| ErrorKind::NoMatchingRatings)?;
    let dem_x = sum_x_sq - sum_x.powi(2) / n;
    let dem_y = sum_y_sq - sum_y.powi(2) / n;
    let dem = dem_x.sqrt() * dem_y.sqrt();

    let pearson = num / dem;
    if pearson.is_nan() {
        Err(ErrorKind::IndeterminateForm)
    } else if pearson.is_infinite() {
        Err(ErrorKind::DivisionByZero)
    } else {
        Ok(pearson)
    }
}
