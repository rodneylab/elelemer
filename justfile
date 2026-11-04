default:
    just --list

# find comments in Rust source
comments:
    rg --pcre2 -t rust '(^|\s+)(\/\/|\/\*)\s+(?!(act|arrange|assert))' .

# find expects and unwraps in Rust source
expects:
    rg --pcre2 -t rust '\.(expect\(.*\)|unwrap\(\))' .

# clean build and test artefacts
clean:
    rm -f elelemer-*.profraw 2>/dev/null
    cargo clean

integration-tests:
    cargo test --test '*'

# run coverage using grcov
coverage:
    just clean
    cargo build
    C_COMPILER=$(brew --prefix llvm)/bin/clang RUSTFLAGS="-Cinstrument-coverage" \
        LLVM_PROFILE_FILE="elelemer-%p-%m.profraw" cargo test
    grcov . -s . --binary-path ./target/debug/ \
        --llvm-path "$(brew --prefix llvm)"/bin -t html --branch \
        --ignore-not-existing -o ./target/debug/coverage/
    open --reveal ./target/debug/coverage
    sed -i '' "s|href=\"https://cdn.jsdelivr.net/npm/bulma@0.9.1/css/bulma.min.css\"|href=\"file://`pwd`/.cache/bulma.min.css\"|g" ./target/debug/coverage/**/*.html
    mkdir -p .cache
    curl --time-cond .cache/bulma.min.css -C - -Lo .cache/bulma.min.css \
      https://cdn.jsdelivr.net/npm/bulma/css/bulma.min.css

# generate docs for a crate and copy link to clipboard
doc crate:
    cargo doc -p {{ crate }}
    @echo "`pwd`/target/doc/`echo \"{{ crate }}\" | tr - _ \
        | sed 's/^rust_//' | sed -E 's/@[0-9\.]+$//' `/index.html" | pbcopy

# copy URL for Rust std docs to clipboard
std:
    @rustup doc --std --path | pbcopy

# review (accept/reject/...) insta snapshots
insta-snapshot-review:
    cargo insta review

# clean unreferenced insta snapshots
insta-snapshot-clean:
    cargo insta test --unreferenced=delete

# generate CLI markdown docs
markdown-docs:
    cargo run --features internal-tools -- markdown-help

# check links are valid
linkcheck:
    lychee --cache --max-cache-age 1d \
        --base-url https://github.com/rodneylab/elelemer/ --root-dir . \
        --exclude-path "deny.toml" . "**/*.toml" "**/*.rs" "**/*.yml"

# copy URL for Rust book to clipboard
book:
    @rustup doc --book --path | pbcopy

# dump trycmd snapshots (for review and manual copy)
trycmd-snapshot-dump:
    cargo build
    TRYCMD=dump cargo test
    @echo "trycmd dumped files should be in \`dump\`.  Copy manually to \`tests\` folders."

# overwrite trycmd snapshots
trycmd-snapshot-overwrite:
    cargo build
    TRYCMD=overwrite cargo test
