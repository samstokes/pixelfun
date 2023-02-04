use std::{convert::TryInto, env::args, f32::consts::PI, str::FromStr, time::Instant};

use pixel_canvas::{input::MouseState, Canvas, Color, Image, XY};

const SPEED: usize = 10;
const SPIN: usize = 3;
const SIZE_PX: usize = 512;
const HIDPI: bool = true;
const ANIMATE: bool = true;
const PX_PER_PX: usize = if HIDPI { 2 } else { 1 };
const REAL_SIZE_PX: usize = SIZE_PX * PX_PER_PX;
const MAPFACTOR: usize = 2;
const MAPPING_SIZE_PX: usize = REAL_SIZE_PX * MAPFACTOR;
const TEX_HEIGHT: usize = REAL_SIZE_PX;
const TEX_WIDTH: usize = REAL_SIZE_PX;

#[derive(Eq, PartialEq)]
enum Which {
    Effect,
    Texture,
    Distance,
    Angle,
}

impl FromStr for Which {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "e" | "eff" | "effect" => Ok(Which::Effect),
            "t" | "tex" | "texture" => Ok(Which::Texture),
            "d" | "dist" | "distance" => Ok(Which::Distance),
            "a" | "ang" | "angle" => Ok(Which::Angle),
            _ => Err(format!("{} not recognised", s)),
        }
    }
}

impl Default for Which {
    fn default() -> Self {
        Which::Effect
    }
}

fn main() {
    let which = parse_args().expect("invalid args");

    let canvas = Canvas::new(SIZE_PX, SIZE_PX)
        .title("hey")
        .hidpi(HIDPI)
        .state(MouseState::new())
        .input(MouseState::handle_input);

    let tex = render_texture(TEX_HEIGHT, TEX_WIDTH);

    let (distancemap, anglemap) =
        calculate_mapping(MAPPING_SIZE_PX, MAPPING_SIZE_PX, TEX_HEIGHT, TEX_WIDTH);

    sample_map(
        "distance",
        &distancemap,
        10,
        MAPPING_SIZE_PX,
        MAPPING_SIZE_PX,
    );
    sample_map("angle", &anglemap, 10, MAPPING_SIZE_PX, MAPPING_SIZE_PX);

    let then = Instant::now();

    let mut lastpos = (0, 0);

    canvas.render(move |mouse, image| {
        let width = image.width();
        let height = image.height();

        let elapsed = if ANIMATE {
            (then.elapsed().as_millis() / 20) as usize
        } else {
            0
        };

        let pos = mousenorm(&mouse, height);
        if pos != lastpos {
            let mouseoffset = pos.1 * width + pos.0;
            let mousetex = tex[XY(pos.0 % TEX_WIDTH, pos.1 % TEX_HEIGHT)];
            let mouseang = anglemap[mouseoffset];
            let mousedist = distancemap[mouseoffset];
            println!(
                "mouse: {:?}, tex: {:?}, ang: {}, dist: {}",
                pos,
                (mousetex.r, mousetex.g, mousetex.b),
                mouseang,
                mousedist
            );
            lastpos = pos;
        }

        if which == Which::Texture {
            let rows = image.chunks_mut(width);
            let tex_rows = tex.chunks(TEX_WIDTH);
            for (row, tex_row) in rows.zip(tex_rows) {
                row[0..TEX_WIDTH].clone_from_slice(tex_row);
            }
            return;
        }

        let half_wf = width as f32 * 0.5;
        let qu_wf = width as f32 * 0.25;
        let half_hf = height as f32 * 0.5;
        let qu_hf = height as f32 * 0.25;
        let elapsedf = elapsed as f32 * 0.005;

        for (y, row) in image.chunks_mut(width).enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                let shift_x = (half_wf + qu_wf * (elapsedf * 2.0).sin()) as usize;
                let shift_y = (half_hf + qu_hf * (elapsedf * 4.5).sin()) as usize;

                let mapoffset = (y + shift_y) * (width * MAPFACTOR) + (x + shift_x);

                let angle = (anglemap[mapoffset] + SPIN * elapsed) % TEX_WIDTH;
                let dist = (distancemap[mapoffset] + SPEED * elapsed) % TEX_HEIGHT;

                let texel = tex[XY(angle, dist)];

                *pixel = match which {
                    Which::Distance => grey("distance", dist, TEX_HEIGHT),
                    Which::Angle => grey("angle", angle, TEX_WIDTH),
                    Which::Effect => texel,
                    _ => unreachable!(),
                };
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

fn parse_args() -> Result<Which, String> {
    let mut args = args();
    if args.len() > 2 {
        return Err("too many args".into());
    }
    match args.nth(1) {
        Some(arg) => Which::from_str(&arg),
        None => Ok(Which::default()),
    }
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
    let half_twf = 0.5 * twf;
    let thf = texheight as f32;
    let ratio_thf = RATIO * thf;

    let mut distmap = vec![0; width * height];
    let mut anglemap = vec![0; width * height];

    for y in 0..height {
        for x in 0..width {
            let xf = x as f32;
            let yf = y as f32;

            let x_center = xf - wf * 0.5;
            let y_center = yf - hf * 0.5;

            distmap[y * width + x] =
                (ratio_thf / (x_center * x_center + y_center * y_center).sqrt()) as usize
                    % texheight;

            anglemap[y * width + x] =
                (half_twf * (1.0 + y_center.atan2(x_center) / PI)) as usize % texwidth;
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

fn mousenorm(mouse: &MouseState, height: usize) -> (usize, usize) {
    return (
        (mouse.x / 2) as usize,
        ((mouse.y + height as i32) / 2) as usize,
    );
}
