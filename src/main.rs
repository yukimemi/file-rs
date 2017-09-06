extern crate clap;
extern crate walkdir;

use std::process::{Command, Output};
use clap::{Arg, App};
use walkdir::{WalkDir, DirEntry};

struct Gsr {
    entry: DirEntry,
    status: Option<Output>,
}

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
        // println!("{}", entry.unwrap().path().display());

        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() && path.ends_with(".git") {
            println!("{}", path.display());
        }
    }

}

impl Gsr {
    pub fn new(entry: DirEntry) -> Self {
        Gsr {
            entry: entry,
            status: None,
        }
    }
}
