use std::env;
use std::io::Error as IOError;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::exit;

use git2::{Repository, StatusOptions, Error};

const USAGE: &str = "Usage: ggs [PATH_TO_DIRECTORY]";
const ALL_GOOD: &str = "All good!";
const UNPUSHED_COMMITS_MSG: &str = "Directories with unpushed commits:";
const STAGED_CHANGES_MSG: &str = "Directories with staged changes:";
const MODIFIED_FILES_MSG: &str = "Directories with modified files:";

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
            println!("{}", USAGE);
        }
    }
}

fn driver(path_string: &str) {
    let path = Path::new(&path_string);
    let directories: Vec<PathBuf> = match list_directories(&PathBuf::from(path)) {
        Ok(dirs) => dirs,
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => println!("Directory not found."),
                ErrorKind::PermissionDenied => println!("Permission to access directory denied."),
                _ => println!("Error, could not read directory. Please check if given path points to a directory"),
            }
            exit(0);
        }
    };
    
    let mut modified: Vec<String> = Vec::new();
    let mut staged: Vec<String> = Vec::new();
    let mut unpushed_commits: Vec<String> = Vec::new();
    let mut no_changes: usize = 0;

    for directory in &directories {
        if let Ok(repository) = Repository::open(&directory) {

            let path = match directory.to_str() {
                        Some(str) => String::from(str),
                        None => continue,
            };

            match check_status(repository) {
                Ok(GitStatus::NoChanges) => no_changes += 1, 
                Ok(GitStatus::Modified) => modified.push(path),
                Ok(GitStatus::Staged) => staged.push(path),
                Ok(GitStatus::UnpushedCommits) => unpushed_commits.push(path),
                Err(_) => {
                    println!("Could not check status for {}", path);
                    continue
                },
            }
        }

    }
    if no_changes == directories.len() {
        println!("{}", ALL_GOOD);
        exit(0);
    }

    print_status(&unpushed_commits, UNPUSHED_COMMITS_MSG);
    print_status(&staged, STAGED_CHANGES_MSG);
    print_status(&modified, MODIFIED_FILES_MSG)

}

fn list_directories(path: &PathBuf) -> Result<Vec<PathBuf>,IOError>{

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
    
fn check_status(repo: Repository) -> Result<GitStatus, Error> {

    let mut opts = StatusOptions::new();
    opts.show(git2::StatusShow::IndexAndWorkdir);
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(status) => status,
        Err(error) => return Err(error),
    };

    for entry in statuses.iter() {
        let status = entry.status();

        if status.intersects(git2::Status::INDEX_NEW | git2::Status::INDEX_MODIFIED | git2::Status::INDEX_DELETED) {
            return Ok(GitStatus::Staged);
        }

        if status.intersects(git2::Status::WT_MODIFIED | git2::Status::WT_DELETED) {
            return Ok(GitStatus::Modified);
        }

        if has_commits_not_pushed(&repo) {
            return Ok(GitStatus::UnpushedCommits);
        }

    }

    Ok(GitStatus::NoChanges)
}

fn print_status(directories: &[String], message: &str) {
    if !directories.is_empty() {
        println!("{}", message);
        for directory in directories {
            println!("  * {}", directory);
        }
    }
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
