#![feature(attr_literals)]
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate walkdir;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;
use structopt::StructOpt;
use walkdir::WalkDir;

const VERSION: &str = "0.1.0";

#[derive(StructOpt, Debug)]
struct Opt {
    // A flag, true if used in the command line.
    #[structopt(short = "v", long = "version", help = "Show version")]
    version: bool,

    #[structopt(short = "a", long = "all", help = "Print all git directory")]
    all: bool,

    // Default is ghq root directory.
    #[structopt(required = false, help = "Input directory")]
    input: Option<String>,
}

#[derive(Debug)]
struct Gsr {
    pb: PathBuf,
    df: bool,
    st: Option<Output>,
}

fn main() {
    let opt = Opt::from_args();

    if opt.version {
        println!("{}", VERSION);
        return;
    }

    let input = get_rootdir(&opt.input);

    let rx = get_gitdir(input);
    rx.into_iter()
        .map(|gsr| {
            let gsr = gsr.get_status().check_diff();
            if opt.all {
                println!("{}", gsr.pb.display());
            } else {
                if gsr.df {
                    println!("{}", gsr.pb.display());
                }
            }
        })
        .collect::<Vec<_>>();
}

fn get_rootdir(input: &Option<String>) -> String {
    match *input {
        Some(ref inp) => inp.to_string(),
        None => {
            if let Ok(out) = Command::new("ghq").arg("root").output() {
                return String::from_utf8_lossy(&out.stdout)
                    .trim_right()
                    .to_string();
            }
            ".".to_string()
        }
    }
}

fn get_gitdir(path: String) -> mpsc::Receiver<Gsr> {
    let (tx, rx) = mpsc::channel::<Gsr>();
    thread::spawn(|| {
        let walker = WalkDir::new(path).into_iter();
        walker
            .map(|e| match e {
                Ok(e) => {
                    if e.file_name().to_str().unwrap_or("").eq(".git") {
                        let parent = e.path().parent().unwrap();
                        tx.send(Gsr::new(parent)).unwrap();
                    }
                }
                Err(e) => println!("{}", e),
            })
            .collect::<Vec<_>>();
        drop(tx);
    });
    rx
}

impl Gsr {
    pub fn new<P: AsRef<Path>>(p: P) -> Self {
        let mut pb = PathBuf::new();
        pb.push(p);
        Gsr {
            pb: pb,
            df: false,
            st: None,
        }
    }

    pub fn check_diff(self) -> Self {
        let df = Command::new("git")
            .current_dir(&self.pb)
            .arg("diff")
            .arg("--quiet")
            .status()
            .expect("failed to execute process");
        Gsr {
            df: !df.success(),
            ..self
        }
    }

    pub fn get_status(self) -> Self {
        let st = Command::new("git")
            .current_dir(&self.pb)
            .arg("status")
            .arg("--porcelain")
            .arg("--branch")
            .output()
            .expect("failed to execute process");
        Gsr {
            st: Some(st),
            ..self
        }
    }
}
