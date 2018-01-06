extern crate osm_xml;
extern crate proj5;

use proj5::prelude::*;
use proj5::FromLonLat;
use osm_xml::{Member, Reference, Tag, Way, OSM};
use std::fs::File;
use std::io::BufReader;

const target_h: f64 = 1000.0f64;

fn filter(tag: &Tag) -> bool {
    if tag.key == "building" {
        return true;
    }
    if tag.val == "pier" {
        return true;
    }
    if tag.val == "riverbank" {
        return true;
    }

    return false;
}

fn convert(input: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let ellipsoid = WGS_1984_ELLIPSOID;
    //let system = UTMSystem { utm_zone: 10 };
    let system =  MercatorSystem;

    let mut strategy = MultithreadingStrategy::SingleCore;
    let out = system.from_lon_lat(input, &ellipsoid, &mut strategy);
    out.data
}
fn print_path(path: Vec<(f64, f64)>) {
    print!(r#"<path style="fill:none; stroke:black;" d=""#);
    let mut first = true;
    for (lon, lat) in path {
        let movement = if first { "M" } else { "L" };
        first = false;
        print!("{}{},{} ", movement, lon, target_h - lat);
    }
    println!(r#"" />"#);
}

fn main() {
    let f = File::open("C:\\Users\\Ty\\Downloads\\map.osm").unwrap();
    let br = BufReader::new(f);
    let doc = OSM::parse(br).unwrap();

    let bounds = doc.bounds.unwrap();

    let bounds_converted = convert(vec![
        (bounds.maxlon, bounds.maxlat),
        (bounds.minlon, bounds.minlat),
    ]);
    let (b_max_lon, b_max_lat) = bounds_converted[0];
    let (b_min_lon, b_min_lat) = bounds_converted[1];
    let target_w = ((b_max_lon - b_min_lon) / (b_max_lat - b_min_lat)) * target_h;
    let scale_x = target_w / (b_max_lon - b_min_lon);
    let scale_y = target_h / (b_max_lat - b_min_lat);

    println!(r#"<svg viewBox="0 0 {} {} " xmlns="http://www.w3.org/2000/svg">"#, target_w, target_h);

    let print_way = |way: &Way, always_print: bool| {
        if !always_print && !way.tags.iter().any(filter) {
            return;
        }
        let mut coords = vec![];

        for node in &way.nodes {
            let node = doc.resolve_reference(node);
            if let Reference::Node(node) = node {
                coords.push((node.lon, node.lat));
            }
        }

        let out = convert(coords);
        let scaled = out.into_iter().map(|(lon, lat)| {
            ((lon - b_min_lon) * scale_x, (lat - b_min_lat) * scale_y)
        });
        print_path(scaled.collect());
    };

    for rel in &doc.relations {
        if !rel.tags.iter().any(filter) {
            continue;
        }
        for member in &rel.members {
            if let &Member::Way(ref reference, _) = member {
                let member = doc.resolve_reference(reference);
                if let Reference::Way(way) = member {
                    print_way(way, true);
                }
            }
        }
    }

    for way in &doc.ways {
        print_way(way, false);
    }
    println!("</svg>");
}
