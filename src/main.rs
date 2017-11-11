#![feature(attr_literals)]
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate walkdir;
extern crate threadpool;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;
use structopt::StructOpt;
use walkdir::WalkDir;
use threadpool::ThreadPool;

const WORKERS: usize = 8;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "a", long = "all", help = "Print all git directory")]
    all: bool,

    #[structopt(short = "f", long = "fetch", help = "Execute git fetch before check")]
    fetch: bool,

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

fn get_rootdir(input: &Option<String>) -> WalkDir {
    match *input {
        Some(ref inp) => WalkDir::new(inp),
        None => {
            if let Ok(out) = Command::new("ghq").arg("root").output() {
                return WalkDir::new(String::from_utf8_lossy(&out.stdout).trim_right());
            }
            WalkDir::new(".")
        }
    }
}

fn get_gitdir(walk_dir: WalkDir) -> mpsc::Receiver<Gsr> {
    let (tx, rx) = mpsc::channel::<Gsr>();
    thread::spawn(move || {
        walk_dir
            .into_iter()
            .map(|e| match e {
                Ok(e) => {
                    if e.file_name().to_str().unwrap_or("").eq(".git") {
                        let parent = e.path().parent().unwrap();
                        tx.send(Gsr::new(parent)).unwrap();
                    }
                }
                Err(e) => eprintln!("{}", e),
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

    pub fn diff(self) -> Self {
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

    pub fn status(self) -> Self {
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

    pub fn fetch(&self) {
        Command::new("git")
            .current_dir(&self.pb)
            .arg("fetch")
            .output()
            .expect("failed to execute process");
    }
}

fn main() {
    let opt = Opt::from_args();

    let pool = ThreadPool::new(WORKERS);
    let walk_dir = get_rootdir(&opt.input);

    let gsrs = get_gitdir(walk_dir);
    let (tx, rx) = mpsc::channel::<Gsr>();

    // Get git status on all git directory.
    let fetch = opt.fetch;
    gsrs.into_iter()
        .map(|gsr| {
            let tx = tx.clone();
            pool.execute(move || {
                if fetch {
                    gsr.fetch();
                }
                let gsr = gsr.status().diff();
                tx.send(gsr).unwrap();
            });
        })
        .collect::<Vec<_>>();

    // Wait all threads.
    pool.join();
    drop(tx);

    rx.into_iter()
        .map(|gsr| if opt.all {
            println!("{}", gsr.pb.display());
        } else {
            if gsr.df {
                println!("{}", gsr.pb.display());
            }
        })
        .collect::<Vec<_>>();
}
