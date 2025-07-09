# jj-watch

`jj-watch` is a simple command to run `jj log` over and over again.
I got tired of always forgetting the change ID of my revisions, so I created
this to stay open in a terminal window.

Configuration options and command-line arguments will come soon, but right now
it will just run `jj log` every two seconds without snapshotting.
The output gets piped in as is, so it shouldn't mess up your templates.

Feel free to file PRs.

## Packaging
There is a Nix flake for this package, which makes it available as `jj-watch`
and `jjw`. It should also work with regular `cargo` binary management:

```
cargo install jj-watch --git https://github.com/johnbchron/jj-watch
```
