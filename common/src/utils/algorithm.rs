use ring::digest;

pub fn sha256(data: String) -> String {
    let mut buf = Vec::new();
    let mut txid = "".to_string();

    let items_buf = data.as_bytes();
    buf.extend(items_buf.iter().cloned());
    let buf256 = digest::digest(&digest::SHA256, &buf);
    let selic256 = buf256.as_ref();
    for i in 0..32 {
        let tmp = format!("{:x}", selic256[i]);
        txid += &tmp;
    }
    txid
}
pub fn u8_arr_to_str(data_arr: [u8; 32]) -> String {
    let mut data = "".to_string();
    for i in data_arr {
        let tmp = format!("{:0>2x}", i);
        data += &tmp;
    }
    data
}

pub fn u8_arr_from_str(data_str: String) -> [u8; 32] {
    let mut data: [u8; 32] = Default::default();
    let test1: Vec<char> = data_str.chars().collect();
    for x in test1.chunks(2).into_iter().enumerate() {
        let chars = format!("{}{}", x.1[0], x.1[1]);
        println!("char1 {}", chars);
        data[x.0] = u8::from_str_radix(chars.as_str(), 16).unwrap();
        println!("char2 {}", data[x.0]);
    }

    data
}

pub fn sha2562(data: String) -> [u8; 32] {
    let mut buf = Vec::new();

    let items_buf = data.as_bytes();
    buf.extend(items_buf.iter().cloned());
    let buf256 = digest::digest(&digest::SHA256, &buf);
    let selic256 = buf256.as_ref();
    let mut data: [u8; 32] = Default::default();
    for i in 0..32 {
        data[i] = selic256[i];
    }
    data
}
#[cfg(test)]
mod tests {
    use crate::utils::algorithm::{u8_arr_from_str, u8_arr_to_str};

    #[test]
    fn test_u8_arr_from_str() {
        let result1 = u8_arr_from_str(
            "d4bcd99699b2385f4582eaf64ef14346e01653923fd688c715a8562834ca6a".to_string(),
        );
        println!("result1 {:?}", result1);
        let result2 = u8_arr_to_str(result1);
        println!("result2 {}", result2);
    }
}
