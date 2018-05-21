use png::*;
use rayon::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;

const RENDER_SIZE: f64 = 15.0;

pub fn draw<F>(f: F)
where
    F: Fn(f64, f64) -> Option<f64> + Sync + Send,
{
    let width = 1000;
    let height = 1000;
    let values = normalize(&render(width, height, f));
    let mut pixels = Vec::with_capacity(values.len() * 4);
    for pix in values {
        if let Some(pix) = pix {
            let value = (pix * 255.0) as u8;
            pixels.push(value);
            pixels.push(value);
            pixels.push(value);
            pixels.push(255);
        } else {
            pixels.push(255);
            pixels.push(0);
            pixels.push(0);
            pixels.push(255);
        }
    }
    save_image(&pixels, width, height).unwrap();
}

fn render<F>(width: u32, height: u32, f: F) -> Vec<Option<f64>>
where
    F: Fn(f64, f64) -> Option<f64> + Sync + Send,
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

fn normalize(data: &[Option<f64>]) -> Vec<Option<f64>> {
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
        .map(|x| x.map(|x| (x - min) / (max - min)))
        .collect();
    result
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
