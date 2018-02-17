extern crate euclid;
extern crate rand;
extern crate vectorphile;

use std::fs::File;
use std::io::BufWriter;
use vectorphile::Canvas;
use vectorphile::backend::{DrawBackend, DrawOptions};
use euclid::{UnknownUnit, vec2};

const RADIUS: f32 = 100.0;
const GAP: f32 = RADIUS / 2.0;
const NUM_AUX: u32 = 3;
const X_COUNT: u32 = 5;
const Y_COUNT: u32 = 5;
const LINE_GAP: f32 = 3.0;
const BIG_COUNT: u32 = 4;
const SMALL_COUNT: u32 = 2;

fn random_point_on_circle(radius: f32, x: f32, y: f32) -> (f32, f32) {
    let angle: f32 = rand::random::<f32>() * 3.14159 * 2.0;
    (x + angle.cos() * radius, y + angle.sin() * radius)
}

fn dist_between_points((p1x, p1y): (f32, f32), (p2x, p2y): (f32, f32)) -> f32 {
    let dx = p1x - p2x;
    let dy = p1y - p2y;
    (dx * dx + dy * dy).sqrt()
}

fn get_line_intersection(
    ((p0_x, p0_y), (p1_x, p1_y)): ((f32, f32), (f32, f32)),
    ((p2_x, p2_y), (p3_x, p3_y)): ((f32, f32), (f32, f32)),
) -> Option<(f32, f32)> {
    let i_x;
    let i_y;
    let s1_x;
    let s1_y;
    let s2_x;
    let s2_y;
    s1_x = p1_x - p0_x;
    s1_y = p1_y - p0_y;
    s2_x = p3_x - p2_x;
    s2_y = p3_y - p2_y;

    let s;
    let t;
    s = (-s1_y * (p0_x - p2_x) + s1_x * (p0_y - p2_y)) / (-s2_x * s1_y + s1_x * s2_y);
    t = (s2_x * (p0_y - p2_y) - s2_y * (p0_x - p2_x)) / (-s2_x * s1_y + s1_x * s2_y);

    if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
        i_x = p0_x + (t * s1_x);
        i_y = p0_y + (t * s1_y);
        return Some((i_x, i_y));
    }

    return None;
}

fn dupe_line_next(((x1, y1), (x2, y2)): ((f32, f32), (f32, f32))) -> ((f32, f32), (f32, f32)) {
    let lv = vec2::<_, UnknownUnit>(x2 - x1, y2 - y1);
    let lv = vec2::<_, UnknownUnit>(-lv.y, lv.x);
    let lv = lv.normalize() * LINE_GAP;
    ((x1 + lv.x, y1 + lv.y), (x2 + lv.x, y2 + lv.y))
}

fn draw_line<B: DrawBackend>(
    (p1, p2): ((f32, f32), (f32, f32)),
    count: u32,
    options: DrawOptions,
    collide: Option<((f32, f32), (f32, f32))>,
    canvas: &mut Canvas<B>,
) -> Result<(), B::Error> {
    let twist = |a, b| if count % 2 == 0 { (a, b) } else { (b, a) };
    let collision_left = collide.and_then(|c_target| get_line_intersection(c_target, (p1, p2)));
    let collision_right = collide
        .map(dupe_line_next)
        .map(dupe_line_next)
        .map(dupe_line_next)
        .map(dupe_line_next)
        .and_then(|c_target| get_line_intersection(c_target, (p1, p2)));

    let collision = match (collision_left, collision_right) {
        (Some(l), Some(r)) => if dist_between_points(p1, l) < dist_between_points(p1, r) {
            Some(l)
        } else {
            Some(r)
        },
        (a @ Some(_), None) => a,
        (None, b @ Some(_)) => b,
        _ => None,
    };

    if let Some(intersection_point) = collision {
        let (a, b) = twist(p1, intersection_point);
        canvas.draw_line(a, b, Some(options))?;
    } else {
        let (a, b) = twist(p1, p2);
        canvas.draw_line(a, b, Some(options))?;
    }

    if count != 0 {
        draw_line(
            dupe_line_next((p1, p2)),
            count - 1,
            options,
            collide,
            canvas,
        )?;
    }
    Ok(())
}

fn generate_intersection<B: DrawBackend>(
    x: f32,
    y: f32,
    canvas: &mut Canvas<B>,
) -> Result<(), B::Error> {
    let draw_options = DrawOptions::stroked((0, 0, 0), 1.0);
    let first = random_point_on_circle(RADIUS, x, y);
    let mut second = random_point_on_circle(RADIUS, x, y);
    loop {
        if dist_between_points(first, second) > RADIUS {
            break;
        }
        second = random_point_on_circle(RADIUS, x, y);
    }

    draw_line((first, second), BIG_COUNT, draw_options, None, canvas)?;

    let mut num_found = 0;
    loop {
        if num_found >= NUM_AUX {
            break;
        }

        let p1 = random_point_on_circle(RADIUS, x, y);
        let p2 = random_point_on_circle(RADIUS, x, y);
        if let Some(intersection_point) = get_line_intersection((first, second), (p1, p2)) {
            if dist_between_points(p1, intersection_point) > RADIUS / 2.0 {
                num_found += 1;
                draw_line(
                    (p1, p2),
                    SMALL_COUNT,
                    draw_options,
                    Some((first, second)),
                    canvas,
                )?;
            }
        }
    }
    Ok(())
}

fn main() {
    let writer = BufWriter::new(File::create("out.svg").unwrap());
    let mut canvas = vectorphile::Canvas::new(
        vectorphile::svg::SvgBackend::new_with_bb(
            writer,
            -RADIUS as f64,
            -RADIUS as f64,
            (RADIUS as f64 * 2.0 + GAP as f64) * X_COUNT as f64,
            (RADIUS as f64 * 2.0 + GAP as f64) * Y_COUNT as f64,
        ).unwrap(),
    );

    for i in 0..X_COUNT {
        for k in 0..Y_COUNT {
            generate_intersection(
                i as f32 * (RADIUS * 2.0 + GAP),
                k as f32 * (RADIUS * 2.0 + GAP),
                &mut canvas,
            ).unwrap();
        }
    }

    canvas.close().unwrap();
}
