#![allow(dead_code)]

use std::fmt::Write;

pub fn to_hex_without_spaces(b: impl AsRef<[u8]>) -> String {
    b.as_ref().iter().fold(String::new(), |mut s, b| {
        write!(s, "{b:02x}").unwrap();
        s
    })
}

pub fn to_hex_with_spaces(b: impl AsRef<[u8]>) -> String {
    let mut s = b.as_ref().iter().fold(String::new(), |mut s, b| {
        write!(s, "{b:02x} ").unwrap();
        s
    });
    s.pop();
    s
}

pub fn from_hex_without_spaces(s: &str) -> Vec<u8> {
    let s = s.trim();
    let mut result = Vec::with_capacity(s.len() / 2);
    for p in s.as_bytes().chunks_exact(2) {
        let p = std::str::from_utf8(p).expect("invalid hex string");
        result.push(u8::from_str_radix(p, 16).expect("invalid hex string"));
    }
    result
}

pub fn from_hex_with_spaces(s: &str) -> Vec<u8> {
    let s = s.trim();
    let mut result = Vec::with_capacity(s.len() / 3 + 1);
    for p in s.split(' ') {
        result.push(u8::from_str_radix(p, 16).expect("invalid hex string"));
    }
    result
}
