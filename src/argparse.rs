use clap::{Arg, App, SubCommand, ArgMatches};
use regex::Regex;
use crate::sift_command::SiftCommand;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
pub enum SiftError {
    InvalidRegExp,
    InvalidCharacters,
    InvalidCommand,
    MissingLetters,
    InvalidNumber,
    FileIOError,
    SerializationError,
    DeserializationError,
}

pub fn get_app() -> App<'static, 'static> {
    let n_arg = Arg::with_name("n")
        .short("n")
        .default_value("1")
        .takes_value(true);
    let letters_arg = Arg::with_name("letters").index(1);
    App::new("sift")
        .arg(Arg::with_name("regex")
            .help("regular expression")
            .index(1))
        .subcommand(SubCommand::with_name("anagram")
            .about("anagram of the letters")
            .arg(letters_arg.clone()))
        .subcommand(SubCommand::with_name("transpose-delete")
            .alias("td")
            .about("anagram of the letters after deleting n chars")
            .arg(n_arg.clone())
            .arg(letters_arg.clone()))
        .subcommand(SubCommand::with_name("delete")
            .about("words achievable after deleting n chars")
            .arg(n_arg.clone())
            .arg(letters_arg.clone()))
        .subcommand(SubCommand::with_name("transpose-add")
            .alias("ta")
            .about("words achievable after adding n chars")
            .arg(n_arg.clone())
            .arg(letters_arg.clone()))
        .subcommand(SubCommand::with_name("bank")
            .about("words using the same set of letters")
            .arg(letters_arg.clone()))
        .subcommand(SubCommand::with_name("delete")
            .about("words achievable by deleting n letters")
            .arg(n_arg.clone())
            .arg(letters_arg.clone()))
        .subcommand(SubCommand::with_name("add")
            .about("words achievable by adding n letters")
            .arg(n_arg.clone())
            .arg(letters_arg.clone()))
        .subcommand(SubCommand::with_name("change")
            .about("words achievable by changing n letters")
            .arg(n_arg.clone())
            .arg(letters_arg.clone()))
}

fn get_regex<'a>(matches: &'a ArgMatches) -> Result<Regex, SiftError> {
    let pattern = matches.value_of("regex").ok_or(SiftError::MissingLetters)?;
    Regex::new(pattern).map_err(|_| SiftError::InvalidRegExp)
}

fn get_letters<'a>(matches: &'a ArgMatches) -> Result<String, SiftError> {
    matches.value_of("letters").ok_or(SiftError::MissingLetters)
        .and_then(|s| Ok(s.to_string()))
}

fn get_n(matches: &ArgMatches) -> Result<usize, SiftError> {
    let n_str = matches.value_of("n").unwrap();
    str::parse::<usize>(n_str).map_err(|_| SiftError::InvalidNumber)
}

pub fn parse_command(matches: &ArgMatches) -> Result<SiftCommand, SiftError> {
    match matches.subcommand() {
        ("anagram", Some(sub_m)) => Ok(SiftCommand::Anagram(get_letters(sub_m)?)),
        ("transpose-delete", Some(sub_m)) => Ok(SiftCommand::TransposeDelete(get_letters(sub_m)?, get_n(sub_m)?)),
        ("delete", Some(sub_m)) => Ok(SiftCommand::Delete(get_letters(sub_m)?, get_n(sub_m)?)),
        ("bank", Some(sub_m)) => Ok(SiftCommand::Bank(get_letters(sub_m)?)),
        ("transpose-add", Some(sub_m)) => Ok(SiftCommand::TransposeAdd(get_letters(sub_m)?, get_n(sub_m)?)),
        ("add", Some(sub_m)) => Ok(SiftCommand::Add(get_letters(sub_m)?, get_n(sub_m)?)),
        ("change", Some(sub_m)) => Ok(SiftCommand::Change(get_letters(sub_m)?, get_n(sub_m)?)),
        (_, Some(_)) => Err(SiftError::InvalidCommand),
        (_, None) => Ok(SiftCommand::RegExp(get_regex(matches)?)),
    }
}
