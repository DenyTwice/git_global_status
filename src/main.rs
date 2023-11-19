use std::env;
use std::io::Error as IOError;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::process::exit;

use git2::{Repository, StatusOptions, Error};

const HELP: &str = r#"
Usage: ggs [OPTIONS] [ARGUMENTS]
Scans through all the repositories in the configuration file and returns a concise git status message.  

Options:
    -h, --help              Show this help message and exit.
    -v, --version           Show the version number.
    -a, --add               Add a directory to scan for repositories and check statuses.

    -f, --file              Set a custom location for the configuration file. (NOT IMPLEMENTED)

Examples:
    ggs --add .             Adds the current directory's path to the config file. 
"#;
const USAGE: &str = "Usage: ggs [OPTIONS] [ARGUMENTS]";
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

enum CommandType<'a> {
    Default,
    DirectorySpecified(&'a str),
    Help,
    Add(&'a str),
    Invalid,
}

fn parse_input(mut args: Vec<&str>) -> CommandType {
    args.swap_remove(0); // remove ggs
    
    if args.is_empty() {
        return CommandType::Default;
    }

    if args.len() == 1 {
        match args[0] {
            "--help" | "-h" => return CommandType::Help,
            dir => return CommandType::DirectorySpecified(dir)
        }
    };

    if args[0] == "--add" || args[0] == "-a" {
        return CommandType::Add(args[1]); 
    } else if args[1] == "--add" || args[1] == "-a" {
        return CommandType::Add(args[0]);
    }

    if args.len() > 2 {
        return CommandType::Invalid; 
    }

    CommandType::Invalid
}

fn main() {
    let args: Vec<String> = env::args()
        .collect();

    let args_str: Vec<&str> = args.iter()
        .map(|s| s.as_str())
        .collect();

    let command_type: CommandType = parse_input(args_str);

    match command_type {
        CommandType::Help => print_message(HELP),
        CommandType::Invalid => print_message(USAGE),
        CommandType::Add(dir) => add_to_config(dir),
        CommandType::DirectorySpecified(dir) => (),
        CommandType::Default => ()
    }
}

fn print_message(message: &str) {
    println!("{}", message);
}

fn default(path_str: &str) {

    let directories = match list_directories(&PathBuf::from(path)) {
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

fn add_to_config(path: &str) {
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


        if has_commits_not_pushed(&repo) {
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

fn add(path: &String) -> Result<(), IOError> {
 
    let home = match env::var("HOME") {
        Ok(val) => val,
        Err(e) => panic!("Couldn't read HOME environment variable ({})", e),
    };

    let mut config_path = PathBuf::from(home);
    config_path.push(".config/ggs/config.txt");

    if let Some(dir) = config_path.parent() {
        std::fs::create_dir_all(dir)?;
    } 

    let mut file = std::fs::File::create(&config_path)?;
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

    let contents = std::fs::read_to_string(config_path)?;
    
    Ok(contents)
}
