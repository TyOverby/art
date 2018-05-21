use std::cmp::{Ord, Ordering, PartialOrd};

#[derive(PartialEq, Deserialize, Serialize, Copy, Clone, Debug)]
pub struct TimeCost {
    pub walk_time: f32,
    pub bus_time: f32,
    pub wait_time: f32,
    pub transfers: u32,
}

impl TimeCost {
    pub fn with_all(a: f32) -> Self {
        TimeCost {
            walk_time: a,
            bus_time: a,
            wait_time: a,
            transfers: 0,
        }
    }

    pub fn of_walking(a: f32) -> Self {
        TimeCost {
            walk_time: a,
            bus_time: 0.0,
            wait_time: 0.0,
            transfers: 0,
        }
    }

    pub fn of_bus(a: f32) -> Self {
        TimeCost {
            walk_time: 0.0,
            bus_time: a,
            wait_time: 0.0,
            transfers: 1,
        }
    }

    pub fn of_waiting(a: f32) -> Self {
        TimeCost {
            walk_time: 0.0,
            bus_time: 0.0,
            wait_time: a,
            transfers: 0,
        }
    }

    pub fn total(&self) -> f32 {
        let TimeCost {
            walk_time,
            bus_time,
            wait_time,
            transfers: _,
        } = *self;
        walk_time + bus_time + wait_time
    }
}

impl Eq for TimeCost {}

impl Ord for TimeCost {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for TimeCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.total().partial_cmp(&other.total())
    }
}

impl ::std::ops::Add for TimeCost {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        TimeCost {
            walk_time: self.walk_time + other.walk_time,
            bus_time: self.bus_time + other.bus_time,
            wait_time: self.wait_time + other.wait_time,
            transfers: self.transfers + other.transfers,
        }
    }
}

impl ::num_traits::Zero for TimeCost {
    fn is_zero(&self) -> bool {
        self.total() == 0.0
    }

    fn zero() -> Self {
        TimeCost {
            walk_time: 0.0,
            bus_time: 0.0,
            wait_time: 0.0,
            transfers: 0,
        }
    }
}
