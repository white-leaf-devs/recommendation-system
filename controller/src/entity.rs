// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use prettytable::{cell, format::consts::FORMAT_NO_LINESEP, row, table, Table};
use std::collections::HashMap;

pub trait Entity {
    type Id;

    fn get_id(&self) -> Self::Id;
    fn get_data(&self) -> HashMap<String, String> {
        Default::default()
    }
}

pub trait ToTable {
    fn to_table(&self) -> Table;
}

impl<I: ToString, E: Entity<Id = I>> ToTable for E {
    fn to_table(&self) -> Table {
        let mut table = table![["id", self.get_id()]];

        for (key, val) in self.get_data() {
            table.add_row(row![key, val]);
        }

        table.set_format(*FORMAT_NO_LINESEP);
        table
    }
}

impl<K, V, B> ToTable for HashMap<K, V, B>
where
    K: ToString,
    V: ToString,
{
    fn to_table(&self) -> Table {
        let mut table = Table::new();

        for (key, val) in self {
            table.add_row(row![key, val]);
        }

        table.set_format(*FORMAT_NO_LINESEP);
        table
    }
}
