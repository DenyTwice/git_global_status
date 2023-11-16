# git_global_status

CLI tool to check the git status of all repositories in the given directory.

## Installing

### From Releases
The binary should be a standalone executable so simply running it should be enough. Use aliases for ease of use and 
move the binary to /usr/bin or other directories in PATH for global access. Don't forget to make it executable.

## Compile from source
Refer to [The Rust Book](https://doc.rust-lang.org/cargo/getting-started/installation.html) for how to install Rust and Cargo using Rustup.

Run `cargo build` to get the compiled binary in `target/releases`. 

You can also install it, i.e compile and add to a directory in your PATH (usually ~/.cargo/bin), using `cargo install`.

## Usage

`ggs [-d] <path_to_directory> `
