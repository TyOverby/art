extern crate csv;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate vectorphile;

use std::error::Error;
use std::io;
use std::fs::File;
use vectorphile::Canvas;
use vectorphile::svg::SvgBackend;

#[derive(Debug, Deserialize)]
struct Star {
    id: String,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    z: f64,
    mag: f64,
    dist: f64,
}

struct Info {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
    max_dist: f64,
    max_vel: f64,
    max_mag: f64,
}

fn get_stars() -> Result<(Vec<Star>, Info), Box<Error>> {
    let mut out = Vec::with_capacity(12000);
    let mut rdr = csv::Reader::from_reader(io::stdin());
    let mut min_x = std::f64::INFINITY;
    let mut min_y = std::f64::INFINITY;
    let mut max_x = std::f64::NEG_INFINITY;
    let mut max_y = std::f64::NEG_INFINITY;
    let mut max_dist = std::f64::NEG_INFINITY;
    let mut max_vel = std::f64::NEG_INFINITY;
    let mut max_mag = std::f64::NEG_INFINITY;

    for result in rdr.deserialize() {
        let star: Star = result?;
        if star.dist > 28571.0 {
            continue;
        }
        min_x = min_x.min(star.x);
        min_y = min_y.min(star.y);
        max_x = max_x.max(star.x);
        max_y = max_y.max(star.y);
        max_dist = max_dist.max(star.dist);
        max_vel = max_vel.max(star.vx).max(star.vy);
        max_mag = max_mag.max(star.mag);

        out.push(star);
    }
    Ok((
        out,
        Info {
            min_x,
            max_x,
            min_y,
            max_y,
            max_dist,
            max_vel,
            max_mag,
        },
    ))
}

const SIZE: f64 = 1000.0;
const VEL_CLAMP: f64 = 100.0;
const VEL_MIN: f64 = 3.5;
const MAG_SCAL: f64 = 10.0;

fn main() {
    let (
        mut stars,
        Info {
            min_x,
            max_x,
            min_y,
            max_y,
            max_dist,
            max_vel,
            max_mag,
        },
    ) = get_stars().unwrap();
    let dx = max_x - min_x;
    let dy = max_y - min_y;
    for star in &mut stars {
        star.x -= min_x;
        star.y -= min_y;
        star.x /= dx;
        star.y /= dy;
        star.x *= SIZE;
        star.y *= SIZE;

        star.vx /= max_vel;
        star.vy /= max_vel;
        star.vx *= VEL_CLAMP;
        star.vy *= VEL_CLAMP;

        star.mag /= max_mag;
        star.mag *=  MAG_SCAL;
    }

    let file = File::create("./out.svg").unwrap();
    let mut canvas = Canvas::new(SvgBackend::new_with_bb(file, 0.0, 0.0, SIZE, SIZE).unwrap());
    for star in stars
        .iter()
        .filter(|star| {
            !(star.x.is_nan() || star.y.is_nan() || star.vx.is_nan() || star.vy.is_nan()
                || star.dist.is_nan())
        })
        .filter(|star| star.dist > max_dist / 20.0)
        .filter(|star| star.dist < 10000000.0)
        .filter(|star| {
            (star.vx * star.vx + star.vy * star.vy).sqrt() > VEL_MIN
        }) {
            println!("{}", star.mag);
        canvas
            .draw_line(
                (star.x, star.y),
                (
                    star.x + star.mag * star.mag,
                    star.y + star.mag * star.mag,
                ),
                None,
            )
            .unwrap();
    }

    canvas.close().unwrap();
}
