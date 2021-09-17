pub mod dictionary;
pub mod trie;
pub mod sifter;
pub mod sift_command;
#[cfg(test)] mod test_utils;
pub mod argparse;

use clap::{Arg, SubCommand, ArgMatches};
use atty::Stream;
use crate::sifter::Sifter;
use crate::sift_command::SiftCommand;
use crate::argparse::{SiftError, get_app, parse_command};
use std::io::{self, stdin, Read, Write};

fn main() {
    let app = get_app()
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
        .subcommand(SubCommand::with_name("create-cache")
            .about("create a dictionary cache file")
            .arg(Arg::with_name("dict-path")
                .help("path to dictionary file")
                .index(1))
            .arg(Arg::with_name("output-path")
                .help("path where cached file will reside")
                .index(2)));

    let matches = app.get_matches();

    if let ("create-cache", Some(sub_m)) = matches.subcommand() {
        let dict_path = sub_m.value_of("dict-path").unwrap();
        let cache_path = sub_m.value_of("output-path").unwrap();
        let sifter = Sifter::new_from_words_file(dict_path).unwrap();
        sifter.save_cache_file(cache_path).unwrap();
        println!("dictionary cache of {} created at {}", dict_path, cache_path);
        return;
    }

    let sifter = load_sifter(&matches).unwrap();

    match parse_command(&matches) {
        Ok(command) => run(&sifter, command),
        Err(err) => eprintln!("{:?}", err),
    }
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

fn run(sifter: &Sifter, command: SiftCommand) {
    let mut stdout = io::stdout();
    let being_piped_to = !atty::is(Stream::Stdin);

    if being_piped_to {
        let mut input = String::new();
        stdin().read_to_string(&mut input).unwrap();
        for word in input.lines() {
            let subbed_command = command.substitute(word);
            for result in subbed_command.run(sifter) {
                print(&mut stdout, &format!("{} => {}", word, result));
            }
        }
    } else {
        for word in command.run(sifter) {
            print(&mut stdout, &format!("{}", word));
        }
    }
}

fn load_sifter(matches: &ArgMatches) -> Result<Sifter, SiftError> {
    if let Some(path) = matches.value_of("cache") {
        Sifter::new_from_cache_file(path)
    } else if let Some(path) = matches.value_of("dictionary") {
        Sifter::new_from_words_file(path)
    } else {
        Sifter::new_from_words_file("/etc/dictionaries-common/words")
    }
}
