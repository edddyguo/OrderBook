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

pub fn sha2562(data: String) -> [u8;32] {
    let mut buf = Vec::new();

    let items_buf = data.as_bytes();
    buf.extend(items_buf.iter().cloned());
    let buf256 = digest::digest(&digest::SHA256, &buf);
    let selic256 = buf256.as_ref();
    let mut data: [u8;32] = Default::default();
    for i in 0..32 {
        data[i] = selic256[i];
    }
    data
}
