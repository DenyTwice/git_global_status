use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error as IOError;
use std::io::{ErrorKind, Write};
use std::path::{PathBuf, Path};
use std::process::exit;

use git2::{Repository, StatusOptions, Error};

const HELP: &str = r#"
Usage: ggs [OPTIONS] [ARGUMENTS]
Scans through all the repositories in the configuration file and returns a concise git status message.  

Options:
    -h, --help              Show this help message and exit.
    -v, --version           Show the version number.
    -a, --add               Add a directory to scan for repositories and check statuses.

    -f, --file              Specify a custom location for the configuration file. (NOT IMPLEMENTED)

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

}

fn add_to_config(path_str: &str) {
    
    let path = Path::new(path_str);
    if !path.is_dir() {
        println!("Specified path is not a directory");
        exit(1);
    }

    let mut write_buff: Vec<PathBuf> = Vec::new();
    if let Ok(_) = Repository::open(path) {
        write_buff.push(PathBuf::from(path));
    } else {
        write_buff = dir_contents(&path.to_path_buf()); 
    }

    let home = match env::var("HOME") {
        Ok(val) => val,
        Err(e) => {
            println!("Could not read HOME variable from environment, {}", e);
            exit(0);
        }
    };

    let mut config_path = PathBuf::from(home);
    config_path.push(".config/ggs/config.txt");

    if let Some(dir) = config_path.parent() {
        match std::fs::create_dir_all(dir) {
            Ok(_) => (),
            Err(e) => {
                println!("Coud not create required directories for configuration file, {}", e);
                exit(1);
            }
        }
    } 

    let mut file = match std::fs::File::create(&config_path) {
        Ok(file) => file,
        Err(e) => {
            println!("ERROR: Could not create config file, {}", e);
            exit(1);
        }
    };

    for path in write_buff {
        let path_str = path.into_os_string()
            .into_string()
            .unwrap(); // TODO Proper Error hadnling 
        match file.write_all(path_str.as_bytes()) { 
            Ok(_) => (),
            Err(e) => {
                println!("Could not write {} into config file, {}", path_str, e);
            }
        }
    }
}

fn dir_contents(path: &PathBuf) -> Vec<PathBuf> {

    let mut directories: Vec<PathBuf> = Vec::new();
    let dir_content = match path.read_dir() {
        Ok(dir_content) => dir_content,
        Err(e) => {
            println!("ERROR: Failed to get directory contents, {}", e);
            exit(1);
        }
    };

    for entry in dir_content {
        if let Ok(dir) = entry  {
            if dir.path().is_dir() {
                directories.push(dir.path());
            }
        }
    }
    
    directories
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
