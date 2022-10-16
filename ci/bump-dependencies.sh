
# GENERATED: This file is automatically updated by 'Bump dependencies', local changes will be overwritten!
set -eEu -o pipefail

function getconf() (
    printf '%s' "$(jq "$1" -r ci-conf.json)"
)

function update() (
    if [ "$(getconf .$1)" != "false" ]
    then
        echo "[UPDATE] $*"
        curl --silent "https://raw.githubusercontent.com/mverleg/ci_util/main/github_action/$2" --output - |\
            sed "1s/^/\n# GENERATED: This file is automatically updated by 'Bump dependencies', local changes will be overwritten\!\n/" >\
            "$3"
    else
        echo "[DISABLED] $*"
        rm -f "$3"
    fi
)

mkdir -p ./.github/workflows ./ci

update bump_dependencies 'bump-dependencies.yml' './.github/workflows/bump-dependencies.yml' &
update bump_dependencies 'bump-dependencies.Dockerfile' './ci/bump-dependencies.Dockerfile' &
update bump_dependencies 'bump-dependencies.sh' './ci/bump-dependencies.sh' &
update check_dependencies 'check-dependencies.yml' './.github/workflows/check-dependencies.yml' &
update check_dependencies 'check-dependencies.Dockerfile' './ci/check-dependencies.Dockerfile' &
update check_dependencies 'release.yml' './.github/workflows/release.yml' &
update check_dependencies 'release.Dockerfile' './ci/release.Dockerfile' &
update check_dependencies 'deny.toml' './deny.toml' &
update test_lint 'test-lint.yml' './.github/workflows/test-lint.yml' &
update test_lint 'test-lint.Dockerfile' './ci/test-lint.Dockerfile' &

wait
