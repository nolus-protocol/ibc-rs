#!/usr/bin/env bash
#
# The fork's quality gate, copied from hermes-lite's ci/gate.sh and adapted. CI runs one
# stage per check; a developer runs the compile stages together, and the audit on its own:
#
#   RUSTFMT_TOOLCHAIN=nightly ci/gate.sh all     # fmt, lint-solray, lint-hermes, test, in order
#   ci/gate.sh <stage>                 # fmt | lint-solray | lint-hermes | test
#   ci/gate.sh audit                   # replace Cargo.lock with a fresh resolution, audit it
#   ci/gate.sh fetch DIR               # CI: resolve a fresh Cargo.lock, fetch what it names into DIR
#   ci/gate.sh --fetched DIR <stage>   # CI: run a stage offline on what `fetch` wrote to DIR
#
# The compile stages check what the consumers build, not every feature combination: the fork
# has no committed Cargo.lock and no target of its own beyond them. Locally cargo resolves
# with the developer's Cargo.lock (`cargo generate-lockfile` creates one) and network. CI
# splits the two halves into two containers. The fetch container has the network; it runs
# only `cargo generate-lockfile` and `cargo fetch --locked`, which run no crate code, with DIR
# as CARGO_HOME, and leaves the resolved Cargo.lock in DIR. The stage container has no
# network: it copies DIR's registry and Cargo.lock in and runs cargo offline, so a stage
# checks exactly what its own job's fetch resolved, and no build script, proc macro or test
# can reach the network. Each CI job fetches on its own, so two jobs of one run resolve
# differently only if a crate is published between them.
#
# The audit checks a fresh resolution, made the way `fetch` makes one, against the RustSec
# advisory database. `cargo generate-lockfile` and cargo-audit read manifests, the registry
# index, Cargo.lock and the database and run no crate code, so CI runs the audit in one
# container that keeps the network, and cargo-audit fetches the database itself. It fails on
# vulnerabilities only: cargo-audit reports unmaintained, unsound and yanked crates as
# warnings and exits zero on them unless told to deny them. It stays out of `all`, which
# checks the developer's own Cargo.lock: the audit needs the network and cargo-audit (the
# image's `audit` target), and it replaces Cargo.lock.

set -euo pipefail

usage() {
    echo "usage: ci/gate.sh fetch DIR | audit | [--fetched DIR] all|fmt|lint-solray|lint-hermes|test" >&2
    exit 2
}

# The 15 crates ibc-solray compiles and the features it enables on them, with default
# features off: ibc-solray's Cargo.toml, its nolus-protocol/ibc-rs dependency lines.
readonly SOLRAY_CRATES=(
    ibc-app-transfer
    ibc-app-transfer-types
    ibc-core-channel
    ibc-core-channel-types
    ibc-core-client
    ibc-core-client-context
    ibc-core-client-types
    ibc-core-commitment-types
    ibc-core-connection
    ibc-core-connection-types
    ibc-core-handler-types
    ibc-core-host
    ibc-core-host-types
    ibc-core-router
    ibc-primitives
)
readonly SOLRAY_FEATURES=(
    ibc-app-transfer/serde
    ibc-app-transfer-types/serde
    ibc-core-channel-types/borsh
    ibc-core-client-types/borsh
    ibc-core-commitment-types/borsh
    ibc-core-connection-types/borsh
    ibc-core-handler-types/borsh
    ibc-core-host/borsh
    ibc-core-host-types/borsh
    ibc-core-router/borsh
    ibc-primitives/borsh
    ibc-primitives/std
)

# hermes-lite's Cargo.toml takes ibc-data-types with its default features (["std"]) and
# ibc-core-host-cosmos without them; one invocation reproduces that feature unification.
readonly HERMES_CRATES=(ibc-data-types ibc-core-host-cosmos)
readonly HERMES_FEATURES=(ibc-data-types/std)

