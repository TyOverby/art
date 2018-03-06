extern crate euclid;
extern crate rand;
extern crate vectorphile;

use std::fs::File;
use std::io::BufWriter;
use vectorphile::Canvas;
use vectorphile::backend::{DrawBackend, DrawOptions};
use euclid::{UnknownUnit, point2, vec2};

const RADIUS: f32 = 100.0;
const GAP: f32 = RADIUS / 2.0;
const NUM_AUX: u32 = 4;
const X_COUNT: u32 = 5;
const Y_COUNT: u32 = 5;
const DASH_DIST: f32 = 10.0;
const LINE_GAP: f32 = 3.0;
const BIG_PATTERN: &[LineStyle] = &[Full, Dashed, Full, Blank, Full, Dashed, Full];
const SMALL_PATTERN: &[LineStyle] = &[Full, Full, Dashed, Full, Full];

#[derive(Clone, Copy)]
enum LineStyle {
    Blank,
    Dashed,
    Full,
}
use LineStyle::*;

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

fn draw_dashed<B: DrawBackend>(
    p1: (f32, f32),
    p2: (f32, f32),
    options: DrawOptions,
    canvas: &mut Canvas<B>,
) -> Result<(), B::Error> {
    if dist_between_points(p1, p2) < DASH_DIST {
        return Ok(());
    }

    let p1 = point2::<_, UnknownUnit>(p1.0, p1.1);
    let p2 = point2::<_, UnknownUnit>(p2.0, p2.1);
    let v = p2 - p1;
    let v = v.normalize() * DASH_DIST;
    let p2_extended = p1 + v;
    let p2_out = p1 + v * 2.0;

    canvas.draw_line((p1.x, p1.y), (p2_extended.x, p2_extended.y), Some(options))?;
    draw_dashed((p2_out.x, p2_out.y), (p2.x, p2.y), options, canvas)
}

fn draw_line<B: DrawBackend>(
    (p1, p2): ((f32, f32), (f32, f32)),
    style: &[LineStyle],
    options: DrawOptions,
    canvas: &mut Canvas<B>,
    drawn_lines: &mut Vec<((f32, f32), (f32, f32))>,
) -> Result<(), B::Error> {
    let twist = |a, b| if style.len() % 2 == 0 { (a, b) } else { (b, a) };

    let current_style = if style.len() == 0 {
        return Ok(());
    } else {
        style[0]
    };

    let mut collision = None;
    let mut shortest_dist = ::std::f32::INFINITY;
    for prev_drawn in &drawn_lines[..] {
        if let Some(intersection) = get_line_intersection((p1, p2), *prev_drawn) {
            let dst = dist_between_points(p1, intersection);
            if dst < shortest_dist {
                collision = Some(intersection);
                shortest_dist = dst;
            }
        }
    }

    if let Some(intersection_point) = collision {
        let (a, b) = twist(p1, intersection_point);
        match current_style {
            Blank => {}
            Dashed => draw_dashed(a, b, options, canvas)?,
            Full => canvas.draw_line(a, b, Some(options))?,
        }
        drawn_lines.push((a, b));
    } else {
        let (a, b) = twist(p1, p2);
        match current_style {
            Blank => {}
            Dashed => draw_dashed(a, b, options, canvas)?,
            Full => canvas.draw_line(a, b, Some(options))?,
        }
        drawn_lines.push((a, b));
    }

    draw_line(
        dupe_line_next((p1, p2)),
        &style[1..],
        options,
        canvas,
        drawn_lines,
    )?;
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

    let mut drawn_lines = vec![];

    draw_line(
        (first, second),
        BIG_PATTERN,
        draw_options,
        canvas,
        &mut drawn_lines,
    )?;

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
                    SMALL_PATTERN,
                    draw_options,
                    canvas,
                    &mut drawn_lines,
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
