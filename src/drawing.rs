use framebuffer::{Framebuffer, KdMode};
use passwd::Key;
use std::fs::File;
use std::io::{self, Read};
use std::sync::mpsc::{channel, Sender};
use std::thread;

struct Coordinate(u32, u32);
struct Color(u8, u8, u8);
struct Pixel(Coordinate, Color);

pub struct Frame {
    buffer: Vec<u8>,
    xoffset: Option<u32>,
    yoffset: Option<u32>,
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
}

struct Shape {
    pixels: Vec<Pixel>,
}

struct Rect {
    min: Coordinate,
    max: Coordinate,
    color: Color,
}

//impl Into<Shape> for Rect {
//    fn into(&self) -> Shape {
//        for i in self.min[0]..self.max[0] {
//            for j in self.min[1]..self.max[1] {}
//        }
//    }
//}

impl Frame {
    fn new(
        xoffset: Option<u32>,
        yoffset: Option<u32>,
        width: u32,
        height: u32,
        bytes_per_pixel: u32,
    ) -> Self {
        Self {
            buffer: vec![0u8; (width * height) as usize],
            xoffset,
            yoffset,
            width,
            height,
            bytes_per_pixel,
        }
    }

    fn draw_image(&mut self, path: &str) {
        println!("{}", path);
        let img = bmp::open(path).unwrap();
        let xoffset = self
            .xoffset
            .unwrap_or((self.width / self.bytes_per_pixel) / 2 - img.get_width() / 2);
        let yoffset = self
            .yoffset
            .unwrap_or(self.height / 2 - img.get_height() / 2);

        for (x, y) in img.coordinates() {
            let px = img.get_pixel(x, y);
            let xb = (x + xoffset) * self.bytes_per_pixel;
            let yb = (y + yoffset) * self.width;
            let idx = (xb + yb) as usize;
            self.buffer[idx] = px.b;
            self.buffer[idx + 1] = px.g;
            self.buffer[idx + 2] = px.r;
        }
    }

    fn draw(&mut self, shape: Shape) {
        for Pixel(Coordinate(x, y), Color(r, g, b)) in shape.pixels {
            let idx = (y * self.width + x) as usize;
            self.buffer[idx] = b;
            self.buffer[idx + 1] = g;
            self.buffer[idx + 2] = r;
        }
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

fn start(fb: &mut Framebuffer, image_path: &str) {
    let xoffset = read_u32_from_file("/sys/firmware/acpi/bgrt/xoffset").ok();
    let yoffset = read_u32_from_file("/sys/firmware/acpi/bgrt/yoffset").ok();
    let bytespp = fb.var_screen_info.bits_per_pixel / 8;
    let width = fb.fix_screen_info.line_length;
    let height = fb.var_screen_info.yres;

    let mut frame = Frame::new(xoffset, yoffset, width, height, bytespp);
    frame.draw_image(image_path);

    // Disable text mode in current tty
    Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    fb.write_frame(frame.buffer.as_slice());
}

fn stop() {
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}

fn draw_pass_validate(fb: &mut Framebuffer) {}
fn draw_pass_type(fb: &mut Framebuffer) {}
fn draw_pass_clear(fb: &mut Framebuffer) {}
fn draw_pass_success(fb: &mut Framebuffer) {}
fn draw_pass_fail(fb: &mut Framebuffer) {}

#[derive(Debug)]
pub enum Msg {
    Start,
    Stop,
    KeyPressed(Key),
    Success,
    Fail,
}

pub fn init(image_path: String) -> Result<Sender<Msg>, io::Error> {
    let (tx, rx) = channel::<Msg>();
    let mut fb = Framebuffer::new("/dev/fb0").unwrap();

    thread::spawn(move || loop {
        match rx.recv().unwrap() {
            Msg::Start => start(&mut fb, &image_path),
            Msg::KeyPressed(Key::Enter) => draw_pass_validate(&mut fb),
            Msg::KeyPressed(Key::Char(_)) => draw_pass_type(&mut fb),
            Msg::KeyPressed(Key::Escape) => draw_pass_clear(&mut fb),
            Msg::KeyPressed(_) => (),
            Msg::Stop => stop(),
            Msg::Fail => draw_pass_fail(&mut fb),
            Msg::Success => draw_pass_success(&mut fb),
        };
    });

    Ok(tx)
}
