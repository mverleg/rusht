
# Rusht

Small shell utilities coded in Rust.

## Install

Clone, build and install:

    git clone https://github.com/mverleg/rusht /tmp/rusht || cd /tmp/rusht ; git pull
    cargo install --bins --all-features --path /tmp/rusht

Or in minified mode:

	RUSTFLAGS="-C target-cpu=native" cargo +nightly install -Z build-std=std,panic_abort --target "$(rustc -vV | grep host | sed -E 's/.*: (.*)/\1/')" --path .

## Examples

* `cmadd`, `cmdo`, `cmlist`, `cmdrop` - push commands onto a stack, directly or from output, and run them one by one or all at once.
* `unique` - filters or shows duplicate lines, including duplicate by prefix (i.e. `/a` and `/a/b` are duplicate).
* `dir_with` - search for directories that contain specific files or subdirs, e.g. all the Rust projects or git repos.
* `cached` - cache the output of a command for a given amount of time.

