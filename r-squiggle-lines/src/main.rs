extern crate rand;
extern crate vectorphile;

use vectorphile::{svg, Canvas};
use std::fs::File;
use std::io::BufWriter;
use rand::Rng;

const COUNT_DOWN: usize = 200;
const COUNT_ACROSS: usize = 200;

const LEFT_OFFSET: f32 = 10.0;
const TOP_OFFSET: f32 = 10.0;

const IMPULSE_X_RAND: f32 = 0.5;
const IMPULSE_Y_RAND: f32 = 0.001;

const VEL_X_RAND: f32 = 0.300;
const VEL_Y_RAND: f32 = 0.001;

const ACC_X_RAND: f32 = 0.001;
const ACC_Y_RAND: f32 = 0.001;

const X_GAP: f32 = 10.0;
const Y_GAP: f32 = 10.0;

struct Strand {
    positions: Vec<(f32, f32)>,
    velocities: Vec<(f32, f32)>,
    accelerations: Vec<(f32, f32)>,
}

fn initial_strand() -> Strand {
    let mut positions = vec![];
    let mut velocities = vec![];
    let mut accelerations = vec![];

    for i in 0..COUNT_DOWN {
        positions.push((
            LEFT_OFFSET,
            TOP_OFFSET + i as f32 * Y_GAP,
        ));
        velocities.push((
            rand::thread_rng().gen_range(-VEL_X_RAND / 2.0, VEL_X_RAND),
            rand::thread_rng().gen_range(-VEL_Y_RAND / 2.0, VEL_Y_RAND),
        ));
        accelerations.push((
            rand::thread_rng().gen_range(-ACC_X_RAND, ACC_X_RAND),
            rand::thread_rng().gen_range(-ACC_Y_RAND, ACC_Y_RAND),
        ));
    }

    let velocities = smooth(&smooth(&velocities));
    let accelerations = smooth(&smooth(&accelerations));

    return Strand {
        positions,
        velocities,
        accelerations,
    };
}

fn get_strand(iteration: usize, last_strand: &Strand) -> Strand {
    let positions: Vec<_> = last_strand
        .positions
        .iter()
        .zip(last_strand.velocities.iter())
        .map(|(&(p_last_x, p_last_y), &(v_last_x, v_last_y))| {
            (
                p_last_x + X_GAP + v_last_x,
                p_last_y, // + rand::thread_rng().gen_range(-IMPULSE_X_RAND, IMPULSE_X_RAND),
            )
        })
        .collect();

    let velocities: Vec<_> = last_strand
        .velocities
        .iter()
        .zip(last_strand.accelerations.iter())
        .map(|(&(v_last_x, v_last_y), &(a_last_x, a_last_y))| {
            (
                v_last_x + a_last_x + rand::thread_rng().gen_range(-VEL_X_RAND, VEL_X_RAND),
                v_last_y + a_last_y + rand::thread_rng().gen_range(-VEL_Y_RAND, VEL_Y_RAND),
            )
        })
        .collect();

    let accelerations: Vec<_> = last_strand
        .accelerations
        .iter()
        .map(|&(a_last_x, a_last_y)| (a_last_x, a_last_y))
        .collect();

    let velocities = smooth(&velocities);

    return Strand {
        positions,
        velocities,
        accelerations,
    };
}

fn smooth(object: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut out = vec![object.first().unwrap().clone()];
    for window in object.windows(3) {
        let ((a_x, a_y), (b_x, b_y), (c_x, c_y)) = (window[0], window[1], window[2]);
        let avg_x = (a_x + b_x * 2.0 + c_x) / 4.0;
        let avg_y = (a_y + b_y * 2.0 + c_y) / 4.0;
        out.push((avg_x, avg_y));
    }
    out.push(object.last().unwrap().clone());

    out
}

fn main() {
    let file = BufWriter::new(File::create("out.svg").unwrap());
    let mut canvas = Canvas::new(svg::SvgBackend::new(file).unwrap());

    let mut strands = vec![initial_strand()];
    for i in 1..(COUNT_ACROSS - 1) {
        let new_strand = get_strand(i, strands.last().unwrap());
        strands.push(new_strand);
    }

    for strand in &strands[2..] {
        canvas.draw_lines(strand.positions.iter().cloned()).unwrap();
    }

    /*
    for i in 0..COUNT_ACROSS {
        canvas
            .draw_lines(strands.iter().map(|s| s.positions[i]))
            .unwrap();
    }
    */

    canvas.close().unwrap();
}
