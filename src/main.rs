extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate walkdir;

use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;
use structopt::StructOpt;
use walkdir::{WalkDir, WalkDirIterator, DirEntry, Result};

#[derive(StructOpt, Debug)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    // A flag, true if used in the command line.
    #[structopt(short = "v", long = "version", help = "Show version")]
    version: bool,

    // Needed parameter, the first on the command line.
    #[structopt(help = "Input directory")]
    input: String,
}

struct Gsr {
    entry: DirEntry,
    status: Option<Output>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let rx = get_gitdir(opt.input);
    rx.into_iter()
        .map(|e| println!("{}", e.entry.path().display()))
        .collect::<Vec<_>>();

}

fn get_gitdir(path: String) -> mpsc::Receiver<Gsr> {
    let (tx, rx) = mpsc::channel::<Gsr>();
    thread::spawn(move || {
        let walker = WalkDir::new(path).into_iter();
        walker
            .map(|e| match e {
                Ok(e) => {
                    if e.file_name().to_str().unwrap_or("").eq(".git") {
                        tx.send(Gsr::new(e)).unwrap();
                    }
                }
                Err(e) => println!("{}", e),
            })
            .collect::<Vec<_>>();
        drop(tx);
    });
    return rx;
}

fn is_gitdir(entry: &DirEntry) -> bool {
    println!("{:?}", entry);
    entry
        .file_name()
        .to_str()
        .map(|s| {
            println!("{}", s);
            s.eq(".git")
        })
        .unwrap_or(false)
}

impl Gsr {
    pub fn new(entry: DirEntry) -> Self {
        Gsr {
            entry: entry,
            status: None,
        }
    }
}
