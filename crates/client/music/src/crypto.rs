use aes::Aes128;
use ecb::Encryptor;
use ecb::cipher::{BlockEncryptMut, KeyInit, block_padding::Pkcs7};
use md5::{Digest, Md5};

pub(super) fn encrypt_request(path: &str, json: &str) -> String {
    // NetEase's eapi request envelope uses AES-128-ECB and a protocol MD5 digest.
    let digest = Md5::digest(format!("nobody{path}use{json}md5forencrypt").as_bytes());
    let digest: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    let text = format!("{path}-36cd479b6b5-{json}-36cd479b6b5-{digest}");
    let encrypted = Encryptor::<Aes128>::new(b"e82ckenh8dichen8".into())
        .encrypt_padded_vec_mut::<Pkcs7>(text.as_bytes());
    encrypted.iter().map(|byte| format!("{byte:02X}")).collect()
}
