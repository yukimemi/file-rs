#![feature(attr_literals)]
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate walkdir;
extern crate threadpool;
extern crate regex;
#[macro_use]
extern crate lazy_static;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;
use structopt::StructOpt;
use walkdir::WalkDir;
use threadpool::ThreadPool;
use regex::Regex;

const WORKERS: usize = 8;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "a", long = "all", help = "Print all git directory")]
    all: bool,

    #[structopt(short = "f", long = "fetch", help = "Execute git fetch before check")]
    fetch: bool,

    #[structopt(long = "behind", help = "Print behind repo")]
    behind: bool,
    #[structopt(long = "ahead", help = "Print ahead repo")]
    ahead: bool,

    // Default is ghq root directory.
    #[structopt(required = false, help = "Input directory. default is $(ghq root) or '.'")]
    input: Option<String>,
}

#[derive(Debug, Clone)]
struct Gsr {
    pb: PathBuf,
    df: bool,
    st: Option<Output>,
    ahead: bool,
    behind: bool,
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

fn get_gsrs(walk_dir: WalkDir, fetch: bool) -> mpsc::Receiver<Gsr> {
    let (tx, rx) = mpsc::channel::<Gsr>();
    let pool = ThreadPool::new(WORKERS);
    thread::spawn(move || {
        walk_dir
            .into_iter()
            .map(|e| match e {
                Ok(e) => {
                    if e.file_name().to_str().unwrap_or("").eq(".git") {
                        let tx = tx.clone();
                        pool.execute(move || {
                            let parent = e.path().parent().unwrap();
                            let gsr = Gsr::new(parent);
                            if fetch {
                                gsr.fetch();
                            }
                            let gsr = gsr.status().diff().is_ahead().is_behind();
                            tx.send(gsr).unwrap();
                        });
                    }
                }
                Err(e) => eprintln!("{}", e),
            })
            .collect::<Vec<_>>();
        pool.join();
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
            ahead: false,
            behind: false,
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

    pub fn is_ahead(self) -> Self {
        lazy_static! { static ref RE: Regex = Regex::new(r"\[.*ahead.*\]").unwrap(); }
        if let Some(ref out) = self.st {
            return Gsr {
                ahead: RE.is_match(&String::from_utf8_lossy(&out.stdout).to_string()),
                ..self.clone()
            };
        }
        self
    }

    pub fn is_behind(self) -> Self {
        lazy_static! { static ref RE: Regex = Regex::new(r"\[.*behind.*\]").unwrap(); }
        if let Some(ref out) = self.st {
            return Gsr {
                behind: RE.is_match(&String::from_utf8_lossy(&out.stdout).to_string()),
                ..self.clone()
            };
        }
        self
    }
}

fn main() {
    let opt = Opt::from_args();

    let walk_dir = get_rootdir(&opt.input);

    let fetch = opt.fetch;
    let gsrs = get_gsrs(walk_dir, fetch);

    gsrs.into_iter()
        .map(|gsr| if opt.all {
            println!("{}", gsr.pb.display());
        } else {
            if gsr.df {
                println!("{}", gsr.pb.display());
            } else if opt.ahead && gsr.ahead {
                println!("{}", gsr.pb.display());
            } else if opt.behind && gsr.behind {
                println!("{}", gsr.pb.display());
            }
        })
        .collect::<Vec<_>>();
}
