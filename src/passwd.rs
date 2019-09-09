/*
   code based on https://github.com/mitsuhiko/console/
   slightly reorganized
*/

/*
The MIT License (MIT)

Copyright (c) 2017 Armin Ronacher <armin.ronacher@active-4.com>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use std::fs;
use std::io;
use std::os::unix::io::AsRawFd;
use std::str;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Key {
    Unknown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Enter,
    Escape,
    Char(char),
    #[doc(hidden)]
    __More,
}

fn key_from_escape_codes(buf: &[u8]) -> Key {
    match buf {
        b"\x1b[D" => Key::ArrowLeft,
        b"\x1b[C" => Key::ArrowRight,
        b"\x1b[A" => Key::ArrowUp,
        b"\x1b[B" => Key::ArrowDown,
        b"\n" | b"\r" => Key::Enter,
        b"\x1b" => Key::Escape,
        buf => {
            if let Ok(s) = str::from_utf8(buf) {
                if let Some(c) = s.chars().next() {
                    return Key::Char(c);
                }
            }
            Key::Unknown
        }
    }
}

fn read_single_key(fd: i32) -> io::Result<Key> {
    let mut buf = [0u8; 20];

    let rv = unsafe {
        let read = libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, 20);
        if read < 0 {
            Err(io::Error::last_os_error())
        } else if buf[0] == b'\x03' {
            Err(io::Error::new(io::ErrorKind::Interrupted, "read interrupted"))
        } else {
            Ok(key_from_escape_codes(&buf[..read as usize]))
        }
    };

    // if the user hit ^C we want to signal SIGINT to outselves.
    if let Err(ref err) = rv {
        if err.kind() == io::ErrorKind::Interrupted {
            unsafe {
                libc::raise(libc::SIGINT);
            }
        }
    }

    rv
}

pub fn read_pass(feedback: &dyn Fn()) -> io::Result<String> {
    let tty_f = fs::File::open("/dev/tty")?;
    let fd = tty_f.as_raw_fd();

    let mut termios = termios::Termios::from_fd(fd)?;
    let original = termios;
    termios::cfmakeraw(&mut termios);
    termios::tcsetattr(fd, termios::TCSADRAIN, &termios)?;

    let mut pass = String::new();
    let rv = loop {
        match read_single_key(fd)? {
            Key::Char(c) => {
                pass.push(c);
                feedback();
            }
            Key::Enter => break Ok(pass),
            _ => (),
        }
    };

    termios::tcsetattr(fd, termios::TCSADRAIN, &original)?;
    rv
}
