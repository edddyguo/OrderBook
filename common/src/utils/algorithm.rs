use std::str::FromStr;
use ethers_core::types::Signature;
use log::info;
use ring::digest;
use ring::{
    rand,
    signature::{self, KeyPair},
    test, test_file,
};
use ring::agreement::UnparsedPublicKey;
use ring::signature::EcdsaKeyPair;

pub fn sha256(data: String) -> String {
    let mut buf = Vec::new();
    let mut txid = "".to_string();

    let items_buf = data.as_bytes();
    buf.extend(items_buf.iter().cloned());
    let buf256 = digest::digest(&digest::SHA256, &buf);
    let selic256 = buf256.as_ref();
    for i in 0..32 {
        let tmp = format!("{:0>2x}", selic256[i]);
        txid += &tmp;
    }
    txid
}
pub fn u8_arr_to_str(data_arr: [u8; 32]) -> String {
    let mut data = "".to_string();
    let mut index = 0;
    for i in data_arr {
        let char = format!("{:0>2x}", i);
        index += 1;
        data += &char;
    }
    data
}

pub fn u8_arr_from_str(data_str: String) -> [u8; 32] {
    let mut data: [u8; 32] = Default::default();
    let test1: Vec<char> = data_str.chars().collect();
    for x in test1.chunks(2).into_iter().enumerate() {
        let chars = format!("{}{}", x.1[0], x.1[1]);
        data[x.0] = u8::from_str_radix(chars.as_str(), 16).unwrap();
    }

    data
}

pub fn test_singer(){
    /***
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes =
        signature::EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_ASN1_SIGNING,&rng).unwrap();
    let pri_key = pkcs8_bytes.as_ref();
    info!()
    let test2 =  EcdsaKeyPair::from_pkcs8(*alg,pri_key).unwrap();
    let private_key =
        signature::EcdsaKeyPair::from_private_key_and_public_key(signing_alg, &d, &q, &rng)
            .unwrap();

    let signature = private_key.
        .sign(&rng, &msg).unwrap();

    let signature = private_key.sign(&rng, &msg).unwrap();

    let public_key = signature::UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_ASN1_SIGNING, &rng);

    let public_key = signature::UnparsedPublicKey::new(verification_alg, q);
    assert_eq!(public_key.verify(&msg, signature.as_ref()), Ok(()));
     */
}


#[cfg(test)]
mod tests {
    use ring::{rand, signature};
    use ring::signature::{EcdsaKeyPair, KeyPair, Signature, UnparsedPublicKey};
    use crate::utils::algorithm::{u8_arr_from_str, u8_arr_to_str};
    use p256::{
        ecdsa::{SigningKey, Signature, signature::Signer},
    };
    use rand_core::OsRng; // requires 'getrandom' feature

    #[test]
    fn test_u8_arr_from_str() {
        let u8_arr = u8_arr_from_str(
            "d4bcd99699b2385f4582eaf64ef14346e01653923fd688c715a8562834ca6a11".to_string(),
        );
        let str = u8_arr_to_str(u8_arr);
        assert_eq!(
            str.as_str(),
            "d4bcd99699b2385f4582eaf64ef14346e01653923fd688c715a8562834ca6a11"
        );
    }

    #[test]
    fn test_sign(){
        //0x81f987ecec5ebce6ba1d3af4c6bef71203e242fd74656e3a1efd8fe3e1d351344acf48a8040a8bdafdd792e4c71186a990d703dfc97c02f56f65960a9c66c8cd1c
        //Example `personal_sign` message
        //0x3bb395b668ff9cb84e55aadfc8e646dd9184da9d
        /***
        let rng = rand::SystemRandom::new();
        let alg = &signature::ECDSA_P256_SHA256_ASN1_SIGNING;
        let pkcs8_bytes: ring::pkcs8::Document = EcdsaKeyPair::generate_pkcs8(alg, &rng).unwrap();
        let key_pair = EcdsaKeyPair::from_pkcs8(&alg, pkcs8_bytes.as_ref()).unwrap();
        const MESSAGE: &[u8] = b"hello, world";
        let sig = key_pair.sign(&rng, MESSAGE).unwrap();
        let sig_bytes2 = sig2.as_ref();
        let verify_res = peer_public_key2.verify(MESSAGE2, &sig_bytes2).unwrap();
        let peer_public_key_bytes = key_pair.public_key().as_ref();
        let peer_public_key = UnparsedPublicKey::new(
            &signature::ECDSA_P256_SHA256_ASN1, peer_public_key_bytes
        );
        */
        /***
        const MESSAGE2: &[u8] = b"hello, world2";
        println!("__0001_{:?}",MESSAGE2);
        let sig2 = Signature::try_from(b"0x81f987ecec5ebce6ba1d3af4c6bef71203e242fd74656e3a1efd8fe3e1d351344acf48a8040a8bdafdd792e4c71186a990d703dfc97c02f56f65960a9c66c8cd1c").unwrap();
        let peer_public_key2 = UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_ASN1,b"0x3bb395b668ff9cb84e55aadfc8e646dd9184da9d");
        let sig_bytes = sig2.as_ref();
        let sig_bytes2 = sig2.as_ref();
        let verify_res = peer_public_key2.verify(MESSAGE2, &sig_bytes2).unwrap();
        println!("___0002_{:?}",verify_res);
         */
    }

    #[test]
    fn test_ecdsa(){

    }
}
