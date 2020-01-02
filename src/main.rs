mod cli;
mod drawing;
mod passwd;

use drawing::Msg;
use passwd::{read_pass, Key};
use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let config = cli::get_config().unwrap();
    let draw = drawing::init(config.device);

    // ``draw_keypress`` is a function that takes a ``Key`` and draws pixels for that key in the
    // frame buffer.
    let draw_keypress = |k: Key| draw(Msg::Keypress(k));

    draw(Msg::Start(config.image_path));

    let pass = read_pass(draw_keypress)?;

    match config.pass_path {
        None => {
            // for testing, get back to text mode
            draw(Msg::Stop);
            println!("You entered: {}", pass);
        }
        Some(fname) => {
            let mut f = File::create(fname)?;
            f.write_all(pass.as_bytes())?;
        }
    }

    Ok(())
}
