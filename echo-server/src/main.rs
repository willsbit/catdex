use ws::{Handler, Sender, WebSocket};
use std::{thread, time};

struct Server {
    out: Sender
}

impl Handler for Server {}

fn main() {
    let server = WebSocket::new(|out| Server { out }).unwrap();

    let broadcaster = server.broadcaster();

    let periodic = thread::spawn(move || loop {
        broadcaster.send("Meow!").unwrap();
        thread::sleep(time::Duration::from_secs(1));
    });

    server.listen("127.0.0.1:8080").unwrap();

    periodic.join().unwrap()
}
