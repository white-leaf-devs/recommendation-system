use controller::Ratings;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct MapedDistance(pub String, pub f64, pub Option<Ratings>);

impl MapedDistance {
    pub fn dist(&self) -> f64 {
        self.1
    }

    pub fn ratings(&self) -> Option<&Ratings> {
        self.2.as_ref()
    }
}

impl PartialEq for MapedDistance {
    fn eq(&self, other: &Self) -> bool {
        self.dist().eq(&other.dist())
    }
}

impl Eq for MapedDistance {}

impl PartialOrd for MapedDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.dist().partial_cmp(&other.dist())
    }
}

impl Ord for MapedDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist().partial_cmp(&other.dist()).unwrap()
    }
}
