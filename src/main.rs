use std::{convert::TryInto, f32::consts::PI, fmt::Display, time::Instant};

use pixel_canvas::{Canvas, Color, Image, XY};

const SPEED: usize = 10;
const SPIN: usize = 3;
const SIZE_PX: usize = 512;
const HIDPI: bool = true;
const ANIMATE: bool = true;
const PX_PER_PX: usize = if HIDPI { 2 } else { 1 };
const REAL_SIZE_PX: usize = SIZE_PX * PX_PER_PX;
const TEX_HEIGHT: usize = REAL_SIZE_PX;
const TEX_WIDTH: usize = REAL_SIZE_PX;

fn main() {
    let canvas = Canvas::new(SIZE_PX, SIZE_PX)
        .title("hey")
        .hidpi(HIDPI)
        .render_on_change(!ANIMATE);

    let tex = render_texture(TEX_HEIGHT, TEX_WIDTH);

    let (distancemap, anglemap) =
        calculate_mapping(REAL_SIZE_PX, REAL_SIZE_PX, TEX_HEIGHT, TEX_WIDTH);

    sample_map("distance", &distancemap, 10, REAL_SIZE_PX, REAL_SIZE_PX);
    sample_map("angle", &anglemap, 10, REAL_SIZE_PX, REAL_SIZE_PX);

    let then = Instant::now();

    canvas.render(move |_sploot, image| {
        let elapsed = (then.elapsed().as_millis() / 100) as usize;

        //image.clone_from_slice(&tex); // draw texture

        let width = image.width();
        let height = image.height();
        for (y, row) in image.chunks_mut(width).enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                let mapoffset = y * width + x;

                let angle = (anglemap[mapoffset] + SPIN * elapsed) % TEX_WIDTH;
                let dist = (distancemap[mapoffset] + SPEED * elapsed) % TEX_HEIGHT;

                let texel = tex[XY(angle, dist)];

                //*pixel = grey("distance", dist, TEX_HEIGHT); // draw distance map
                //*pixel = grey("angle", angle, TEX_WIDTH); // draw angle map
                *pixel = texel; // draw actual effect
            }
        }

        // OLD experiments with weird diamond stuff
        //let width = image.width();
        //let height = image.height();
        //for (y, row) in image.chunks_mut(width).enumerate() {
        //for (x, pixel) in row.iter_mut().enumerate() {
        //let xp = x as f32 / width as f32 - 0.5;
        //let yp = y as f32 / height as f32 - 0.5;

        //let xps = (5.0 * xp + yp + elapsed).sin();
        //let yps = (3.0 * yp + 2.0 * elapsed).sin();

        //let intensity = ((xps * xps + yps * yps) * 256.0) as u8;

        //*pixel = Color::rgb(intensity, intensity, intensity);
        //}
        //}
    });
}

fn render_texture(width: usize, height: usize) -> Image {
    let mut tex = Image::new(width, height);

    for (y, row) in tex.chunks_mut(width).enumerate() {
        for (x, texel) in row.iter_mut().enumerate() {
            *texel = Color::rgb(0, 0, (x * 256 / width) as u8 ^ (y * 256 / height) as u8)
        }
    }

    tex
}

fn calculate_mapping(
    width: usize,
    height: usize,
    texwidth: usize,
    texheight: usize,
) -> (Vec<usize>, Vec<usize>) {
    const RATIO: f32 = 32.0;

    let wf = width as f32;
    let hf = height as f32;
    let twf = texwidth as f32;
    let thf = texheight as f32;

    let mut distmap = vec![0; width * height];
    let mut anglemap = vec![0; width * height];

    for y in 0..height {
        for x in 0..width {
            let xf = x as f32;
            let yf = y as f32;

            let x_center = xf - wf * 0.5;
            let y_center = yf - hf * 0.5;

            distmap[y * width + x] =
                (RATIO * thf / (x_center * x_center + y_center * y_center).sqrt()) as usize
                    % texheight;

            anglemap[y * width + x] = (twf * y_center.atan2(x_center) / PI) as usize;
        }
    }

    (distmap, anglemap)
}

fn sample_map(label: &str, map: &[usize], samples: usize, width: usize, height: usize) {
    println!("{}:", label);

    let (mut min, mut max) = (usize::MAX, usize::MIN);
    for y in 0..height {
        for x in 0..width {
            let v = map[y * width + x];

            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }

            if x < samples && y < samples {
                print!("{:2} ", v);
            }
        }
        if y < samples {
            println!();
        }
    }

    println!("{} min: {}, max: {}", label, min, max);
}

fn grey(_label: &str, value: usize, max: usize) -> Color {
    let intensity = (255 * value / max).try_into().unwrap();
    //println!(
    //"{}: {}, {} * {} / {} = {}",
    //label, value, max, value, max, intensity
    //);
    Color::rgb(intensity, intensity, intensity)
}
