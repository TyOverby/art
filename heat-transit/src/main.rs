extern crate astar;
extern crate csv;
extern crate ord_subset;
extern crate png;
extern crate spade;
#[macro_use]
extern crate serde_derive;

mod model;
mod pathing;
use model::lat_lon_to_x_y;
use ord_subset::OrdSubsetIterExt;
use pathing::*;

fn main() {
    let (stops, connections) = model::read_connections();
    let cap_hill = Position::Arbitrary(47.616760, -122.318097);
    let uw = Position::Arbitrary(47.656016, -122.312520);

    let mut searcher = TransitSearchProblem {
        stops: &stops,
        connections: &connections,
        start: uw,     // UW
        end: cap_hill, // Cap Hill
    };

    println!("running search");
    let path = astar::astar(&mut searcher);
    for point in path.unwrap() {
        match point {
            Position::Arbitrary(x, y) => {
                println!("Walk to {} {} at {:?}", x, y, lat_lon_to_x_y(x, y))
            }
            Position::BusStop(id, how) => {
                let stop = &stops[&id];
                println!(
                    "{:?} to {} at {} {}",
                    how, stop.name, stop.stop_x, stop.stop_y
                );
            }
        }
    }
}
