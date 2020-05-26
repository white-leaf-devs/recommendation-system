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

//use controller::{Controller, Entity, Rating};
//use std::marker::PhantomData;

pub mod distances;

pub enum Distance {
    Manhattan,
    Euclidean,
    Minkowski(usize),
    CosineSimilarity,
    PearsonCorrelation,
}

/*
pub struct Engine<C, U, I, UId, IId>
where
    U: Entity<Id = UId>,
    I: Entity<Id = IId>,
    C: Controller<U, I>,
{
    controller: C,

    _user: PhantomData<U>,
    _item: PhantomData<I>,
    _user_id: PhantomData<UId>,
    _item_id: PhantomData<IId>,
}

impl<C, U, I, UI, II> Engine<C, U, I, UI, II> {
    fn distance(&self, id_a: UI, id_b: UI) -> Option<f64> {}
}
*/
