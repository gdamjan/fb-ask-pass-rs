extern crate bmp;
extern crate framebuffer;

mod drawing;
mod passwd;

use std::env;
use std::fs::File;
use std::io::{self, Write};

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

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let write_to = parse_args(&args).unwrap();

    let tx = drawing::init()?;
    tx.send(drawing::Msg::Start);

    let feedback = || {};
    let pass = passwd::read_pass(&feedback)?;

    match write_to {
        None => {
            // for testing, get back to text mode
            tx.send(drawing::Msg::Stop);
            println!("You entered: {}", pass);
        }
        Some(fname) => {
            let mut f = File::create(fname)?;
            f.write_all(pass.as_bytes())?;
        }
    }

    Ok(())
}
