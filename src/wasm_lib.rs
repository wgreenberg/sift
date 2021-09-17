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
pub fn wasm_sift(args: String, sifter: &Sifter) -> String {
    let app = get_app().setting(AppSettings::NoBinaryName);
    let matches = app.get_matches_from(args.split(" "));
    let cmd = parse_command(&matches).unwrap();
    cmd.run(sifter).join("\n")
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
