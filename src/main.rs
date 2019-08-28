extern crate bmp;
extern crate framebuffer;

mod drawing;
mod passwd;

use drawing::Msg;
use passwd::*;
use std::env;
use std::io;
use std::sync::mpsc::Sender;

struct Config {
    image_path: Option<String>,
    pass_path: Option<String>,
}

fn parse_args(args: &[String]) -> Result<Config, &'static str> {
    let config: Option<Config> = match args.iter().map(String::as_str).collect::<Vec<&str>>()[..] {
        [_, "--image", img, "--write", path] | [_, "--write", path, "--image", img] => {
            Some(Config {
                image_path: Some(String::from(img)),
                pass_path: Some(String::from(path)),
            })
        }
        [_, "--write", path] => Some(Config {
            image_path: None,
            pass_path: Some(String::from(path)),
        }),
        [_, "--image", img] => Some(Config {
            image_path: Some(String::from(img)),
            pass_path: None,
        }),
        [_] => Some(Config {
            image_path: None,
            pass_path: None,
        }),
        _ => None,
    };

    config.ok_or("Possible arguments are --write and --image.")
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args).unwrap();
    let write_to = config.pass_path;
    let image_path = config.image_path;

    let send_to_draw = |sink: Sender<Msg>| {
        let tx = sink.clone();
        move |msg: Msg| {
            println!("Sending msg {:?}", msg);
            tx.send(msg).unwrap()
        }
    };
    let default_image = String::from("/sys/firmware/acpi/bgrt/image");
    let draw = send_to_draw(drawing::init(image_path.unwrap_or(default_image))?);
    let draw_key_callback = |k: Key| {
        println!("Drawing key {:?}", k);
        draw(Msg::KeyPressed(k))
    };

    // Start graphical mode
    draw(Msg::Start);

    read_pass(draw_key_callback)
        .and_then(validate_pass)
        .and_then(|pass| {
            draw(Msg::Success);
            write_pass(write_to, pass);
            draw(Msg::Stop);
            Ok(())
        })
        .map_err(|_| {
            draw(Msg::Fail);
            io::Error::new(io::ErrorKind::InvalidData, "FAAIIILLL")
        })
}
