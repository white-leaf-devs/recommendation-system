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
use recommend::record::Record;
use std::collections::HashMap;

fn generate_records(size: u64) -> (Record<f64>, Record<f64>) {
    let mut rng = thread_rng();

    let mut a = HashMap::new();
    let mut b = HashMap::new();

    for i in 0..size {
        if rng.gen() {
            a.insert(i, rng.gen_range(-10., 10.));
        }

        if rng.gen() {
            b.insert(i, rng.gen_range(-10., 10.));
        }
    }

    (a.into(), b.into())
}

fn manhattan_distance_10000(c: &mut Criterion) {
    let (a, b) = generate_records(10000);

    c.bench_function("manhattan 10000 kinda", |bench| {
        bench.iter(|| a.manhattan_distance(black_box(&b)))
    });
}

fn manhattan_distance_100000(c: &mut Criterion) {
    let (a, b) = generate_records(100_000);

    c.bench_function("manhattan 100000 kinda", |bench| {
        bench.iter(|| a.manhattan_distance(black_box(&b)))
    });
}

fn euclidean_distance_10000(c: &mut Criterion) {
    let (a, b) = generate_records(10000);

    c.bench_function("euclidean 10000 kinda", |bench| {
        bench.iter(|| a.euclidean_distance(black_box(&b)))
    });
}

fn euclidean_distance_100000(c: &mut Criterion) {
    let (a, b) = generate_records(100_000);

    c.bench_function("euclidean 100000 kinda", |bench| {
        bench.iter(|| a.euclidean_distance(black_box(&b)))
    });
}

criterion_group!(
    distances,
    manhattan_distance_10000,
    euclidean_distance_10000,
    manhattan_distance_100000,
    euclidean_distance_100000
);

criterion_main!(distances);
