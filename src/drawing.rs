extern crate framebuffer;

use framebuffer::{Framebuffer, KdMode};
use std::fs::File;
use std::io::{self, Read};
use std::sync::mpsc;
use std::thread;

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

fn draw_image_centered(device: String, image_path: String) {
    let mut framebuffer = Framebuffer::new(device).unwrap();
    let frame = Frame::from_image(&framebuffer, &image_path, None, None);
    frame.draw(&mut framebuffer);
}

fn draw_bgrt(device: String) {
    let mut framebuffer = Framebuffer::new(device).unwrap();
    let xoffset = read_u32_from_file("/sys/firmware/acpi/bgrt/xoffset").ok();
    let yoffset = read_u32_from_file("/sys/firmware/acpi/bgrt/yoffset").ok();
    let frame = Frame::from_image(&framebuffer, "/sys/firmware/acpi/bgrt/image", xoffset, yoffset);
    frame.draw(&mut framebuffer);
}

pub enum Msg {
    Start(String, Option<String>),
    Stop,
}

fn start(device: String, image_path: Option<String>) {
    match image_path {
        None => draw_bgrt(device),
        Some(image_path) => draw_image_centered(device, image_path),
    }
}

fn stop() {
    Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}

pub fn init() -> impl Fn(Msg) -> () {
    let (tx, rx) = mpsc::channel::<Msg>();

    thread::spawn(move || loop {
        match rx.recv().unwrap() {
            Msg::Start(device, image_path) => start(device, image_path),
            Msg::Stop => stop(),
        }
    });

    move |m: Msg| tx.send(m).unwrap()
}
