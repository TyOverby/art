extern crate astar;
extern crate csv;
extern crate fnv;
extern crate png;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod model;
mod pathing;
mod precache;
use model::lat_lon_to_x_y;
use pathing::*;

fn main() {
    let (stops, connections) = model::get_connections();
    let cap_hill = Position::Arbitrary(47.616760, -122.318097);
    let _uw = Position::Arbitrary(47.656016, -122.312520);
    let ballard = Position::Arbitrary(47.668809, -122.382799);

    let precache = precache::get_cache(&stops, &connections, ballard);

    let mut searcher = TransitSearchProblem {
        stops: &stops,
        connections: &connections,
        start: ballard, // UW
        end: cap_hill,  // Cap Hill
        precache: precache,
    };

    println!("running search");
    let (path, _total_cost) = astar::astar(&mut searcher).unwrap();
    for (point, cost) in path {
        match point {
            Position::Arbitrary(x, y) => println!(
                "{} | Walk to {} {} at {:?}",
                cost,
                x,
                y,
                lat_lon_to_x_y(x, y)
            ),
            Position::BusStop(id, how) => {
                let stop = &stops[&id];
                println!(
                    "{} | {:?} to {} at {} {}",
                    cost, how, stop.name, stop.stop_x, stop.stop_y
                );
            }
        }
    }
}
