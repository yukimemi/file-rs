extern crate clap;
extern crate walkdir;

use clap::{Arg, App};
use walkdir::WalkDir;

fn main() {
    let matches = App::new("Get all file and dir under the directory")
        .version("1.0")
        .author("yukimemi <yukimemi@gmail.com>")
        .arg(
            Arg::with_name("INPUT")
                .help("Get under the INPUT dir info")
                .required(true)
                .index(1),
        )
        .get_matches();

    let entries = WalkDir::new(matches.value_of("INPUT").unwrap());

    for entry in entries.into_iter() {
        println!("{}", entry.unwrap().path().display());
    }

}
