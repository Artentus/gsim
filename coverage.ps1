$env:CARGO_INCREMENTAL=0
$env:RUSTFLAGS='-Cinstrument-coverage'
$env:LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw'
& cargo test
if (!(Test-Path -Path ./target/coverage/)) {
    New-Item -ItemType Directory -Path ./target/coverage/
}
& grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/tests.lcov
Remove-Item cargo-test-*-*.profraw
