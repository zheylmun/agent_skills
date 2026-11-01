#!/bin/sh
# Builds the rust-refinement-loop fixture: a scratch git repo whose `main`
# holds a copy of tests/fixtures/sensor_kit and whose `feat/sensor-sampling`
# branch adds a seeded change. See tests/fixtures/README.md.
#
# Usage: setup.sh <target-dir> [--no-spec]
set -eu

usage() {
    echo "usage: $0 <target-dir> [--no-spec]" >&2
    exit 2
}

[ $# -ge 1 ] && [ $# -le 2 ] || usage
target=$1
with_spec=yes
if [ $# -eq 2 ]; then
    [ "$2" = "--no-spec" ] || usage
    with_spec=no
fi

here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
kit=$here/../sensor_kit
change=$here/change
created=no

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

# A build that fails partway leaves nothing behind in a directory it made.
cleanup() {
    status=$?
    if [ "$status" -ne 0 ] && [ "$created" = yes ]; then
        rm -rf -- "$target"
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

# feat/sensor-sampling: the change, in the commits a real branch would have.
g checkout -q -b feat/sensor-sampling

mkdir -p "$target/src" "$target/scripts"
cp "$change/src/sampling.rs" "$target/src/sampling.rs"
{
    printf 'pub mod sampling;\n\n'
    cat "$kit/src/lib.rs"
} >"$target/src/lib.rs"
cp "$change/scripts/watch-sampling.sh" "$target/scripts/watch-sampling.sh"
chmod +x "$target/scripts/watch-sampling.sh"
g add -A
g commit -q -m "Add a periodic sampler"

mkdir -p "$target/tests"
cp "$change/tests/sampling.rs" "$target/tests/sampling.rs"
g add -A
g commit -q -m "Test the sampler's happy path"

if [ "$with_spec" = yes ]; then
    mkdir -p "$target/docs/superpowers/specs"
    cp "$here/spec.md" "$target/docs/superpowers/specs/sampling-design.md"
    g add -A
    g commit -q -m "Sampling design"
fi

echo "built $target on feat/sensor-sampling"
