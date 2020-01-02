use crate::passwd::Key;
use framebuffer::{Framebuffer, KdMode};
use rand::distributions::{Distribution, Uniform};
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, Read};
use std::sync::mpsc;
use std::thread;

#[derive(Clone)]
struct Coordinate(u32, u32);
#[derive(Clone)]
struct Color(u8, u8, u8);
#[derive(Clone)]
struct Pixel(Coordinate, Color);

#[derive(Clone)]
pub struct Frame {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
}

impl Frame {
    pub fn new(fb: &Framebuffer) -> Self {
        let width = fb.fix_screen_info.line_length;
        let height = fb.var_screen_info.yres;
        let bytes_per_pixel = fb.var_screen_info.bits_per_pixel / 8;

        Self {
            buffer: vec![0u8; (width * height) as usize],
            width,
            height,
            bytes_per_pixel,
        }
    }

    pub fn from_image(fb: &Framebuffer, path: &str, xoffset: Option<u32>, yoffset: Option<u32>) -> Self {
        let mut frame = Self::new(fb);
        let img = bmp::open(path).unwrap();
        let xof = xoffset.unwrap_or((frame.width / frame.bytes_per_pixel) / 2 - img.get_width() / 2);
        let yof = yoffset.unwrap_or(frame.height / 2 - img.get_height() / 2);

        for (x, y) in img.coordinates() {
            let px = img.get_pixel(x, y);
            let xb = (x + xof) * frame.bytes_per_pixel;
            let yb = (y + yof) * frame.width;
            let idx = (xb + yb) as usize;
            frame.buffer[idx] = px.b;
            frame.buffer[idx + 1] = px.g;
            frame.buffer[idx + 2] = px.r;
        }
        frame
    }

    fn set_shape(&mut self, shape: Shape) {
        for Pixel(Coordinate(x, y), Color(r, g, b)) in shape.pixels {
            let idx = (y * self.width + x * self.bytes_per_pixel) as usize;
            self.buffer[idx] = b;
            self.buffer[idx + 1] = g;
            self.buffer[idx + 2] = r;
        }
    }

    fn center(&self) -> Coordinate {
        Coordinate(self.width / 8, self.height / 2)
    }

    pub fn draw(self: &Self, fb: &mut Framebuffer) {
        fb.write_frame(self.buffer.as_slice());
    }
}

fn read_u32_from_file(fname: &str) -> io::Result<u32> {
    let mut f = File::open(fname)?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    buffer
        .trim()
        .parse::<u32>()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "can't parse number"))
}

struct Shape {
    pixels: Vec<Pixel>,
}

struct Circle {
    center: Coordinate,
    radius: i32,
    color: Color,
}

impl Into<Shape> for Circle {
    fn into(self) -> Shape {
        let mut pixels: Vec<Pixel> = Vec::new();

        for y in -self.radius..self.radius {
            for x in -self.radius..self.radius {
                if x * x + y * y <= self.radius * self.radius {
                    let new_x: u32 = (self.center.0 as i32 + x).try_into().unwrap_or(0);
                    let new_y: u32 = (self.center.1 as i32 + y).try_into().unwrap_or(0);
                    let pixel = Pixel(Coordinate(new_x, new_y), self.color.clone());
                    pixels.push(pixel);
                }
            }
        }
        Shape { pixels }
    }
}

fn draw_image_centered(fb: &mut Framebuffer, image_path: String) -> Frame {
    Frame::from_image(fb, &image_path, None, None)
}

fn draw_bgrt(fb: &mut Framebuffer) -> Frame {
    let xoffset = read_u32_from_file("/sys/firmware/acpi/bgrt/xoffset").ok();
    let yoffset = read_u32_from_file("/sys/firmware/acpi/bgrt/yoffset").ok();
    Frame::from_image(fb, "/sys/firmware/acpi/bgrt/image", xoffset, yoffset)
}

pub enum Msg {
    Start(Option<String>),
    Stop,
    Keypress(Key),
}

fn start(fb: &mut Framebuffer, image_path: Option<String>) -> Frame {
    match image_path {
        None => draw_bgrt(fb),
        Some(image_path) => draw_image_centered(fb, image_path),
    }
}

fn stop() {
    Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}

fn draw_input_update(frame: &mut Frame, ring_color: Color, inner_color: Color) {
    let b = Circle {
        center: frame.center(),
        color: Color(0, 0, 0),
        radius: 60,
    };
    let c = Circle {
        center: frame.center(),
        color: ring_color,
        radius: 55,
    };
    let b2 = Circle {
        center: frame.center(),
        color: Color(0, 0, 0),
        radius: 46,
    };
    let changing = Circle {
        center: frame.center(),
        color: inner_color,
        radius: 45,
    };
    frame.set_shape(b.into());
    frame.set_shape(c.into());
    frame.set_shape(b2.into());
    frame.set_shape(changing.into());
}

fn draw_pass_enter(frame: &mut Frame) {
    draw_input_update(frame, Color(0, 200, 0), Color(0, 200, 0));
}

fn draw_pass_char(frame: &mut Frame) {
    let mut rng = rand::thread_rng();
    let colgen = Uniform::from(0..255);
    let rand_color: Color = Color(
        colgen.sample(&mut rng),
        colgen.sample(&mut rng),
        colgen.sample(&mut rng),
    );
    draw_input_update(frame, Color(155, 0, 255), rand_color);
}

fn draw_pass_escape() {}

fn draw_keypress(frame: &mut Frame, key: Key) {
    match key {
        Key::Enter => draw_pass_enter(frame),
        Key::Char(_) => draw_pass_char(frame),
        Key::Escape => draw_pass_escape(),
        _ => (),
    }
}

pub fn init(device: String) -> impl Fn(Msg) -> () {
    let (tx, rx) = mpsc::channel::<Msg>();

    let mut fb = Framebuffer::new(device).unwrap();
    let mut initial_frame = Frame::new(&fb);

    thread::spawn(move || loop {
        match rx.recv().unwrap() {
            Msg::Start(image_path) => {
                initial_frame = start(&mut fb, image_path);
                initial_frame.draw(&mut fb);
            }
            Msg::Stop => stop(),
            Msg::Keypress(k) => {
                let mut f = initial_frame.clone();
                draw_keypress(&mut f, k);
                f.draw(&mut fb);
            }
        }
    });

    move |m: Msg| tx.send(m).unwrap()
}
