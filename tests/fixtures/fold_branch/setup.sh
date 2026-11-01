#!/bin/sh
# Builds the rust-fold-history fixture: a scratch git repo whose `main` holds
# a copy of tests/fixtures/sensor_kit and whose `feat/sensor-fold` branch
# carries a spec, a three-task plan, the three task commits and eight
# refinement-loop fix-ups. See tests/fixtures/README.md.
#
# Usage: setup.sh <target-dir> [--dirty] [--remote]
set -eu

usage() {
    echo "usage: $0 <target-dir> [--dirty] [--remote]" >&2
    exit 2
}

[ $# -ge 1 ] && [ $# -le 3 ] || usage
target=$1
shift
dirty=no
remote=no
for flag in "$@"; do
    case $flag in
        --dirty) dirty=yes ;;
        --remote) remote=yes ;;
        *) usage ;;
    esac
done

here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
kit=$here/../sensor_kit
commits=$here/commits
created=no
bare=
bare_created=no

if [ -e "$target" ]; then
    if [ ! -d "$target" ]; then
        echo "$0: $target exists and is not a directory" >&2
        exit 1
    fi
    if [ -n "$(ls -A "$target")" ]; then
        echo "$0: refusing to build into non-empty $target" >&2
        exit 1
    fi
else
    mkdir -p -- "$target"
    created=yes
fi
target=$(CDPATH='' cd -- "$target" && pwd)

if [ "$remote" = yes ]; then
    bare=$target.remote.git
    if [ -e "$bare" ]; then
        echo "$0: refusing to build the remote over existing $bare" >&2
        [ "$created" = yes ] && rm -rf -- "$target"
        exit 1
    fi
fi

# A build that fails partway leaves nothing behind in a directory it made.
cleanup() {
    status=$?
    if [ "$status" -ne 0 ]; then
        [ "$created" = yes ] && rm -rf -- "$target"
        [ "$bare_created" = yes ] && rm -rf -- "$bare"
    fi
    exit "$status"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

g() {
    git -C "$target" -c user.name="Fixture Author" \
        -c user.email="fixture@example.invalid" \
        -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@"
}

# main: a copy of sensor_kit, without its build output or lockfile.
for entry in "$kit"/* "$kit"/.[!.]*; do
    [ -e "$entry" ] || continue
    case ${entry##*/} in
        target | Cargo.lock) continue ;;
    esac
    cp -R "$entry" "$target"/
done
printf '/target\nCargo.lock\n' >"$target/.gitignore"

g init -q
g symbolic-ref HEAD refs/heads/main
g add -A
g commit -q -m "Import sensor_kit"

# feat/sensor-fold: one commit per directory under commits/, in name order.
# Each directory holds `message`, a `tree/` copied over the repo, and
# optionally `lib-mods`, the `pub mod` lines put before sensor_kit's lib.rs.
g checkout -q -b feat/sensor-fold
for dir in "$commits"/*/; do
    dir=${dir%/}
    cp -R "$dir/tree/." "$target"/
    if [ -f "$dir/lib-mods" ]; then
        {
            cat "$dir/lib-mods"
            printf '\n'
            cat "$kit/src/lib.rs"
        } >"$target/src/lib.rs"
    fi
    g add -A
    g commit -q -F "$dir/message"
done

if [ "$remote" = yes ]; then
    git init -q --bare -- "$bare"
    bare_created=yes
    git -C "$bare" symbolic-ref HEAD refs/heads/main
    g remote add origin "$bare"
    g push -q -u origin main feat/sensor-fold
fi

# One uncommitted edit to a tracked file, for the dirty-tree scenario.
if [ "$dirty" = yes ]; then
    printf '\n// TODO: try a fixed array instead of the VecDeque.\n' \
        >>"$target/src/buffer.rs"
fi

echo "built $target on feat/sensor-fold"
