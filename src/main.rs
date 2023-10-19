#![allow(dead_code)]
#![allow(unused_imports)]

use std::{env, io};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        
        2 => { 
            let path = Path::new(&args[1]);
            walk_dirs(&PathBuf::from(path));
        }
        _ => {
            println!("Usage: gg [PATH_TO_DIRECTORY]");
        }
    }
}

fn walk_dirs(path: &PathBuf) {

    match path.read_dir() {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("{}", entry.file_name().to_str().expect("ok"));
                }
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                println!("yeS");
            }
            else {
                println!("dont care");
            }
        }
    }
}
fn check_status() {}
