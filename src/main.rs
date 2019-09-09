mod cli;
mod drawing;
mod passwd;

use drawing::Msg;
use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let config = cli::get_config().unwrap();
    let draw = drawing::init();

    draw(Msg::Start(config.device, config.image_path));

    let feedback = || {};
    let pass = passwd::read_pass(&feedback)?;

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
