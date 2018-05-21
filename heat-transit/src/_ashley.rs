use spade::rtree::RTree;
use spade::PointN;
use std::collections::{hash_map, HashMap};
use std::error::Error;

// distances in km
const START_RADIUS: f64 = 0.5;
const TRANSFER_RADIUS: f64 = 0.5;
const END_RADIUS: f64 = 0.5; // TY: This is the "dot size"
                             // km/second
const WALKING_SPEED: f64 = 0.0014;
const BUS_WAIT_TIME: f64 = 7.5 * 60.0; // in seconds
const ORIGIN_LAT: f64 = 47.6;
const ORIGIN_LON: f64 = -122.33;
const RENDER_SIZE: f64 = 15.0;
const IMPOSSIBLE_ROUTE_WEIGHT: f64 = 60.0 * 60.0;
// https://www.google.com/maps/@47.6,-122.33,12z


impl PointN for Stop {
    type Scalar = f64;
    fn dimensions() -> usize {
        2
    }
    fn from_value(value: Self::Scalar) -> Self {
        Self::fake(value, value)
    }
    fn nth(&self, index: usize) -> &Self::Scalar {
        if index == 0 {
            &self.stop_x
        } else {
            &self.stop_y
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        if index == 0 {
            &mut self.stop_x
        } else {
            &mut self.stop_y
        }
    }
}



fn build_routes(stop_times: &Vec<StopTime>) -> Routes {
    print!("Building routes (grouping)... ");
    let mut group_by_trip_id = HashMap::new();
    for stop in stop_times {
        group_by_trip_id
            .entry(stop.trip_id)
            .or_insert_with(|| Vec::new())
            .push(stop.clone());
    }
    print!("calculating...");
    let mut result: HashMap<RouteId, RouteInfo> = HashMap::new();
    for (_, trip) in &group_by_trip_id {
        for stop_one in trip {
            for stop_two in trip {
                let stop_one_time = parse_time(&stop_one.arrival_time);
                let stop_two_time = parse_time(&stop_two.arrival_time);
                // Time can skip over midnight and go "backwards". Ignore those entries.
                if stop_one.shape_dist_traveled < stop_two.shape_dist_traveled
                    && stop_one_time < stop_two_time
                {
                    let time = stop_two_time - stop_one_time;
                    // this overwites a bunch (since there's many times per trip)
                    let route_info = RouteInfo {
                        time: time,
                        trip_id: stop_one.trip_id,
                    };
                    let key = RouteId::new(StopId(stop_one.stop_id), StopId(stop_two.stop_id));
                    match result.entry(key) {
                        hash_map::Entry::Occupied(ref mut entry) => {
                            let entry = entry.get_mut();
                            if entry.time > time {
                                *entry = route_info;
                            }
                        }
                        hash_map::Entry::Vacant(entry) => {
                            entry.insert(route_info);
                        }
                    }
                }
            }
        }
    }
    println!(" done.");
    Routes { routes: result }
}

fn build_transfers(stops: &Stops) -> Routes {
    let mut result = HashMap::new();
    for (_, transfer_from) in &stops.stops {
        let transfer_to = stops.rtree.lookup_in_circle(
            &Stop::fake(transfer_from.stop_x, transfer_from.stop_y),
            &(TRANSFER_RADIUS * TRANSFER_RADIUS),
        );
        for transfer_to in transfer_to {
            if transfer_from.stop_id == transfer_to.stop_id {
                continue;
            }
            let distance = (
                transfer_from.stop_x - transfer_to.stop_x,
                transfer_from.stop_y - transfer_to.stop_y,
            );
            let distance = (distance.0 * distance.0 + distance.1 * distance.1).sqrt();
            let walk_and_wait_time = distance / WALKING_SPEED + BUS_WAIT_TIME;
            let walk_and_wait_time = walk_and_wait_time as u32;
            result.insert(
                RouteId::new(transfer_from.stop_id.clone(), transfer_to.stop_id.clone()),
                RouteInfo {
                    time: walk_and_wait_time,
                    trip_id: 0,
                },
            );
        }
    }
    Routes { routes: result }
}

fn construct_whole_map(routes: &mut Routes) {
    println!("Constructing transitive stuff and things");
    loop {
        let mut start_to_route = HashMap::new();
        for (key, value) in &routes.routes {
            start_to_route
                .entry(key.start.clone())
                .or_insert_with(|| Vec::new())
                .push((key.clone(), value.clone()));
        }
        println!(".");
        let old_len = routes.routes.len();
        for (route, info) in routes
            .routes
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>()
        {
            if let Some(second_route) = start_to_route.get(&route.end) {
                for second_route in second_route {
                    let combo_route = RouteId::new(route.start.clone(), second_route.0.end.clone());
                    let combo_route_info = RouteInfo {
                        time: info.time + second_route.1.time,
                        trip_id: 0,
                    };
                    match routes.routes.entry(combo_route) {
                        hash_map::Entry::Occupied(ref mut existing)
                            if combo_route_info.time < existing.get().time =>
                        {
                            existing.insert(combo_route_info);
                        }
                        hash_map::Entry::Occupied(_) => (),
                        hash_map::Entry::Vacant(vacant) => {
                            vacant.insert(combo_route_info);
                        }
                    }
                }
            }
        }
        let new_len = routes.routes.len();
        println!("{} -> {}", old_len, new_len);
        // Loop to calculate all transfers. Only do one loop iter to do one transfer.
        //if old_len == new_len {
        break;
        //}
    }
    println!("Done.");
}

fn plan_trip_rtree(
    routes: &Routes,
    stops: &Stops,
    from_x: f64,
    from_y: f64,
    to_x: f64,
    to_y: f64,
) -> Option<f64> {
    let close_from = stops
        .rtree
        .lookup_in_circle(&Stop::fake(from_x, from_y), &(START_RADIUS * START_RADIUS));
    let close_to = stops
        .rtree
        .lookup_in_circle(&Stop::fake(to_x, to_y), &(END_RADIUS * END_RADIUS));
    let default = u32::max_value();
    let mut min_time = default;
    for stop_from in &close_from {
        for stop_to in &close_to {
            let diff_from = (stop_from.stop_x - from_x, stop_from.stop_y - from_y);
            let diff_to = (stop_to.stop_x - to_x, stop_to.stop_y - to_y);
            let dist_from = (diff_from.0 * diff_from.0 + diff_from.1 * diff_from.1).sqrt();
            let dist_to = (diff_to.0 * diff_to.0 + diff_to.1 * diff_to.1).sqrt();
            let walk_dist = dist_from + dist_to;
            let walk_and_wait_time = walk_dist / WALKING_SPEED + BUS_WAIT_TIME;
            let walk_and_wait_time = walk_and_wait_time as u32;
            if let Some(route_info) = routes.routes.get(&RouteId::new(
                stop_from.stop_id.clone(),
                stop_to.stop_id.clone(),
            )) {
                let total_time = route_info.time + walk_and_wait_time;
                if total_time < min_time {
                    min_time = total_time;
                }
            }
        }
    }
    if min_time == default {
        None
    } else {
        Some(min_time as f64)
    }
}

fn shenanigans_build(routes: &Routes, stops: &Stops) -> HashMap<StopId, f64> {
    let mut result = HashMap::new();
    for (stop, _) in &stops.stops {
        let mut sum = 0usize;
        let mut count = 0usize;
        for (other_stop, _) in &stops.stops {
            if let Some(x) = routes
                .routes
                .get(&RouteId::new(stop.clone(), other_stop.clone()))
            {
                sum += x.time as usize;
                count += 1;
            } else {
                sum += IMPOSSIBLE_ROUTE_WEIGHT as usize;
                count += 1;
            }
        }
        result.insert(stop.clone(), sum as f64 / count as f64);
    }
    result
}

fn shenanigans(
    stops: &Stops,
    stop_score: &HashMap<StopId, f64>,
    from_x: f64,
    from_y: f64,
) -> Option<f64> {
    stops
        .rtree
        .lookup_in_circle(&Stop::fake(from_x, from_y), &1.0)
        .iter()
        .filter_map(|x| {
            let diff = (x.stop_x - from_x, x.stop_y - from_y);
            let dist = (diff.0 * diff.0 + diff.1 * diff.1).sqrt();
            if dist < 0.5 {
                Some((x, dist))
            } else {
                None
            }
        })
        .map(|(neigh, dist)| *stop_score.get(&neigh.stop_id).unwrap() + dist / WALKING_SPEED)
        .min_by(|x, y| x.partial_cmp(y).unwrap())
}

fn render(routes: &Routes, stops: &Stops, width: u32, height: u32) -> Vec<Option<f64>> {
    let center_x = 0.0;
    let center_y = 0.0;
    let shenanigans_stuff = shenanigans_build(routes, stops);
    println!("Rendering...");
    let mut result = Vec::new();
    for y in 0..height {
        if (height - y) % (1 << 4) == 0 {
            println!("{}", height - y);
        }
        for x in 0..width {
            let dest_x = (x as f64 / width as f64 - 0.5) * (2.0 * RENDER_SIZE);
            // invert - zero is top left in image coords, bottom left in geo coords
            let dest_y = (y as f64 / height as f64 - 0.5) * (2.0 * RENDER_SIZE * -1.0);
            // TY: plan_trip_rtree is distance from a single point. shenanigans is the "distance from all to all points" thingy.
            //let planned = plan_trip_rtree(routes, stops, center_x, center_y, dest_x, dest_y);
            let planned = shenanigans(stops, &shenanigans_stuff, dest_x, dest_y);
            result.push(planned);
        }
    }
    println!("done.");
    result
}

fn normalize(data: &mut Vec<Option<f64>>) -> Vec<f32> {
    print!("Normalizing...");
    // TY: This is log2 vs. linear choice.
    //for datum in data.iter_mut() {
    //    *datum = datum.map(|x| x.log2());
    //}
    let min = data.iter()
        .cloned()
        .filter_map(|x| x)
        .fold(1.0 / 0.0, f64::min);
    let max = data.iter()
        .cloned()
        .filter_map(|x| x)
        .fold(-1.0 / 0.0, f64::max);
    let result = data.iter()
        .map(|x| x.map(|x| (x - min) / (max - min)).unwrap_or(0.0) as f32)
        .collect();
    println!(" done.");
    result
}

fn save_image(data: &[u8], width: u32, height: u32, path: &str) -> Result<(), Box<Error>> {
    print!("Saving...");
    use png::HasParameters;
    let file = ::std::fs::File::create(path)?;
    let w = &mut ::std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;
    println!(" done.");
    Ok(())
}

fn do_image(routes: &Routes, stops: &Stops) -> Result<(), Box<Error>> {
    let width = 1000;
    let height = 1000;
    let asdf = normalize(&mut render(routes, stops, width, height));
    let mut pixels = Vec::with_capacity(asdf.len() * 4);
    for pix in asdf {
        let value = (pix * 255.0) as u8;
        pixels.push(value);
        pixels.push(value);
        pixels.push(value);
        pixels.push(255);
    }
    save_image(&pixels, width, height, "image.png")
}

/*
fn main() {
    let (stops, mut routes) = read().expect("Reading failed");

    construct_whole_map(&mut routes);

    do_image(&routes, &stops).expect("Writing failed");
}
*/
