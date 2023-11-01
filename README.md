# git_global_status

This is a CLI tool for those who keep forgetting to commit/push their work on time. The idea is to run through a directory and find all
repositories that are not up to date. Currently, it works perfectly as long as the directory provided only contains repositories.

Usage: ggs <path_to_directory>

Move the binary in Releases to /usr/bin or other directories in PATH for global access. Don't forget to make it executable.

TODO:
- Improve format to be similar to one provided in Format.txt
- Implement better error handling to make it safe to run on larger directories that may or may not have repositories.
- Refactor the function that checks if there are commits pending to be pushed.
