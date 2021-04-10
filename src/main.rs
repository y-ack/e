mod interface;

use interface::{Buffer, Interface, Window};

fn main() {
    let mut interface = Interface::default();

    let buffer = Buffer {
        content: String::from("function hello_world() { console.log('hello, world!'); }"),
        name: String::from("test.txt"),
    };

    interface.windows.push(Window::new(&buffer));
    interface.windows.push(Window::new(&buffer));

    interface.clear().ok();
    interface.draw().ok();

    loop {
        interface.update().ok().expect("oh well 2");
    }
}
