use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use sha1::{Sha1, Digest};
use urlencoding::encode;
use std::convert::TryInto;
use std::fmt::Write;
use std::net::Ipv4Addr;

use crate::bencode::*;
use crate::peer::*;

#[derive(Debug)]
pub struct TorrentInfo {
    pub length: u64,
    pub name: Vec<u8>,
    pub piece_length: u64,
    pub pieces: Vec<u8>,
}

#[derive(Debug)]
pub struct Torrent {
    pub announce: Vec<u8>,
    pub info: TorrentInfo,
    pub info_hash: Vec<u8>
}

impl Torrent {
    pub fn download(&self) {
        let mut hash_str = String::new();
        for elem in &self.info_hash {
            if (*elem as char).is_ascii_alphanumeric() {
                hash_str.push(*elem as char);
            } else {
                write!(hash_str, "%{:02x}", *elem).unwrap();
            }
        }

        let peer_id: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .collect();
        let peer_id = encode(&peer_id);

        let port = String::from("6881");
        let uploaded = String::from("0");
        let downloaded = String::from("0");
        let left = self.info.length.to_string();
        let compact = String::from("1");
        let event = String::from("started");

        let mut req_url = String::from_utf8(self.announce.clone()).unwrap();
        req_url.push('?');
        req_url.push_str("info_hash=");
        req_url.push_str(&hash_str);
        req_url.push('&');
        req_url.push_str("peer_id=");
        req_url.push_str(&peer_id);
        req_url.push('&');
        req_url.push_str("port=");
        req_url.push_str(&port);
        req_url.push('&');
        req_url.push_str("uploaded=");
        req_url.push_str(&uploaded);
        req_url.push('&');
        req_url.push_str("downloaded=");
        req_url.push_str(&downloaded);
        req_url.push('&');
        req_url.push_str("left=");
        req_url.push_str(&left);
        req_url.push('&');
        req_url.push_str("compact=");
        req_url.push_str(&compact);
        req_url.push('&');
        req_url.push_str("event=");
        req_url.push_str(&event);        

        println!("Tracker request: {}", req_url);

        let tracker_resp = reqwest::blocking::get(&req_url)
            .unwrap()
            .bytes()
            .unwrap()
            .to_vec();

        let mut pointer: usize = 0;
        let tracker_dict = BencodeDecoder::decode_dict(&mut pointer, &tracker_resp).unwrap();
        let tracker_dict = tracker_dict.get_dict().unwrap();

        let peers_bencode = BencodeDecoder::get_from_dict(&tracker_dict, "peers")
            .unwrap()
            .get_byte_string()
            .unwrap();

        let mut peers: Vec<Peer> = Vec::new();
        let mut pointer = 0;
        while pointer < peers_bencode.len() {
            let ip_1 = peers_bencode[pointer];
            let ip_2 = peers_bencode[pointer + 1];
            let ip_3 = peers_bencode[pointer + 2];
            let ip_4 = peers_bencode[pointer + 3];
            let ip = Ipv4Addr::new(ip_1, ip_2, ip_3, ip_4);

            let port_bytes = &peers_bencode[pointer+4..pointer+6];
            let port = u16::from_be_bytes(port_bytes.try_into().unwrap());

            let peer = Peer {
                ip,
                port,
                peer_id: peer_id.clone(),
                torrent: *self,
            };
            peers.push(peer);

            pointer += 6;
        }

        for peer in &peers {
            peer.handshake();
        }

    }

    pub fn from_file(contents: Vec<u8>) -> Result<Torrent, &'static str> {    
        let mut pointer: usize = 0;
        let decoded_file = BencodeDecoder::decode_dict(&mut pointer, &contents)?;
        let decoded_file = decoded_file.get_dict()?;

        let announce = BencodeDecoder::get_from_dict(&decoded_file, "announce").unwrap();
        let announce = announce.get_byte_string()?;

        let info_dict = BencodeDecoder::get_from_dict(&decoded_file, "info").unwrap();
        let info_dict = info_dict.get_dict()?;
 
        let length = BencodeDecoder::get_from_dict(&info_dict, "length").unwrap();
        let length = length.get_int()?;

        let name = BencodeDecoder::get_from_dict(&info_dict, "name").unwrap();
        let name = name.get_byte_string()?;

        let piece_length = BencodeDecoder::get_from_dict(&info_dict, "piece length").unwrap();
        let piece_length = piece_length.get_int()?;

        let pieces = BencodeDecoder::get_from_dict(&info_dict, "pieces").unwrap();
        let pieces = pieces.get_byte_string()?;

        let info_pointer = BencodeDecoder::get_from_dict(&decoded_file, "info pointer").unwrap();
        let info_pointer = info_pointer.get_int()?;

        let dict_start = info_pointer as usize;
        let info_dict = &contents[dict_start..contents.len() - 1];

        let mut hasher = Sha1::new();
        hasher.update(info_dict);
        let result_hash = hasher.finalize();

        let torrent_info = TorrentInfo {
            length: length,
            name: name,
            piece_length: piece_length,
            pieces: pieces,
        };

        let torrent = Torrent {
            announce: announce,
            info: torrent_info,
            info_hash: result_hash.to_vec(),
        };

        Ok(torrent)
    }
}