build target:
    @echo 'Building target: {{target}}...'
    cargo zigbuild --target {{target}} --release -v
build-universal-mac-release:
    just build "universal2-apple-darwin"
build-x86_64-linux-release:
    just build "x86_64-unknown-linux-musl"
build-aarch64-linux-release:
    just build "aarch64-unknown-linux-musl"
