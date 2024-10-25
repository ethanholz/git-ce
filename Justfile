build target:
    cargo zigbuild --target {{target}} --release -v

release target:
    mkdir -p release
    cp target/{{target}}/release/git-ce release/git-ce-{{target}} 

build-and-release target:
    just build target={{target}}
    just release target={{target}}

build-and-release-all:
    just build-and-release "universal2-apple-darwin"
    just build-and-release "x86_64-unknown-linux-musl"
    just build-and-release "aarch64-unknown-linux-musl"

pre-release:
    cargo check --release --locked --all-targets


