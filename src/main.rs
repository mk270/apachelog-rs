#[allow(dead_code)]

use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::str;
use clap::{Arg, App};
use std::io::Read;
use flate2::read::GzDecoder;

mod log_entry;

// read the contents of the file in `filename` and zcat them to the
// channel `tx`
fn zcat(filename: PathBuf, tx: Sender<Vec<u8>>) {
    let mut buffer = Vec::new();
    let mut f = File::open(filename).unwrap();
    f.read_to_end(&mut buffer).unwrap();

    let mut decoder = GzDecoder::new(&buffer[..]);
    let mut decoded_data = Vec::new();
    decoder.read_to_end(&mut decoded_data).unwrap();

    tx.send(decoded_data).unwrap();
}

fn split_lines(contents: Vec<u8>, tx: Sender<String>) {
    let s = str::from_utf8(&contents).unwrap();
    let lines = s.lines();
    for line in lines {
        let l2 = String::from(line);
        tx.send(l2).unwrap();
    }
}

fn process_dir(dir_name: String) -> std::io::Result<()> {
    let (tx1, rx1) = mpsc::channel();
    thread::spawn(move || {
        for entry in fs::read_dir(dir_name).unwrap() {
            let dir = entry.unwrap();
            tx1.send(dir.path()).unwrap();
        }
    });

    let (tx2, rx2) = mpsc::channel();
    thread::spawn(move || {
        for filename in rx1 {
            zcat(filename, tx2.clone());
        }
    });

    let (tx3, rx3) = mpsc::channel();
    thread::spawn(move || {
        for contents in rx2 {
            split_lines(contents, tx3.clone());
        }
    });

    for line in rx3 {
        let le = log_entry::parse_string(&line);
        println!("{}", le?.to_string());
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Apache Logs")
            .version("0.1.0")
            .author("Martin Keegan")
            .about("Parses Apache logs")
            .arg(Arg::with_name("dir")
                    .help("Path to access logs")
                    .short("d")
                    .takes_value(true)
                    .default_value("./logs"))
            .get_matches();

    let dir_name = String::from(matches.value_of("dir").unwrap());

    process_dir(dir_name)
}
