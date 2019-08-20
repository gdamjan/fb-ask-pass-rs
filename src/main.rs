extern crate bmp;
extern crate framebuffer;

mod drawing;
mod passwd;

use drawing::Msg;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::sync::mpsc::Sender;

fn parse_args(args: &[String]) -> Result<Option<String>, &'static str> {
    match args.len() {
        3 => {
            if &args[1] == "--write" {
                Ok(Some(args[2].clone()))
            } else {
                Err("only allowed 1st argument is --write")
            }
        }
        1 => Ok(None),
        _ => Err("only 0 or 2 arguments are allowed"),
    }
}

fn validate_password(pass: String) -> io::Result<String> {
    Ok(pass)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let write_to = parse_args(&args).unwrap();
    let send_to_draw = |sink: Sender<Msg>| {
        let tx = sink.clone();
        move |msg: Msg| tx.send(msg).unwrap()
    };
    let draw = send_to_draw(drawing::init()?);
    let draw_key_callback = |k: passwd::Key| draw(Msg::KeyPressed(k));

    // Start graphical mode
    draw(Msg::Start);

    passwd::read_pass(draw_key_callback)
        .and_then(validate_password)
        .map_err(|_| draw(Msg::Fail))
        .and_then(|pass| {
            draw(Msg::Success);
            match write_to {
                None => {
                    // for testing, get back to text mode
                    println!("You entered: {}", pass);
                }
                Some(fname) => {
                    let mut f = File::create(fname).unwrap();
                    f.write_all(pass.as_bytes()).unwrap();
                }
            }
            draw(Msg::Stop);
            Ok(())
        })
        .map_err(|_| {
            draw(Msg::Fail);
            io::Error::new(io::ErrorKind::InvalidData, "FAAIIILLL")
        })
}
