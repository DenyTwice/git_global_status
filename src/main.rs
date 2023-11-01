#![allow(dead_code)]
#![allow(unused_imports)]

use std::{env, io};
use std::io::Error;
use std::path::{Path, PathBuf};

use git2::{Repository, StatusOptions};

// get directory name -> get each dierectory in that directory -> go to each directory in that directory -> run git status
// -> return formatted output

enum GitStatus {
    NoChanges,
    Modified,
    Staged,
    UnpushedCommits
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => { 
            driver(&args[1]);
        }
        _ => {
            println!("Usage: gg [PATH_TO_DIRECTORY]");
        }
    }
}

fn driver(path_string: &str) {
            let path = Path::new(&path_string);
            let directories = directories(&PathBuf::from(path)).expect("oops");
            
            for directory in directories {
                let repository = Repository::open(&directory).expect("won't happen indeed");
                match check_status(repository) {
                    GitStatus::NoChanges => println!("{} has no changes", directory.to_str().expect("hopefully doesn't happen")),
                    GitStatus::Modified => println!("{} has changes", directory.to_str().expect("hopefully doesn't happen either")),
                    GitStatus::Staged => println!("{} has staged changes", directory.to_str().expect("hopefully doesn't happen as well")),
                    GitStatus::UnpushedCommits => println!("{} has unpushed commits", directory.to_str().expect("hopefully doesn't happen as well")),
                }
            }
}

fn directories(path: &PathBuf) -> Result<Vec<PathBuf>, Error>{

    let mut directories: Vec<PathBuf> = Vec::new();
    for entry in path.read_dir()? {
        if let Ok(dir) = entry {
            if  dir.path().is_dir() {
                directories.push(dir.path());
            }
        }
    }
    
    Ok(directories)
}
    
fn check_status(repo: Repository) -> GitStatus {

    let mut opts = StatusOptions::new();
    opts.show(git2::StatusShow::IndexAndWorkdir);
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts)).expect("Could not retrieve statuses"); 

    for entry in statuses.iter() {
        let status = entry.status();


        if status.intersects(git2::Status::INDEX_NEW | git2::Status::INDEX_MODIFIED | git2::Status::INDEX_DELETED) {
            return GitStatus::Staged;
        }

        if status.intersects(git2::Status::WT_MODIFIED | git2::Status::WT_DELETED) {
            return GitStatus::Modified;
        }

        if has_commits_not_pushed(&repo) {
            return GitStatus::UnpushedCommits;
        }

    }

    GitStatus::NoChanges
}

// refactor so your eyes don't bleed when you look at this in the future
fn has_commits_not_pushed(repo: &Repository) -> bool {
    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => return false,
    };

    let branch_name = match head.shorthand() {
        Some(name) => name,
        None => return false,
    };

    let local_branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(branch) => branch,
        Err(_) => return false,
    };

    let upstream_branch = match local_branch.upstream() {
        Ok(branch) => branch,
        Err(_) => return false,
    };

    let local_oid = match repo.refname_to_id(local_branch.get().name().unwrap_or("")) {
        Ok(oid) => oid,
        Err(_) => return false,
    };

    let upstream_oid = match repo.refname_to_id(upstream_branch.get().name().unwrap_or("")) {
        Ok(oid) => oid,
        Err(_) => return false,
    };

    local_oid != upstream_oid
}
