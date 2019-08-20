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
    xoffset: u32,
    yoffset: u32,
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
    fn new(xoffset: u32, yoffset: u32, width: u32, height: u32, bytes_per_pixel: u32) -> Self {
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
        let img = bmp::open(path).unwrap();
        for (x, y) in img.coordinates() {
            let px = img.get_pixel(x, y);
            let idx = (((y + self.yoffset) * self.width)
                + ((x + self.xoffset) * self.bytes_per_pixel)) as usize;
            self.buffer[idx] = px.b;
            self.buffer[idx + 1] = px.g;
            self.buffer[idx + 2] = px.r;
        }
    }

    fn draw(&mut self, shape: Shape) {
        for Pixel(Coordinate(x, y), Color(r, g, b)) in shape.pixels {
            let idx = (((y + self.yoffset) * self.width)
                + ((x + self.xoffset) * self.bytes_per_pixel)) as usize;
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

fn start() {
    // Disable text mode in current tty
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();

    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let xoffset = read_u32_from_file("/sys/firmware/acpi/bgrt/xoffset").unwrap_or(10);
    let yoffset = read_u32_from_file("/sys/firmware/acpi/bgrt/yoffset").unwrap_or(10);
    let width = framebuffer.fix_screen_info.line_length;
    let height = framebuffer.var_screen_info.yres;
    let bytespp = framebuffer.var_screen_info.bits_per_pixel / 8;

    let mut frame = Frame::new(xoffset, yoffset, width, height, bytespp);
    // frame.draw_image("/sys/firmware/acpi/bgrt/image");
    frame.draw_image("./test.bmp");

    framebuffer.write_frame(frame.buffer.as_slice());
}

fn stop() {
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}

fn draw_pass_validate() {}
fn draw_pass_type() {}
fn draw_pass_clear() {}
fn draw_pass_success() {}
fn draw_pass_fail() {}

pub enum Msg {
    Start,
    Stop,
    KeyPressed(Key),
    Success,
    Fail,
}

pub fn init() -> Result<Sender<Msg>, io::Error> {
    let (tx, rx) = channel::<Msg>();

    thread::spawn(move || loop {
        match rx.recv().unwrap() {
            Msg::Start => start(),
            Msg::Stop => stop(),
            Msg::KeyPressed(Key::Enter) => draw_pass_validate(),
            Msg::KeyPressed(Key::Char(_)) => draw_pass_type(),
            Msg::KeyPressed(Key::Escape) => draw_pass_clear(),
            Msg::KeyPressed(_) => (),
            Msg::Success => draw_pass_success(),
            Msg::Fail => draw_pass_fail(),
        }
    });

    Ok(tx)
}
