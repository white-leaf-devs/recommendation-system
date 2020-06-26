// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::{Controller, Entity};

pub struct LazyUserChunks<'a, C, U>
where
    C: Controller<User = U>,
    U: Entity,
{
    pub(crate) curr_offset: usize,
    pub(crate) chunk_size: usize,
    pub(crate) controller: &'a C,
}

impl<'a, C, U> Iterator for LazyUserChunks<'a, C, U>
where
    C: Controller<User = U>,
    U: Entity,
{
    type Item = Vec<U>;

    fn next(&mut self) -> Option<Self::Item> {
        let users = self
            .controller
            .users_offset_limit(self.curr_offset, self.chunk_size)
            .ok();

        self.curr_offset += self.chunk_size;
        match users {
            Some(users) => {
                if users.is_empty() {
                    None
                } else {
                    Some(users)
                }
            }
            None => None,
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.curr_offset = n * self.chunk_size;
        self.next()
    }
}

pub struct LazyItemChunks<'a, C, I>
where
    C: Controller<Item = I>,
    I: Entity,
{
    pub(crate) curr_offset: usize,
    pub(crate) chunk_size: usize,
    pub(crate) controller: &'a C,
}

impl<'a, C, I> Iterator for LazyItemChunks<'a, C, I>
where
    C: Controller<Item = I>,
    I: Entity,
{
    type Item = Vec<I>;

    fn next(&mut self) -> Option<Self::Item> {
        let items = self
            .controller
            .items_offset_limit(self.curr_offset, self.chunk_size)
            .ok();

        self.curr_offset += self.chunk_size;
        match items {
            Some(items) => {
                if items.is_empty() {
                    None
                } else {
                    Some(items)
                }
            }
            None => None,
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.curr_offset = n * self.chunk_size;
        self.next()
    }
}
