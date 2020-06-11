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

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};
use recommend::distances::{euclidean_distance, manhattan_distance};
use std::collections::HashMap;

fn generate_records(size: u64) -> (HashMap<u64, f64>, HashMap<u64, f64>) {
    let mut rng = thread_rng();

    let mut a = HashMap::new();
    let mut b = HashMap::new();

    for i in 0..size {
        a.insert(i, rng.gen_range(-10., 10.));

        // Create one that's shortest
        if i > (0.3 * size as f64) as u64 {
            b.insert(i, rng.gen_range(-10., 10.));
        }
    }

    (a, b)
}

fn manhattan_distance_1000(c: &mut Criterion) {
    let (a, b) = generate_records(1000);

    c.bench_function("manhattan 1000", |bench| {
        bench.iter(|| manhattan_distance(black_box(&a), black_box(&b)))
    });
}

fn manhattan_distance_10_000(c: &mut Criterion) {
    let (a, b) = generate_records(10_000);

    c.bench_function("manhattan 10000", |bench| {
        bench.iter(|| manhattan_distance(black_box(&a), black_box(&b)))
    });
}

fn euclidean_distance_1000(c: &mut Criterion) {
    let (a, b) = generate_records(1000);

    c.bench_function("euclidean 1000", |bench| {
        bench.iter(|| euclidean_distance(black_box(&a), black_box(&b)))
    });
}

fn euclidean_distance_10_000(c: &mut Criterion) {
    let (a, b) = generate_records(10_000);

    c.bench_function("euclidean 10000", |bench| {
        bench.iter(|| euclidean_distance(black_box(&a), black_box(&b)))
    });
}

criterion_group! {
    name = distances_1000;
    config = Criterion::default();
    targets = manhattan_distance_1000, euclidean_distance_1000
}

criterion_group! {
    name = distances_10_000;
    config = Criterion::default();
    targets = manhattan_distance_10_000, euclidean_distance_10_000

}

criterion_main!(distances_1000, distances_10_000);
