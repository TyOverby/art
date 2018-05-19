use csv;
use spade::rtree::RTree;
use std::collections::HashMap;
use std::error::Error;

const ORIGIN_LAT: f64 = 47.6;
const ORIGIN_LON: f64 = -122.33;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct StopId(u32);

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct RawStop {
    stop_id: u32,
    stop_name: String,
    stop_lat: f64,
    stop_lon: f64,
    zone_id: u32,
    stop_timezone: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct RawStopTime {
    trip_id: u32,
    stop_id: u32,
    arrival_time: String,
    departure_time: String,
    stop_sequence: u32,
    stop_headsign: String,
    shape_dist_traveled: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct Stop {
    stop_id: StopId,
    // Distances in km
    stop_x: f64,
    stop_y: f64,
}

#[derive(Clone)]
struct Routes {
    routes: HashMap<RouteId, RouteInfo>,
}

struct Stops {
    stops: HashMap<StopId, Stop>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RouteId(u32);

impl RouteId {
    fn new(start: StopId, end: StopId) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RouteInfo {
    time: u32,
    trip_id: u32,
    stops: Vec<StopId>,
}

fn read() -> (Stops, Routes) {
    print!("Reading...");
    let mut stops = HashMap::new();
    for result in csv::Reader::from_path("stops.txt")
        .unwrap()
        .into_deserialize()
    {
        let result: RawStop = result.unwrap();
        stops.insert(StopId(result.stop_id), Stop::new(result));
    }
    let mut stop_times = Vec::new();
    for result in csv::Reader::from_path("stop_times.txt")
        .unwrap()
        .into_deserialize()
        .map(Result::unwrap)
    {
        stop_times.push(result);
    }
    println!(" done.");

    let routes = build_routes(&stops, &stop_times);
    unimplemented!();
}

fn build_routes(stops: &HashMap<StopId, Stop>, times: &[RawStopTime]) -> Routes {
    let mut group_by_trip_id = HashMap::new();
    for stop in times {
        group_by_trip_id
            .entry(stop.trip_id)
            .or_insert_with(|| Vec::new())
            .push(stop.clone());
    }

    let mut route = HashMap::new();

    for (trip_id, stops) in group_by_trip_id {
    }

    unimplemented!();
}

fn parse_time(time: &str) -> u64 {
    let mut split = time.split(':');
    let hours = split
        .next()
        .expect("Missing hours in time")
        .parse::<u64>()
        .expect("Hours invalid format");
    let minutes = split
        .next()
        .expect("Missing minutes in time")
        .parse::<u64>()
        .expect("Minutes invalid format");
    let seconds = split
        .next()
        .expect("Missing seconds in time")
        .parse::<u64>()
        .expect("Seconds invalid format");
    assert!(split.next().is_none());
    hours * (60 * 60) + minutes * 60 + seconds
}

impl Stop {
    fn new(raw_stop: RawStop) -> Self {
        // Convert lat/lon to km around an origin, assuming locally flat earth.
        let delta_lat = raw_stop.stop_lat - ORIGIN_LAT;
        let delta_lon = raw_stop.stop_lon - ORIGIN_LON;
        let skew = (ORIGIN_LAT * (::std::f64::consts::PI / 180.0)).cos(); // approx. 1.0/1.48
        let x = delta_lon * skew * (40_075.0 / 360.0);
        let y = delta_lat * (40_075.0 / 360.0);
        Stop {
            stop_id: StopId(raw_stop.stop_id),
            stop_x: x,
            stop_y: y,
        }
    }

    fn fake(x: f64, y: f64) -> Self {
        Self {
            stop_id: StopId(0),
            stop_x: x,
            stop_y: y,
        }
    }
}

impl Stops {
    fn new(stops: HashMap<StopId, Stop>) -> Self {
        Self { stops }
    }
}
