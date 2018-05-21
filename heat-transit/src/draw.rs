use png::*;
use rayon::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use time::TimeCost;

const RENDER_SIZE: f64 = 15.0;

pub fn draw<F>(f: F)
where
    F: Fn(f64, f64) -> Option<TimeCost> + Sync + Send,
{
    let width = 1000;
    let height = 1000;
    let values = normalize(&render(width, height, f));
    let mut pixels = Vec::with_capacity(values.len() * 4);
    for pix in values {
        if let Some(pix) = pix {
            pixels.push((pix.walk_time * 255.0) as u8);
            pixels.push((pix.bus_time * 255.0) as u8);
            pixels.push((pix.wait_time * 255.0) as u8);
            pixels.push(255);
        } else {
            pixels.push(0);
            pixels.push(0);
            pixels.push(0);
            pixels.push(0);
        }
    }
    save_image(&pixels, width, height).unwrap();
}

fn render<F>(width: u32, height: u32, f: F) -> Vec<Option<TimeCost>>
where
    F: Fn(f64, f64) -> Option<TimeCost> + Sync + Send,
{
    let mut result = Vec::with_capacity((height * width) as usize);
    for y in 0..height {
        println!("{}", height - y);
        let row = (0..width).collect::<Vec<_>>();
        let row = row.par_iter().map(|&x| {
            let dest_x = (x as f64 / width as f64 - 0.5) * (2.0 * RENDER_SIZE);
            let dest_y = (y as f64 / height as f64 - 0.5) * (2.0 * RENDER_SIZE * -1.0);
            f(dest_x, dest_y)
        });
        result.par_extend(row);
    }
    result
}

fn normalize(data: &[Option<TimeCost>]) -> Vec<Option<TimeCost>> {
    // TY: This is log2 vs. linear choice.
    //for datum in data.iter_mut() {
    //    *datum = datum.map(|x| x.log2());
    //}
    let min = data.iter()
        .cloned()
        .filter_map(|x| x)
        .fold(TimeCost::with_all(1.0 / 0.0), ::std::cmp::min);
    let max = data.iter()
        .cloned()
        .filter_map(|x| x)
        .fold(TimeCost::with_all(-1.0 / 0.0), ::std::cmp::max);

    data.iter()
        .map(|x| {
            x.map(|x| TimeCost {
                walk_time: (x.walk_time - min.walk_time) / (max.walk_time - min.walk_time),
                bus_time: (x.bus_time - min.bus_time) / (max.bus_time - min.bus_time),
                wait_time: (x.wait_time - min.wait_time) / (max.wait_time - min.wait_time),
                transfers: x.transfers,
            })
        })
        .collect()
}

fn save_image(data: &[u8], width: u32, height: u32) -> Result<(), Box<Error>> {
    let file = File::create("./out/out.png")?;
    let w = &mut BufWriter::new(file);
    let mut encoder = Encoder::new(w, width, height);
    encoder.set(ColorType::RGBA).set(BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;
    Ok(())
}
