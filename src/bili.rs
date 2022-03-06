use miniz_oxide::inflate;
use regex::Regex;
use tracing::{error, info};

pub struct BiliPacket{
    packet_len: i32,
    header_len: i32,
    version: i32,
    pub op: i32,
    seq: i32,
    pub body: Vec<Option<String>>,
}



pub fn read_int(buffer: &Vec<u8>, start: i32, len: i32) -> i32 {
    let mut i = len - 1;
    let mut result = 0;
    while i >= 0 {
        result += 256i32.pow((len - i - 1) as u32) * buffer[(start + i) as usize] as i32;
        i -= 1;
    }
    result
}

pub fn write_int(mut buffer: Vec<u8>, start: i32, len: i32, value: i32) -> Vec<u8> {
    let mut i = 0;
    while i < len {
        buffer[(start + i) as usize] = (value / 256i32.pow((len - i - 1) as u32) as i32) as u8;
        i += 1;
    }
    buffer
}

pub fn encode(raw_str: &str, op: i32) -> Vec<u8> {
    let data = raw_str.as_bytes();
    let packet_len = data.len() as i32 + 16;
    let header= [0, 0, 0, 0, 0, 16, 0, 1, 0, 0, 0, op as u8, 0, 0, 0, 1];
    let mut packet = write_int(header.to_vec(), 0, 4, packet_len);
    packet.extend(data);
    packet
}

pub fn decode(buffer: Vec<u8>) -> BiliPacket {
    let mut result = BiliPacket {
        packet_len: read_int(&buffer, 0, 4),
        header_len: read_int(&buffer, 4, 2),
        version: read_int(&buffer, 6, 2),
        op: read_int(&buffer, 8, 4),
        seq: read_int(&buffer, 12, 4),
        body: Vec::new(),
    };
    match result.op {
        5 => {
            let mut offset: i32 = 0;
            while offset < buffer.len() as i32 {
                let packet_len = read_int(&buffer, offset, 4);
                let header_len = 16;
                let _start = offset + header_len;
                let _end = offset + packet_len;
                let data = &buffer[_start as usize.._end as usize];
                let decoded = inflate::decompress_to_vec_zlib(&data);
                match decoded {
                    Ok(decoded) => {
                        let decoded_str = String::from_utf8_lossy(&decoded);
                        let seperator = Regex::new(r"[\x00-\x1f]+").expect("Invalid regex");
                        for item in reg_split(&seperator, &decoded_str) {
                            if item.contains("{"){
                                result.body.push(Some(item.to_string()));
                            }
                        }
                    }
                    Err(_) => {
                        error!("decode error");
                    }
                }
                offset += packet_len;
            }
        },
        3 => {
            let count = read_int(&buffer, 16, 4);
            let fragment = "{\"count\": ".to_string() + &count.to_string() + "}";
            result.body.push(Some(fragment));

        },

        _ => {
            //
        },
    }
    result
}

fn reg_split<'a>(r: &Regex, text: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last = 0;
    for (index, matched) in text.match_indices(r) {
        if last != index {
            result.push(&text[last..index]);
        }
        result.push(matched);
        last = index + matched.len();
    }
    if last < text.len() {
        result.push(&text[last..]);
    }
    result
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn test_read_int() {
        let buffer = [
            0, 0, 0, 26, 0, 16, 0, 1, 0, 0, 0, 8, 0, 0, 0, 1, 123, 34, 99, 111, 100, 101, 34, 58,
            48, 125,
        ];
        let result = 26;
        assert_eq!(read_int(&buffer.to_vec(), 0, 4), result);
    }

    #[test]
    fn test_write_int() {
        let buffer = [0, 0, 0, 0, 0, 16, 0, 1, 0, 0, 0, 7, 0, 0, 0, 1];
        let _start = 0;
        let _len = 4;
        let _value = 34;
        let result = write_int(buffer.to_vec(), _start, _len, _value);
        assert_eq!(result, [0, 0, 0, 34, 0, 16, 0, 1, 0, 0, 0, 7, 0, 0, 0, 1]);
    }
    
    #[test]
    fn _encode() {
        let raw_str = "{\"roomid\":3470615}";
        let _ok = vec![0u8, 0, 0, 34, 0, 16, 0, 1, 0, 0, 0, 7, 0, 0, 0, 1, 123, 34, 114, 111, 111, 109, 105, 100, 34, 58, 51, 52, 55, 48, 54, 49, 53, 125];
        assert_eq!(encode(raw_str, 7), _ok);
    }

    #[test]
    fn _decode() {
        let buffer = vec![0u8];
        decode(buffer);
    }

}
