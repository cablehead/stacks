use base64::{engine::general_purpose, Engine as _};

pub fn b64decode(encoded: &str) -> Vec<u8> {
    general_purpose::STANDARD.decode(encoded).unwrap()
}

pub fn b64encode(s: &Vec<u8>) -> String {
    general_purpose::STANDARD.encode(s)
}
