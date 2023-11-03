use std::env;
use std::fs;
use std::fs::File;
use std::io::Error as IOError;
use std::io::ErrorKind;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

use git2::{Repository, StatusOptions, Error};

const USAGE: &str = "Usage: ggs [-d] [input]";
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
        1 => {
            let default_directory = match get_default_directory() {
                Ok(dir) => dir,
                Err(_) => {
                    println!("No defaults specified.\n{}", USAGE);
                    exit(1);
                }
            };
            driver(&default_directory);
        }
        2 => { 
            driver(&args[1]);
        }
        3 => {
            match set_default_directory(&args[2]) {
                Ok(()) => driver(&args[2]),
                Err(e) => println!("Error: {}. Could not set default directory.", e),
            }
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
            exit(1);
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


        if let Ok(false) = has_commits_not_pushed(&repo) {
            return Ok(GitStatus::UnpushedCommits);
        }

        if status.intersects(git2::Status::INDEX_NEW | git2::Status::INDEX_MODIFIED | git2::Status::INDEX_DELETED) {
            return Ok(GitStatus::Staged);
        }

        if status.intersects(git2::Status::WT_MODIFIED | git2::Status::WT_DELETED) {
            return Ok(GitStatus::Modified);
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

fn has_commits_not_pushed(repo: &Repository) -> Result<bool, Error> {
    let head = repo.head()?;
    let branch_name = head.shorthand().ok_or(Error::from_str("Branch name not found"))?;
    let local_branch = repo.find_branch(branch_name, git2::BranchType::Local)?;
    let upstream_branch = local_branch.upstream()?;

    let local_oid = repo.refname_to_id(local_branch.get().name().ok_or(Error::from_str("Local branch name not found"))?)?;
    let upstream_oid = repo.refname_to_id(upstream_branch.get().name().ok_or(Error::from_str("Upstream branch name not found"))?)?;

    Ok(local_oid != upstream_oid)
}

fn set_default_directory(path: &String) -> Result<(), IOError> {
 
    let home = match env::var("HOME") {
        Ok(val) => val,
        Err(e) => panic!("Couldn't read HOME environment variable ({})", e),
    };

    let mut config_path = PathBuf::from(home);
    config_path.push(".config/ggs/config.txt");

    if let Some(dir) = config_path.parent() {
        std::fs::create_dir_all(dir)?;
    } 

    let mut file = File::create(&config_path)?;
    file.write_all(path.as_bytes())?;
    Ok(())
}

fn get_default_directory() -> Result<String, IOError> {
    let home = match env::var("HOME") {
        Ok(val) => val,
        Err(e) => panic!("Couldn't read HOME environment variable ({})", e),
    };

    // Create a path using the HOME variable
    let mut config_path = PathBuf::from(home);
    config_path.push(".config/ggs/config.txt");

    let contents = fs::read_to_string(config_path)?;
    
    Ok(contents)
}
