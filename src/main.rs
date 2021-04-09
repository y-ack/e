mod interface;

use interface::Interface;
use termion::async_stdin;

fn main() {
    println!("Hello, world!");
    let mut interface = Interface::default();

    interface.clear().ok();
    interface.draw().ok();

    let mut stdin = async_stdin();
    let b = stdin.next();
    if let Some(Ok(b'q')) = b {
        break;
    }
}
