use png::*;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;

const RENDER_SIZE: f64 = 15.0;

pub fn draw<F>(f: F)
where
    F: Fn(f64, f64) -> f64,
{
    let width = 1000;
    let height = 1000;
    let asdf = normalize(&render(width, height, f));
    let mut pixels = Vec::with_capacity(asdf.len() * 4);
    for pix in asdf {
        let value = (pix * 255.0) as u8;
        pixels.push(value);
        pixels.push(value);
        pixels.push(value);
        pixels.push(255);
    }
    save_image(&pixels, width, height).unwrap();
}

fn render<F>(width: u32, height: u32, f: F) -> Vec<f64>
where
    F: Fn(f64, f64) -> f64,
{
    let mut result = Vec::with_capacity((height * width) as usize);
    for y in 0..height {
        if (height - y) % (1 << 4) == 0 {
            println!("{}", height - y);
        }
        for x in 0..width {
            let dest_x = (x as f64 / width as f64 - 0.5) * (2.0 * RENDER_SIZE);
            let dest_y = (y as f64 / height as f64 - 0.5) * (2.0 * RENDER_SIZE * -1.0);
            result.push(f(dest_x, dest_y));
        }
    }
    result
}

fn normalize(data: &[f64]) -> Vec<f64> {
    // TY: This is log2 vs. linear choice.
    //for datum in data.iter_mut() {
    //    *datum = datum.map(|x| x.log2());
    //}
    let min = data.iter().cloned().fold(1.0 / 0.0, f64::min);
    let max = data.iter().cloned().fold(-1.0 / 0.0, f64::max);
    let result = data.iter()
        .map(|x| (x - min) / (max - min))
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
