pub mod dictionary;
pub mod trie;
pub mod sifter;
pub mod sift_command;
#[cfg(test)] mod test_utils;
pub mod argparse;

use crate::sifter::Sifter;
use crate::argparse::{get_app, parse_command};
use wasm_bindgen::prelude::*;
use std::io::Cursor;
use clap::AppSettings;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn wasm_get_sifter(dict_data: Vec<u8>) -> Sifter {
    let cursor = Cursor::new(dict_data);
    Sifter::new_from_cache(cursor).unwrap()
}

#[wasm_bindgen]
pub struct SifterResult {
    result: Result<Vec<String>, String>,
}

#[wasm_bindgen]
impl SifterResult {
    fn err(message: String) -> SifterResult {
        SifterResult { result: Err(message) }
    }

    fn ok(results: Vec<&str>) -> SifterResult {
        let owned_results = results.iter().map(|s| s.to_string()).collect();
        SifterResult { result: Ok(owned_results) }
    }

    pub fn len(&self) -> usize {
        match &self.result {
            Ok(results) => results.len(),
            Err(_) => 0,
        }
    }

    pub fn to_string(&self, limit: usize) -> String {
        let limit = limit.min(self.len());
        match &self.result {
            Ok(results) => results[0..limit].join("\n"),
            Err(err) => err.clone(),
        }
    }
}

#[wasm_bindgen]
pub fn wasm_sift(args: String, sifter: &Sifter) -> SifterResult {
    let app = get_app().setting(AppSettings::NoBinaryName);
    let matches = match app.get_matches_from_safe(args.split(" ")) {
        Ok(matches) => matches,
        Err(err) => return SifterResult::err(format!("{}", err)),
    };
    match parse_command(&matches) {
        Ok(cmd) => SifterResult::ok(cmd.run(sifter)),
        Err(err) => SifterResult::err(format!("{:?}", err)),
    }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
