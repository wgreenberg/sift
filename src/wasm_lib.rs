pub mod dictionary;
pub mod trie;
pub mod sifter;
pub mod sift_command;
#[cfg(test)] mod test_utils;
pub mod argparse;

use crate::sift_command::SiftCommand;
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

    fn ok_str(results: Vec<&str>) -> SifterResult {
        let owned_results = results.iter().map(|s| s.to_string()).collect();
        SifterResult { result: Ok(owned_results) }
    }

    fn ok(results: Vec<String>) -> SifterResult {
        SifterResult { result: Ok(results) }
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

enum Operation {
    Pipe(SiftCommand, SiftCommand),
    Single(SiftCommand),
}

fn parse_arg(args: &str) -> Result<SiftCommand, String> {
    let app = get_app().setting(AppSettings::NoBinaryName);
    app.get_matches_from_safe(args.split_whitespace()).map_err(|err| format!("{}", err))
        .and_then(|matches| parse_command(&matches).map_err(|err| format!("{:?}", err)))
}

fn parse_operation(args: String) -> Result<Operation, String> {
    let args: Vec<&str> = args.split("|").collect();
    match args.len() {
        1 => Ok(Operation::Single(parse_arg(args[0])?)),
        2 => Ok(Operation::Pipe(parse_arg(args[0])?, parse_arg(args[1])?)),
        _ => Err("too many pipes".into()),
    }
}

#[wasm_bindgen]
pub fn wasm_sift(args: String, sifter: &Sifter) -> SifterResult {
    match parse_operation(args) {
        Ok(Operation::Single(cmd)) => SifterResult::ok_str(cmd.run(sifter)),
        Ok(Operation::Pipe(first, second)) => {
            let mut results = Vec::new();
            for word in first.run(sifter) {
                for result in second.substitute(word).run(sifter) {
                    results.push(format!("{} => {}", word, result));
                }
            }
            SifterResult::ok(results)
        },
        Err(err) => return SifterResult::err(err),
    }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
