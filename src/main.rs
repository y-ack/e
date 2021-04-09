mod interface;

use interface::Interface;

fn main() {
    println!("Hello, world!");
    let mut interface = Interface::default();

    interface.clear().ok();
    interface.draw().ok();
}
