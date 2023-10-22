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
            let files_list: Vec<PathBuf> = files(&PathBuf::from(path)); 
        }
        _ => {
            println!("Usage: gg [PATH_TO_DIRECTORY]");

        }
    }
}
fn files(path: &PathBuf) -> Result<Vec<PathBuf>, ErrorKind>{
    let mut file_list: Vec<PathBuf> = Vec::new();
    let entries = path.read_dir()?;
    for entry in entries {
        if let Ok(entry) = entry {
            file_list.push(PathBuf::from(entry.file_name()));
            println!("{}", entry.file_name().to_str().expect("ok"));
        }
    }
    Ok(file_list)
}

fn check_status() {}
