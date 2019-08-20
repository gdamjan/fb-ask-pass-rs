extern crate bmp;
extern crate framebuffer;

mod drawing;
mod passwd;

use drawing::Msg;
use passwd::*;
use std::env;
use std::io;
use std::sync::mpsc::Sender;

fn parse_args(args: &[String]) -> Result<Option<String>, &'static str> {
    match args.len() {
        3 => {
            if &args[1] == "--write" {
                Ok(Some(args[2].to_string()))
            } else {
                Err("only allowed 1st argument is --write")
            }
        }
        1 => Ok(None),
        _ => Err("only 0 or 2 arguments are allowed"),
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let write_to = parse_args(&args).unwrap();
    let send_to_draw = |sink: Sender<Msg>| {
        let tx = sink.clone();
        move |msg: Msg| tx.send(msg).unwrap()
    };
    let draw = send_to_draw(drawing::init()?);
    let draw_key_callback = |k: Key| draw(Msg::KeyPressed(k));

    // Start graphical mode
    draw(Msg::Start);

    read_pass(draw_key_callback)
        .and_then(validate_pass)
        .map_err(|_| draw(Msg::Fail))
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
