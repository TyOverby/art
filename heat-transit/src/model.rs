use csv;
use fnv::FnvHashMap as HashMap;
use serde_json::{from_reader, to_writer_pretty};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

const ORIGIN_LAT: f64 = 47.6;
const ORIGIN_LON: f64 = -122.33;

#[derive(Clone, Deserialize, Serialize, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StopId(pub u32);

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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Stop {
    pub stop_id: StopId,
    // Distances in km
    pub stop_x: f64,
    pub stop_y: f64,
    pub name: String,
}

type Connections = HashMap<(StopId, StopId), Connection>;
pub type PreConnections = HashMap<StopId, HashMap<StopId, Connection>>;
pub type Stops = HashMap<StopId, Stop>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RouteId(u32);

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Connection {
    pub time: f64,
    trip_id: u32,
}

fn translate_connections(c: Connections) -> PreConnections {
    let mut out = HashMap::default();
    for ((start, stop), info) in c {
        if !out.contains_key(&start) {
            out.insert(start, HashMap::default());
        }
        out.get_mut(&start).unwrap().insert(stop, info);
    }
    out
}

pub fn get_connections() -> (Stops, PreConnections) {
    read_connections().unwrap_or_else(|_| build_connections())
}

fn read_connections() -> Result<(Stops, PreConnections), Box<Error>> {
    let file = BufReader::new(File::open("cache/stops.json")?);
    let stops = from_reader(file)?;

    let file = BufReader::new(File::open("cache/connections.json")?);
    let connections = from_reader(file)?;

    Ok((stops, connections))
}

fn build_connections() -> (Stops, PreConnections) {
    let mut stops = HashMap::default();
    for result in csv::Reader::from_path("data/stops.txt")
        .unwrap()
        .into_deserialize()
    {
        let result: RawStop = result.unwrap();
        stops.insert(StopId(result.stop_id), Stop::new(result));
    }
    let mut stop_times = Vec::new();
    for result in csv::Reader::from_path("data/stop_times.txt")
        .unwrap()
        .into_deserialize()
        .map(Result::unwrap)
    {
        stop_times.push(result);
    }

    let connections = translate_connections(build_routes(&stop_times));

    {
        let file = BufWriter::new(File::create("cache/stops.json").unwrap());
        to_writer_pretty(file, &stops).unwrap();
    }
    {
        let file = BufWriter::new(File::create("cache/connections.json").unwrap());
        to_writer_pretty(file, &connections).unwrap();
    }

    (stops, connections)
}

fn build_routes(times: &[RawStopTime]) -> Connections {
    use std::collections::hash_map::Entry::*;
    let mut group_by_trip_id = HashMap::default();
    for stop in times {
        group_by_trip_id
            .entry(stop.trip_id)
            .or_insert_with(|| Vec::new())
            .push(stop.clone());
    }

    let mut result: Connections = HashMap::default();

    for (&trip_id, trip) in &group_by_trip_id {
        for stop_one in trip {
            for stop_two in trip {
                if stop_one == stop_two {
                    continue;
                }

                let stop_one_time = parse_time(&stop_one.arrival_time);
                let stop_two_time = parse_time(&stop_two.arrival_time);
                if stop_one.shape_dist_traveled >= stop_two.shape_dist_traveled
                    || stop_one_time >= stop_two_time
                {
                    continue;
                }

                // Time can skip over midnight and go "backwards". Ignore those entries.
                let time = stop_two_time - stop_one_time;
                // this overwites a bunch (since there's many times per trip)
                let route_info = Connection {
                    time: time as f64,
                    trip_id: trip_id,
                };

                match result.entry((StopId(stop_one.stop_id), StopId(stop_two.stop_id))) {
                    Occupied(ref mut entry) => {
                        let entry = entry.get_mut();
                        if entry.time > time as f64 {
                            *entry = route_info;
                        }
                    }
                    Vacant(entry) => {
                        entry.insert(route_info);
                    }
                }
            }
        }
    }

    println!("done building routes");
    return result;
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

pub fn lat_lon_to_x_y(lat: f64, lon: f64) -> (f64, f64) {
    let delta_lat = lat - ORIGIN_LAT;
    let delta_lon = lon - ORIGIN_LON;
    let skew = (ORIGIN_LAT * (::std::f64::consts::PI / 180.0)).cos(); // approx. 1.0/1.48
    let x = delta_lon * skew * (40_075.0 / 360.0);
    let y = delta_lat * (40_075.0 / 360.0);
    (x, y)
}

impl Stop {
    fn new(raw_stop: RawStop) -> Self {
        // Convert lat/lon to km around an origin, assuming locally flat earth.
        let (x, y) = lat_lon_to_x_y(raw_stop.stop_lat, raw_stop.stop_lon);
        Stop {
            stop_id: StopId(raw_stop.stop_id),
            stop_x: x,
            stop_y: y,
            name: raw_stop.stop_name,
        }
    }
}
