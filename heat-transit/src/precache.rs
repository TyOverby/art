use astar::astar;
use fnv::FnvHashMap as HashMap;
use model::*;
use pathing::*;
use serde_json::{from_reader, to_writer_pretty};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub type RouteCache = HashMap<StopId, f64>;

fn build_cache(stops: &Stops, connections: &PreConnections, destination: Position) -> RouteCache {
    let mut searcher = TransitSearchProblem {
        stops: &stops,
        connections: &connections,
        start: destination,
        end: destination,
        precache: HashMap::default(),
    };

    let total = stops.len();
    for (i, (id, _)) in stops.iter().enumerate() {
        searcher.start = Position::BusStop(*id, HowGet::Walk);
        let (_, total_cost) = astar(&mut searcher).unwrap();
        searcher.precache.insert(*id, total_cost);
        println!("{} / {}", i, total);
    }

    let file = BufWriter::new(File::create("cache/precache.json").unwrap());
    to_writer_pretty(file, &searcher.precache).unwrap();
    searcher.precache
}

fn read_cache() -> Result<RouteCache, Box<Error>> {
    let file = BufReader::new(File::open("cache/precache.json")?);
    Ok(from_reader(file)?)
}

pub fn get_cache(stops: &Stops, connections: &PreConnections, destination: Position) -> RouteCache {
    read_cache().unwrap_or_else(|_| build_cache(stops, connections, destination))
}
