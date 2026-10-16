use ahash::{AHashMap, AHashSet};
use js_sys::Array;
use ruzstd::decoding::StreamingDecoder;
use std::cell::RefCell;
use std::io::{BufRead, BufReader, Cursor};
use wasm_bindgen::prelude::*;

thread_local! {
    static WORDS: RefCell<AHashMap<u32, AHashSet<String>>> = RefCell::new(AHashMap::new());
}

#[wasm_bindgen]
pub fn load_wordlist(compressed: &[u8]) -> Result<usize, JsValue> {
    let decoder = StreamingDecoder::new(Cursor::new(compressed))
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let mut words = AHashMap::<u32, AHashSet<String>>::new();
    let mut count = 0;
    for word in BufReader::new(decoder).lines() {
        let word = word.map_err(|error| JsValue::from_str(&error.to_string()))?;
        words
            .entry(fnv32(word.as_bytes()))
            .or_default()
            .insert(word);
        count += 1;
    }

    WORDS.with(|loaded| *loaded.borrow_mut() = words);
    Ok(count)
}

#[wasm_bindgen]
pub fn lookup_hash(hash: u32) -> Array {
    WORDS.with(|loaded| {
        let words = loaded.borrow();
        words
            .get(&hash)
            .into_iter()
            .flatten()
            .map(|word| JsValue::from_str(word))
            .collect()
    })
}

fn fnv32(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c9dc5, |hash, byte| {
        hash.wrapping_mul(0x01000193) ^ u32::from(*byte)
    })
}
