use astar::SearchProblem;
use model::{Connections, StopId, Stops};
use std::hash::{Hash, Hasher};

const WALKING_SPEED: f64 = 0.0014;
const DRIVING_SPEED: f64 = 0.0178;

fn travel_time((ax, ay): (f64, f64), (bx, by): (f64, f64), speed: f64) -> f64 {
    let dx = ax - bx;
    let dy = ay - by;
    let distance = (dx * dx + dy + dy).sqrt();
    (distance / speed)
}

fn walking_time(s: (f64, f64), e: (f64, f64)) -> f64 {
    travel_time(s, e, WALKING_SPEED)
}

fn driving_time(s: (f64, f64), e: (f64, f64)) -> f64 {
    travel_time(s, e, DRIVING_SPEED)
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Position {
    BusStop(StopId),
    Arbitrary(f64, f64),
}

impl Position {
    fn get_coords(&self, stops: &Stops) -> (f64, f64) {
        match self {
            Position::Arbitrary(x, y) => (*x, *y),
            Position::BusStop(id) => {
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
            Position::BusStop(StopId(id)) => state.write_u32(*id),
            Position::Arbitrary(a, b) => {
                state.write_u64(a.to_bits());
                state.write_u64(b.to_bits());
            }
        }
    }
}

pub struct TransitSearchProblem<'a> {
    pub stops: &'a Stops,
    pub connections: &'a Connections,
    pub start: Position,
    pub end: Position,
}

impl<'a> SearchProblem for TransitSearchProblem<'a> {
    type Node = Position;
    type Cost = f64;
    type Iter = ::std::vec::IntoIter<(Position, Self::Cost)>;
    fn start(&self) -> Self::Node {
        self.start
    }
    fn is_end(&self, a: &Self::Node) -> bool {
        a == &self.end
    }
    fn heuristic(&self, p: &Self::Node) -> Self::Cost {
        let p = p.get_coords(self.stops);
        let g = self.end.get_coords(self.stops);
        driving_time(g, p)
    }

    fn neighbors(&mut self, cur: &Self::Node) -> Self::Iter {
        let mut neighbors = vec![];
        let end_coords = self.end.get_coords(&self.stops);
        let cur_coords = cur.get_coords(&self.stops);

        // Walk to the end
        neighbors.push((self.end, walking_time(cur_coords, end_coords)));

        // Walk to every bus stop
        for (id, stop) in self.stops {
            let walk_time = walking_time(cur_coords, (stop.stop_x, stop.stop_y));
            neighbors.push((Position::BusStop(*id), walk_time))
        }

        neighbors.into_iter()
    }
}
