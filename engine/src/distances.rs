// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

pub mod items;
pub mod users;

#[cfg(test)]
mod tests {
    use super::users::*;
    use assert_approx_eq::*;
    use common_macros::hash_map;

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

        assert!(manhattan_distance(&a, &b).is_err());
        assert!(euclidean_distance(&a, &b).is_err());
        assert!(minkowski_distance(&a, &b, 1).is_err());
        assert!(minkowski_distance(&a, &b, 2).is_err());
        assert!(minkowski_distance(&a, &b, 3).is_err());
        assert!(cosine_similarity(&a, &b).is_err());
        assert!(pearson_correlation(&a, &b).is_err());
        assert!(pearson_approximation(&a, &b).is_err());
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

        assert_approx_eq!(3_f64, manhattan_distance(&a, &b).unwrap());
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

        assert_approx_eq!(3_f64, euclidean_distance(&a, &b).unwrap());
    }

    #[test]
    fn minkowski_distance_test() {
        let a = hash_map! {
            0 => 0_f64,
            2 => 1.,
            3 => 2.,
        };

        let b = hash_map! {
            0 => 2_f64,
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

        assert!(cosine_similarity(&a, &b).is_err());
    }
}
