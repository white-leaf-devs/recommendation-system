// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::{Controller, Entity};

pub struct LazyUserChunks<'a, User, UserId, Item, ItemId> {
    pub(crate) curr_offset: usize,
    pub(crate) chunk_size: usize,
    pub(crate) controller: &'a dyn Controller<User, UserId, Item, ItemId>,
}

impl<'a, User, UserId, Item, ItemId> Iterator for LazyUserChunks<'a, User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId>,
{
    type Item = Vec<User>;

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

pub struct LazyItemChunks<'a, User, UserId, Item, ItemId> {
    pub(crate) curr_offset: usize,
    pub(crate) chunk_size: usize,
    pub(crate) controller: &'a dyn Controller<User, UserId, Item, ItemId>,
}

impl<'a, User, UserId, Item, ItemId> Iterator for LazyItemChunks<'a, User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId>,
{
    type Item = Vec<Item>;

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
