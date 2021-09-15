#[allow(dead_code)]

use clap::{Arg, App, SubCommand, ArgMatches};
use regex::Regex;
use atty::Stream;

use crate::sifter::Sifter;
use crate::sift_command::SiftCommand;
use std::io::{self, stdin, Read, Write};

mod dictionary;
mod trie;
mod sifter;
mod sift_command;
mod test_utils;

/* qhex functionality:
 *  - <regex>
 *  - anagram <letters>
 *  - transdelete [n] <letters>: anagram of letters after removing n chars (default 1)
 *  - transadd [n] <letters>: anagram of letters after adding n chars (default 1)
 *  - bank <letters>: words using the same set of letters
 *  - delete [n] <latters>: words achievable by deleting n letters (default 1)
 *  - add [n] <latters>: words achievable by adding n letters (default 1)
 *  - change [n] <letters>: achievable by exactly N letter changes (default 1)
 *
 * cross filtering:
 *   sift .{8} | sift anagram %
 *     runs "sift anagram" with the letters from each result of "sift .{8}", printing each pair
 *
 * output should be sorted by a score, and we should only print the top N results by default
 */

fn main() {
    let n_arg = Arg::with_name("n")
        .short("n")
        .default_value("1")
        .takes_value(true);
    let letters_arg = Arg::with_name("letters").index(1);
    let matches = App::new("sift")
        .arg(Arg::with_name("cache")
            .help("Path to a cached dictionary file")
            .short("c")
            .long("cache")
            .takes_value(true))
        .arg(Arg::with_name("dictionary")
            .help("Path to a word list")
            .short("d")
            .long("dict")
            .takes_value(true))
        .arg(Arg::with_name("regex")
            .help("regular expression")
            .index(1))
        .subcommand(SubCommand::with_name("create-cache")
            .about("create a dictionary cache file")
            .arg(Arg::with_name("dict-path")
                .help("path to dictionary file")
                .index(1))
            .arg(Arg::with_name("output-path")
                .help("path where cached file will reside")
                .index(2)))
        .subcommand(SubCommand::with_name("anagram")
            .about("anagram of the letters")
            .arg(letters_arg.clone()))
        .subcommand(SubCommand::with_name("transdelete")
            .about("anagram of the letters after deleting n chars")
            .arg(n_arg.clone())
            .arg(letters_arg.clone()))
        .subcommand(SubCommand::with_name("delete")
            .about("words achievable after deleting n chars")
            .arg(n_arg.clone())
            .arg(letters_arg.clone()))
        .subcommand(SubCommand::with_name("transadd")
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
        .get_matches();

    if let ("create-cache", Some(sub_m)) = matches.subcommand() {
        let dict_path = sub_m.value_of("dict-path").unwrap();
        let cache_path = sub_m.value_of("output-path").unwrap();
        let sifter = Sifter::new_from_dict_path(dict_path).unwrap();
        sifter.save_cache(cache_path).unwrap();
        println!("dictionary cache of {} created at {}", dict_path, cache_path);
        return;
    }

    let sifter = load_sifter(&matches).unwrap();

    match parse_command(&matches) {
        Ok(command) => run(&sifter, command),
        Err(err) => eprintln!("{:?}", err),
    }
}

#[derive(Debug)]
pub enum SiftError {
    InvalidRegExp,
    InvalidCharacters,
    InvalidCommand,
    MissingLetters,
    InvalidNumber,
}

fn print(out: &mut io::Stdout, message: &str) {
    if let Err(e) = writeln!(out, "{}", message) {
        match e.kind() {
            io::ErrorKind::BrokenPipe => std::process::exit(0),
            _ => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

fn run<'a>(sifter: &'a Sifter, command: SiftCommand) {
    let mut stdout = io::stdout();
    eprintln!("{}", atty::is(Stream::Stdout));
    let being_piped_to = !atty::is(Stream::Stdin);
    let being_piped_from = !atty::is(Stream::Stdout);

    if being_piped_to {
        let mut input = String::new();
        stdin().read_to_string(&mut input).unwrap();
        for word in input.lines() {
            let subbed_command = command.substitute(word);
            for result in subbed_command.run(sifter) {
                if being_piped_from {
                    print(&mut stdout, &format!("{}", word));
                } else {
                    print(&mut stdout, &format!("{} => {}", word, result));
                }
            }
        }
    } else {
        for word in command.run(sifter) {
            print(&mut stdout, &format!("{}", word));
        }
    }
}

fn load_sifter(matches: &ArgMatches) -> io::Result<Sifter> {
    if let Some(path) = matches.value_of("cache") {
        Sifter::load_cached(path)
    } else if let Some(path) = matches.value_of("dictionary") {
        Sifter::new_from_dict_path(path)
    } else {
        Ok(Sifter::new())
    }
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

fn parse_command(matches: &ArgMatches) -> Result<SiftCommand, SiftError> {
    match matches.subcommand() {
        ("anagram", Some(sub_m)) => Ok(SiftCommand::Anagram(get_letters(sub_m)?)),
        ("transdelete", Some(sub_m)) => Ok(SiftCommand::Transdelete(get_letters(sub_m)?, get_n(sub_m)?)),
        ("delete", Some(sub_m)) => Ok(SiftCommand::Delete(get_letters(sub_m)?, get_n(sub_m)?)),
        ("bank", Some(sub_m)) => Ok(SiftCommand::Bank(get_letters(sub_m)?)),
        ("transadd", Some(sub_m)) => Ok(SiftCommand::Transadd(get_letters(sub_m)?, get_n(sub_m)?)),
        ("add", Some(sub_m)) => Ok(SiftCommand::Add(get_letters(sub_m)?, get_n(sub_m)?)),
        ("change", Some(sub_m)) => Ok(SiftCommand::Change(get_letters(sub_m)?, get_n(sub_m)?)),
        (_, Some(_)) => Err(SiftError::InvalidCommand),
        (_, None) => Ok(SiftCommand::RegExp(get_regex(matches)?)),
    }
}
