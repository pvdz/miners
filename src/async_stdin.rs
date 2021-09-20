// https://stackoverflow.com/questions/30012995/how-can-i-read-non-blocking-from-stdin
// :shrug:

use std::io;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

pub fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();

        // let mut buf = Vec::new();
        // io::stdin().read_to_end(&mut buf).unwrap();
        // let s = String::from_utf8(buf).expect("Found invalid UTF-8");
        // tx.send(s).unwrap();

        // let mut buf = [0, 1];
        // io::stdin().read_exact(&mut buf).unwrap();
        // let s = String::from_utf8(buf.to_vec()).expect("Found invalid UTF-8");
        // tx.send(s).unwrap();

        // let mut buf = [0, 1];
        // let mut handle = io::stdin().take(1);
        // handle.read(&mut buf).unwrap();
        // let s = String::from_utf8(buf.to_vec()).expect("Found invalid UTF-8");
        // tx.send(s).unwrap();
    });
    rx
}
