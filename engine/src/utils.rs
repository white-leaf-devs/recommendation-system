// Copyright (C) 2020 Kevin Del Castillo Ramírez
//
// This file is part of recommendation-system.
//
// recommendation-system is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// recommendation-system is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with recommendation-system.  If not, see <http://www.gnu.org/licenses/>.

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
    let (shortest, longest) = if a.len() > b.len() { (b, a) } else { (a, b) };

    CommonKeyIterator {
        shortest: shortest.iter(),
        longest,
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

impl<'a, K, V> Iterator for CommonKeyIterator<'a, K, V>
where
    K: Hash + Eq,
{
    type Item = (&'a K, (&'a V, &'a V));

    fn next(&mut self) -> Option<Self::Item> {
        let mut a_val = self.shortest.next()?;

        loop {
            if let Some(b_val) = self.longest.get(a_val.0) {
                break Some((a_val.0, (a_val.1, b_val)));
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
