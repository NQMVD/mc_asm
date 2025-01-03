_default:
    bat justfile -l yaml

run:
    cargo run

test:
    cargo build
    ./target/debug/mc_asm assemble demo.as --old
    ./target/debug/mc_asm generate demo.mc
    ./target/debug/mc_asm assemble demo.as lol.mc --old

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