# Each consumer's crates, plus tests-integration, which carries the handler and event tests.
readonly TEST_CRATES=("${SOLRAY_CRATES[@]}" "${HERMES_CRATES[@]}" tests-integration)

# The image pins the compiler; the workspace states the version consumers are held to. A
# mismatch means one moved without the other.
assert_toolchain() {
    local want have
    want=$(sed -n 's/^rust-version *= *"\([^"]*\)"$/\1/p' Cargo.toml)
    have=$(rustc --version | cut -d' ' -f2)
    [[ -n "$want" ]] || { echo "error: no rust-version in Cargo.toml" >&2; return 1; }
    if [[ "$have" != "$want" ]]; then
        echo "error: rustc is $have but the workspace rust-version is $want" >&2
        return 1
    fi
}

fetch_into() {
    local dir=$1
    CARGO_HOME=$dir cargo generate-lockfile
    CARGO_HOME=$dir cargo fetch --locked
    cp Cargo.lock "$dir/Cargo.lock"
}

use_fetched_dependencies() {
    local dir=$1
    local home=${CARGO_HOME:?CARGO_HOME names where the fetched dependencies go}
    [[ -f "$dir/Cargo.lock" && -d "$dir/registry" ]] ||
        { echo "error: $dir holds no Cargo.lock and registry: did the fetch run?" >&2; return 1; }
    cp "$dir/Cargo.lock" Cargo.lock
    mkdir -p "$home/registry"
    cp -a --no-preserve=ownership "$dir/registry/." "$home/registry/"
    export CARGO_NET_OFFLINE=true
}

stage_fmt() {
    local nightly=${RUSTFMT_TOOLCHAIN:?RUSTFMT_TOOLCHAIN names the nightly that runs rustfmt (ci/Containerfile pins it)}
    cargo "+$nightly" fmt --all -- --check
    bash ci/code-quality/whitespace-lints.sh
}

stage_lint_solray() {
    local features
    features=$(IFS=,; echo "${SOLRAY_FEATURES[*]}")
    cargo clippy --locked --lib "${SOLRAY_CRATES[@]/#/--package=}" --no-default-features --features "$features" -- -D warnings
}

stage_lint_hermes() {
    local features
    features=$(IFS=,; echo "${HERMES_FEATURES[*]}")
    cargo clippy --locked --lib "${HERMES_CRATES[@]/#/--package=}" --no-default-features --features "$features" -- -D warnings
}

stage_test() {
    cargo test --locked "${TEST_CRATES[@]/#/--package=}"
}

stage_audit() {
    cargo generate-lockfile
    cargo audit
}

fetched=
if [[ "${1:-}" == --fetched ]]; then
    [[ $# -ge 2 ]] || usage
    fetched=$2
    shift 2
fi
[[ $# -ge 1 ]] || usage
case "$1" in
    fetch) [[ -z "$fetched" && $# -eq 2 ]] || usage ;;
    audit) [[ -z "$fetched" && $# -eq 1 ]] || usage; stages=(audit) ;;
    all) [[ $# -eq 1 ]] || usage; stages=(fmt lint-solray lint-hermes test) ;;
    fmt | lint-solray | lint-hermes | test) [[ $# -eq 1 ]] || usage; stages=("$1") ;;
    *) usage ;;
esac

cd "$(dirname "${BASH_SOURCE[0]}")/.."
echo "== $(cargo --version), $(rustc --version)"
assert_toolchain
if [[ "$1" == fetch ]]; then
    fetch_into "$2"
    echo "== resolved a fresh Cargo.lock and fetched every dependency it names"
    exit 0
fi
if [[ -n "$fetched" ]]; then
    use_fetched_dependencies "$fetched"
fi
for stage in "${stages[@]}"; do
    echo "== gate stage: $stage"
    case "$stage" in
        fmt) stage_fmt ;;
        lint-solray) stage_lint_solray ;;
        lint-hermes) stage_lint_hermes ;;
        test) stage_test ;;
        audit) stage_audit ;;
    esac
done
echo "== gate passed: ${stages[*]}"
