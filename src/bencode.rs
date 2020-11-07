use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum BencodeData {
    Integer(u64),
    ByteString(Vec<u8>),
    List(Vec<BencodeData>),
    Dictionary(HashMap<String, BencodeData>)
}

impl BencodeData {
    pub fn get_int(self) -> Result<u64, &'static str> {
        match self {
            Self::Integer(int) => Ok(int),
            _ => Err("Invalid integer")
        }
    }

    pub fn get_byte_string(self) -> Result<Vec<u8>, &'static str> {
        match self {
            Self::ByteString(byte_string) => Ok(byte_string),
            _ => Err("Invalid byte string")
        }
    }

    pub fn get_list(self) -> Result<Vec<BencodeData>, &'static str> {
        match self {
            Self::List(list) => Ok(list),
            _ => Err("Invalid list")
        }
    }

    pub fn get_dict(self) -> Result<HashMap<String, BencodeData>, &'static str> {
        match self {
            Self::Dictionary(dict) => Ok(dict),
            _ => Err("Invalid dict")
        }
    }
}

pub struct BencodeDecoder;

impl BencodeDecoder {
    pub fn decode_int(pointer: &mut usize, data: &Vec<u8>) -> Result<BencodeData, &'static str> {
        if data[*pointer] as char != 'i' {
            return Err("Invalid bencode integer");
        }

        let mut int_str = String::new();

        *pointer += 1;
        while data[*pointer] as char != 'e' {
            let single_int_str = match String::from_utf8(vec![data[*pointer]]) {
                Ok(res) => res,
                Err(_) => return Err("Invalid/corrupt bencode")
            };
            int_str.push_str(&single_int_str);

            *pointer += 1;
        }
        *pointer += 1;

        match int_str.parse::<u64>() {
            Ok(res) => return Ok(BencodeData::Integer(res)),
            Err(_) => return Err("Invalid bencode integer")
        }
    }

    pub fn decode_byte_string(pointer: &mut usize, data: &Vec<u8>) -> Result<BencodeData, &'static str> {
        let mut data_len_str = String::new();

        while data[*pointer] as char != ':' {
            let single_len_str = match String::from_utf8(vec![data[*pointer]]) {
                Ok(res) => res,
                Err(_) => return Err("Invalid bencode string length")
            };
            data_len_str.push_str(&single_len_str);

            *pointer += 1;
        }

        let data_len = match data_len_str.parse::<i32>() {
            Ok(res) => res,
            Err(_) => return Err("Invalid bencode")
        };
        let mut byte_string: Vec<u8> = Vec::new();

        *pointer += 1;
        for pointer_offset in 0..data_len {
            byte_string.push(data[*pointer + pointer_offset as usize]);
        }

        *pointer += data_len as usize;

        Ok(BencodeData::ByteString(byte_string))
    }

    pub fn decode_list(pointer: &mut usize, data: &Vec<u8>) -> Result<BencodeData, &'static str> {
        if data[*pointer] as char != 'l' {
            return Err("Invalid bencode list");
        }

        let mut list: Vec<BencodeData> = Vec::new();

        *pointer += 1;
        while data[*pointer] as char != 'e' {
            let decoded_data_res = match data[*pointer] as char {
                'i' => BencodeDecoder::decode_int(pointer, &data),
                'l' => BencodeDecoder::decode_list(pointer, &data),
                'd' => BencodeDecoder::decode_dict(pointer, &data),
                _ => BencodeDecoder::decode_byte_string(pointer, &data),
            };

            let decoded_data = match decoded_data_res {
                Ok(res) => res,
                Err(err) => return Err(err)
            };

            list.push(decoded_data)
        }

        *pointer += 1;
        Ok(BencodeData::List(list))
    }

    pub fn decode_dict(pointer: &mut usize, data: &Vec<u8>) -> Result<BencodeData, &'static str> {
        if data[*pointer] as char != 'd' {
            return Err("Invalid bencode dictionary");
        }

        let mut dict: HashMap<String, BencodeData> = HashMap::new();

        *pointer += 1;
        while data[*pointer] as char != 'e' {
            let decoded_key = match BencodeDecoder::decode_byte_string(pointer, &data) {
                Ok(res) => res,
                Err(err) => return Err(err)
            };
            let decoded_key_vec = match decoded_key {
                BencodeData::ByteString(val) => val,
                _ => return Err("Invalid dictionary key")
            };
            let decoded_key_str = match String::from_utf8(decoded_key_vec) {
                Ok(res) => res,
                Err(_) => return Err("Invalid dictionary key"),
            };

            if decoded_key_str == "info" {
                dict.insert(String::from("info pointer"), BencodeData::Integer(*pointer as u64));
            }

            let decoded_val_res = match data[*pointer] as char {
                'i' => BencodeDecoder::decode_int(pointer, &data),
                'l' => BencodeDecoder::decode_list(pointer, &data),
                'd' => BencodeDecoder::decode_dict(pointer, &data),
                _ => BencodeDecoder::decode_byte_string(pointer, &data),
            };
            let decoded_val = match decoded_val_res {
                Ok(res) => res,
                Err(err) => return Err(err)
            };

            dict.insert(decoded_key_str, decoded_val);
        }
        *pointer += 1;

        Ok(BencodeData::Dictionary(dict))
    }

    pub fn get_from_dict(dict: &HashMap<String, BencodeData>, key: &str) -> Result<BencodeData, &'static str> {
        match dict.get(key) {
            Some(val) => Ok(val.clone()),
            None => Err("Invalid key")
        }
    }
}