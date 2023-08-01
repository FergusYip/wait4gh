# wait4gh

A program that waits for the latest commit of a pull request on GitHub to be in sync with the local repository. Useful for when you want to start a build once GitHub has processed your recent `git push`.

## Dependencies

- [GitHub CLI](https://github.com/cli/cli)


## Installation

```shell
 cargo install --git 'https://github.com/FergusYip/wait4gh' --locked --force 
```

## Example

```sh
git push && wait4gh && echo "Test command"
```
