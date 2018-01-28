extern crate rand;
extern crate vectorphile;

use vectorphile::{svg, Canvas};
use std::fs::File;
use std::io::BufWriter;

const COUNT_DOWN: usize = 10;
const COUNT_ACROSS: usize = 10;

const LEFT_OFFSET: f32 = 10.0;
const TOP_OFFSET: f32 = 10.0;

const Y_GAP: f32 = 10.0;
const X_GAP: f32 = 10.0;

struct Strand {
    positions: Vec<(f32, f32)>,
    velocities: Vec<(f32, f32)>,
}

fn get_strand(iteration: usize, last_strand: Option<&Strand>) -> Strand {
    let mut positions = vec![];
    let mut velocities = vec![];

    for i in 0..COUNT_DOWN {
        positions.push((
            LEFT_OFFSET + X_GAP * iteration as f32,
            TOP_OFFSET + i as f32 * Y_GAP,
        ));
    }

    return Strand {
        positions,
        velocities,
    };
}

fn main() {
    let file = BufWriter::new(File::create("out.svg").unwrap());
    let mut canvas = Canvas::new(svg::SvgBackend::new(file).unwrap());

    let mut strands = vec![];
    for i in 0..COUNT_ACROSS {
        let new_strand = {
            let prev = if i == 0 { None } else { strands.last() };
            get_strand(i, prev)
        };
        strands.push(new_strand);
    }

    for strand in &strands {
        canvas.draw_lines(strand.positions.iter().cloned()).unwrap();
    }

    for i in 0..COUNT_ACROSS {
        canvas
            .draw_lines(strands.iter().map(|s| s.positions[i]))
            .unwrap();
    }

    canvas.close().unwrap();
}
