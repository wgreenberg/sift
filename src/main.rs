#[allow(dead_code)]

use clap::{Arg, App, SubCommand};

mod dictionary;
mod trie;
mod sifter;

/* qhex functionality:
 *  - <regex>
 *  - anagram <letters>
 *  - transdelete [n] <letters>: anagram of letters after removing n chars (default 1)
 *  - transadd [n] <letters>: anagram of letters after adding n chars (default 1)
 *  - bank <letters>: words using the same set of letters
 *  - substring <letters>: words contained in the substring
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
    let matches = App::new("sift")
        .arg(Arg::with_name("regex")
            .help("regular expression")
            .index(1))
        .subcommand(SubCommand::with_name("anagram")
            .about("anagram of the letters")
            .arg(Arg::with_name("letters")
                .index(1)))
        .subcommand(SubCommand::with_name("transdelete")
            .about("anagram of the letters after deleting n chars")
            .arg(Arg::with_name("n")
                .short("n")
                .default_value("1")
                .takes_value(true))
            .arg(Arg::with_name("letters")
                .index(1)))
        .subcommand(SubCommand::with_name("delete")
            .about("words achievable after deleting n chars")
            .arg(Arg::with_name("n")
                .short("n")
                .default_value("1")
                .takes_value(true))
            .arg(Arg::with_name("letters")
                .index(1)))
        .get_matches();

    let sifter = sifter::Sifter::new();
    let words = match matches.subcommand() {
        ("anagram", Some(sub_m)) => sifter.anagrams(sub_m.value_of("letters").unwrap()),
        ("transdelete", Some(sub_m)) => todo!(),
        ("delete", Some(sub_m)) => todo!(),
        _ => sifter.regex(matches.value_of("regex").unwrap()).unwrap(),
    };
    dbg!(words);
}
