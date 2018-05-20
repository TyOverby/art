extern crate astar;
extern crate csv;
extern crate png;
extern crate spade;
#[macro_use]
extern crate serde_derive;

mod model;
mod pathing;

use pathing::*;

fn main() {
    let (stops, connections) = model::read_connections();
    let mut searcher = TransitSearchProblem {
        stops: &stops,
        connections: &connections,
        start: Position::Arbitrary(47.656016, -122.312520), // UW
        end: Position::Arbitrary(47.616760, -122.318097),   // Cap Hill
    };

    println!("running search");
    println!("{:?}", astar::astar(&mut searcher));
}
