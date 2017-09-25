extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate walkdir;
extern crate crossbeam;

use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;
use std::path::Path;
use std::ffi::OsStr;
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

struct Gsr<'a> {
    path: &'a Path,
    status: Option<Output>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let rx = get_gitdir(opt.input);
    rx.into_iter()
        // .map(|g| println!("{}", g.path.display()))
        .map(|p| println!("{}", p.display()))
        .collect::<Vec<_>>();

}

// fn get_gitdir<'a>(path: String) -> mpsc::Receiver<Gsr<'a>> {
fn get_gitdir<'a>(path: String) -> mpsc::Receiver<&'a Path> {
    // let (tx, rx) = mpsc::channel::<Gsr<'a>>();
    let (tx, rx) = mpsc::channel::<&'a Path>();
    crossbeam::scope(|scope| {
        scope.spawn(|| {
            let walker = WalkDir::new(path).into_iter();
            walker
                .map(|e| match e {
                    Ok(e) => {
                        if e.file_name().to_str().unwrap_or("").eq(".git") {
                            let parent = e.path().parent().unwrap();
                            // tx.send(Gsr::new(parent)).unwrap();
                            tx.send(parent).unwrap();
                        }
                    }
                    Err(e) => println!("{}", e),
                })
                .collect::<Vec<_>>();
            drop(tx);
        })
    });
    return rx;
}

impl<'a> Gsr<'a> {
    pub fn new<S: AsRef<OsStr> + ?Sized>(s: &'a S) -> Self {
        Gsr {
            path: Path::new(s),
            status: None,
        }
    }
}
