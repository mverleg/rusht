
//TODO @mverleg: hash of current state (including index and unstaged)
// 	files-affected-uncommitted = "!f() { set -eE; git diff --name-only --staged; git ls-files --others --exclude-standard; }; f"
// 	checksum-uncommitted = "!f() { set -eE; printf 'Q'; ( git rev-parse HEAD; git files-affected-uncommitted | filter test -f | sort | xargs sha256sum; ) | sha256sum | cut -c1-31; }; f"

//TODO @mverleg: all affected files in different scopes
// 	files-affected-branch = "!f() { set -eE; git diff-tree --no-commit-id --name-only -r \"$(git master-base)..\"; git files-affected-uncommitted; }; f"
// 	files-affected-head = diff-tree --no-commit-id --name-only -r HEAD
// 	files-affected-uncommitted = "!f() { set -eE; git diff --name-only --staged; git ls-files --others --exclude-standard; }; f"
