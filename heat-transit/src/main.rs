extern crate astar;
extern crate csv;
extern crate fnv;
extern crate num_traits;
extern crate png;
extern crate rayon;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod draw;
mod model;
mod pathing;
mod precache;
mod time;

use pathing::*;

fn main() {
    let (stops, connections) = model::get_connections();
    let cap_hill = Position::LatLon(47.616760, -122.318097);
    let _uw = Position::LatLon(47.656016, -122.312520);
    let ballard = Position::LatLon(47.668809, -122.382799);

    let precache = precache::get_cache(&stops, &connections, ballard);

    let searcher = TransitSearchProblem {
        stops: &stops,
        connections: &connections,
        end: cap_hill, // Cap Hill
        precache: precache,
    };

    draw::draw(|x, y| {
        let start = Position::Custom(x, y);
        astar::astar(&searcher, start).map(|(_, b)| b)
    });
}
