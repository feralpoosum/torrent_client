use std::env;
use std::fs::File;
use std::io::prelude::*;

use torrent_client::torrent;

fn get_torrent_filename(mut args: std::env::Args) -> Result<String, &'static str> {
    args.next();

    match args.next() {
        Some(filename) => Ok(filename),
        None => Err("Error parsing arguments")
    }
}

fn main() {
    println!("torrent_client - v0.1\n");

    let torrent_filename = get_torrent_filename(env::args()).unwrap();

    let mut torrent_file = File::open(torrent_filename).unwrap();
    let mut file_contents = Vec::new();

    torrent_file.read_to_end(&mut file_contents).expect("Error reading torrent file");

    let torrent = torrent::Torrent::from_file(file_contents).unwrap();

    let announce_str = String::from_utf8(torrent.announce.clone()).unwrap();
    let name_str = String::from_utf8(torrent.info.name.clone()).unwrap();

    println!("Parsed torrent");
    println!("Announce: {}", announce_str);
    println!("Length: {}", torrent.info.length);
    println!("Name: {}", name_str);
    println!("Piece length: {}", torrent.info.piece_length);

    println!("\nDownloading...\n");

    torrent.download();
}