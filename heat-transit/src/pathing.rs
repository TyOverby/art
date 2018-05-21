use astar::SearchProblem;
use model::{lat_lon_to_x_y, PreConnections, StopId, Stops};
use precache::RouteCache;
use std::hash::{Hash, Hasher};
use time::TimeCost;

const WALKING_SPEED: f64 = 0.0014;
const DRIVING_SPEED: f64 = 0.0178;
const BUS_WAIT_TIME: f32 = 7.5 * 60.0; // 7.5 minutes
const MAX_WALK_TIME: f32 = 20.0 * 60.0; // 1 hour

fn travel_time((ax, ay): (f64, f64), (bx, by): (f64, f64), speed: f64) -> f64 {
    let dx = ax - bx;
    let dy = ay - by;
    let distance = (dx * dx + dy * dy).sqrt();
    distance / speed
}

pub fn walking_time(s: (f64, f64), e: (f64, f64)) -> TimeCost {
    TimeCost::of_walking(travel_time(s, e, WALKING_SPEED) as f32)
}

fn driving_time(s: (f64, f64), e: (f64, f64)) -> TimeCost {
    TimeCost::of_bus(travel_time(s, e, DRIVING_SPEED) as f32)
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HowGet {
    Walk,
    Bus,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Position {
    BusStop(StopId, HowGet),
    LatLon(f64, f64),
    Custom(f64, f64),
}

impl Position {
    pub fn get_coords(&self, stops: &Stops) -> (f64, f64) {
        match self {
            Position::Custom(x, y) => (*x, *y),
            Position::LatLon(lat, lon) => lat_lon_to_x_y(*lat, *lon),
            Position::BusStop(id, _) => {
                let stop = &stops[id];
                (stop.stop_x, stop.stop_y)
            }
        }
    }
}

impl Eq for Position {}
impl Hash for Position {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self {
            Position::BusStop(StopId(id), _) => state.write_u32(*id),
            Position::LatLon(a, b) => {
                state.write_u64(a.to_bits());
                state.write_u64(b.to_bits());
            }
            Position::Custom(a, b) => {
                state.write_u64(a.to_bits());
                state.write_u64(b.to_bits());
            }
        }
    }
}

pub struct TransitSearchProblem<'a> {
    pub stops: &'a Stops,
    pub connections: &'a PreConnections,
    pub end: Position,
    pub precache: RouteCache,
}

impl<'a> SearchProblem for TransitSearchProblem<'a> {
    type Node = Position;
    type Cost = TimeCost;
    type Iter = ::std::vec::IntoIter<(Position, Self::Cost)>;
    fn is_end(&self, a: &Self::Node) -> bool {
        a == &self.end
    }
    fn heuristic(&self, p: &Self::Node) -> Self::Cost {
        if let Position::BusStop(id, _) = p {
            if let Some(&r) = self.precache.get(id) {
                return r;
            }
        }

        let p = p.get_coords(self.stops);
        let g = self.end.get_coords(self.stops);
        driving_time(g, p)
    }

    fn neighbors(&self, cur: &Self::Node, cost: &Self::Cost) -> Self::Iter {
        if cost.walk_time > MAX_WALK_TIME {
            return vec![].into_iter();
        }

        let mut neighbors = vec![];
        let end_coords = self.end.get_coords(&self.stops);
        let cur_coords = cur.get_coords(&self.stops);

        // Walk to the end
        let walk_time = walking_time(cur_coords, end_coords);
        neighbors.push((self.end, walk_time));

        // Walk to every bus stop
        for (id, stop) in self.stops {
            let walk_time = walking_time(cur_coords, (stop.stop_x, stop.stop_y));
            if walk_time.walk_time < MAX_WALK_TIME {
                neighbors.push((Position::BusStop(*id, HowGet::Walk), walk_time))
            }
        }

        // If at a bus stop, travel to other things on the route
        if let Position::BusStop(id, _) = cur {
            if self.connections.contains_key(id) {
                for (end, info) in &self.connections[id] {
                    neighbors.push((
                        Position::BusStop(*end, HowGet::Bus),
                        TimeCost::of_bus(info.time) + TimeCost::of_waiting(BUS_WAIT_TIME),
                    ));
                }
            }
        }
        neighbors.into_iter()
    }
}
