_default:
    bat justfile -l yaml

run:
    cargo run

check:
    cargo check

update:
    cargo update

nightly-build:
    CARGO_PROFILE_DEV_CODEGEN_BACKEND=cranelift cargo +nightly build -Zcodegen-backend

timings:
    cargo build --timings

duplicates:
    cargo tree --duplicate

unused:
    cargo machete
