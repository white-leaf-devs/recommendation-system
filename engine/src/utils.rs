// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::collections::{hash_map::Iter as MapIter, HashMap};
use std::hash::Hash;

// Creating a common key iterator is kinda interesting since it'll
// decide which map is going to be iterated based on it's length.
// Basically if one of them is empty, it'll choose it as the main
// iterator, therefore ending the iteration early.
pub fn common_keys_iter<'a, K, V>(
    a: &'a HashMap<K, V>,
    b: &'a HashMap<K, V>,
) -> CommonKeyIterator<'a, K, V>
where
    K: Hash + Eq,
{
    let is_swaped;

    let (shortest, longest) = if a.len() > b.len() {
        is_swaped = true;
        (b, a)
    } else {
        is_swaped = false;
        (a, b)
    };

    CommonKeyIterator {
        shortest: shortest.iter(),
        longest,
        is_swaped,
    }
}

#[derive(Debug)]
pub struct CommonKeyIterator<'a, K, V>
where
    K: Hash + Eq,
{
    shortest: MapIter<'a, K, V>,
    longest: &'a HashMap<K, V>,
    is_swaped: bool,
}

impl<'a, K, V> Iterator for CommonKeyIterator<'a, K, V>
where
    K: Hash + Eq,
{
    type Item = (&'a K, (&'a V, &'a V));

    fn next(&mut self) -> Option<Self::Item> {
        let mut a_val = self.shortest.next()?;

        loop {
            if let Some(b_val) = self.longest.get(a_val.0) {
                let values = if self.is_swaped {
                    (b_val, a_val.1)
                } else {
                    (a_val.1, b_val)
                };
                break Some((a_val.0, values));
            } else {
                a_val = self.shortest.next()?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let iter = common_keys_iter(&a, &b);

        for (k, _) in iter {
            assert!(k == &2 || k == &0 || k == &5);
        }
    }
}
