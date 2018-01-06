extern crate vecmath;
extern crate vectorphile;
extern crate rand;

mod geom;

use std::fs::File;
use std::io::Error;
use vectorphile::{Canvas};
use vectorphile::svg::SvgBackend;
use geom::{Line, Point, Ray};

const size: f32 = 500.0;
const factor: f32 = 1.0;
const count: usize = 1000;
const offset: f32 = 0.0;

fn random_point() -> geom::Point {
    Point{x: rand::random::<f32>() * size, y: rand::random::<f32>() * size}
}

fn random_point2() -> geom::Point {
    Point{x: rand::random::<f32>() * size, y: rand::random::<f32>() * size * factor}
}

fn draw() -> Result<(), Error> {
    let mut boundaries: Vec<geom::Line> = vec![];
    let mut to_draw: Vec<geom::Line> = vec![];

    boundaries.push(Line(Point{x: 0.0, y: 0.0}, Point{x: 0.0, y:size}));
    boundaries.push(Line(Point{x: 0.0, y: 0.0}, Point{x: size, y:0.0}));
    boundaries.push(Line(Point{x: size, y: 0.0}, Point{x: size, y:size}));
    boundaries.push(Line(Point{x: 0.0, y: size}, Point{x: size, y:size}));
    to_draw.extend(boundaries.iter().cloned());

    for _ in 0 .. count {
        let p1 = random_point();
        let p2 = random_point2();
        let v = (p1 - p2).normalized();
        let (p1_boundary, p1_draw) = {
            let d = 1.0;
            let v = v * d;
            let r = Ray(p1, v);
            let mut d = std::f32::INFINITY;
            for line in &boundaries {
                if let Some(p) = r.intersect_with_line(line) {
                    d = d.min((p-p1).magnitude());
                }
            }
            (p1 + v.normalized() * d,
             p1 + v.normalized() * (d - offset))
        };
        let (p2_boundary, p2_draw) = {
            let d = -1.0;
            let v = v * d;
            let r = Ray(p1, v);
            let mut d = std::f32::INFINITY;
            for line in &boundaries {
                if let Some(p) = r.intersect_with_line(line) {
                    d = d.min((p-p1).magnitude());
                }
            }
            (p1 + v.normalized() * d,
             p1 + v.normalized() * (d - offset))
        };

        boundaries.push(Line(p1_boundary, p2_boundary));
        to_draw.push(Line(p1_draw, p2_draw));
    }



    let file = File::create("./out.svg").unwrap();
    let mut canvas = Canvas::new(SvgBackend::new(file)?);

    for Line(p1, p2) in to_draw {
        canvas.draw_line(p1.into_tuple(), p2.into_tuple())?;
    }

    canvas.close()?;
    Ok(())
}

fn main() {
    draw().unwrap();
}
