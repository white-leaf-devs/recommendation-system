use controller::Ratings;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct MapedDistance<UserId, ItemId>(pub UserId, pub f64, pub Option<Ratings<ItemId>>);

impl<UserId, ItemId> MapedDistance<UserId, ItemId> {
    pub fn dist(&self) -> f64 {
        self.1
    }

    pub fn ratings(&self) -> Option<&Ratings<ItemId>> {
        self.2.as_ref()
    }
}

impl<UserId, ItemId> PartialEq for MapedDistance<UserId, ItemId> {
    fn eq(&self, other: &Self) -> bool {
        self.dist().eq(&other.dist())
    }
}

impl<UserId, ItemId> Eq for MapedDistance<UserId, ItemId> {}

impl<UserId, ItemId> PartialOrd for MapedDistance<UserId, ItemId> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.dist().partial_cmp(&other.dist())
    }
}

impl<UserId, ItemId> Ord for MapedDistance<UserId, ItemId> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist().partial_cmp(&other.dist()).unwrap()
    }
}
