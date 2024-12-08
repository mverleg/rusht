
# Rusht

Small shell utilities coded in Rust.

## Install

Clone, build and install:

    git clone https://github.com/mverleg/rusht /tmp/rusht || cd /tmp/rusht ; git pull
    cargo install --bins --all-features --path /tmp/rusht

Or in minified mode:

	RUSTFLAGS="-C target-cpu=native" cargo +nightly install -Z build-std=std,panic_abort --target "$(rustc -vV | grep host | sed -E 's/.*: (.*)/\1/')" --path .

## Tools

Each of these has its own `--help` for more info

### Top

* `cached`    Cache the output of a command for a given duration, running it only if there is no cache or it has expired. Stderr is only shown on first run.
* `unique`    Remove any duplicate lines, keeping the first match and preserving order unless sorting is requested.
* `namesafe`  Convert each line to a string that is safe for names (no whitespace or special characters, not too long).
* `mon`       Log the command, the outcome, timings and play a sound.
* `grab`      Filter lines by regular expression, keeping only the matching capture group.

### Other

* `cmadd`, `cmdo`, `cmlist`, `cmdrop` - push commands onto a stack, directly or from output, and run them one by one or all at once.
* `cmbuf`     Read input, build commands and buffer them, then run them all. Somewhat like xargs. See also cmadd, cmdo, cmlist, cmdrop
* `dir_with`  Find directories that contain certain files or directories.
* `filter`    Run a test command for each line, keeping the file if the command succeeds
* `locked`    Do not start a command until a given lock is released.
* `mvnw`      Wrapper for maven (daemon) to add speed flags. Needs maven and uses git.
* `pomp`      Parse a tiny subset of pomfiles.
* `piped`     Split into two commands, and pipe the output of the first into the second.
* `batched`   Split stdin into groups of N lines, and pass them to the command
* `jl`        A mix of ls and find that outputs json (use jq)
* `between`   Select all lines between two matches

