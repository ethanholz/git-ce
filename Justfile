cargo-version := `cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].version'`

build target:
    cargo zigbuild --target {{target}} --release -v

release target version:
    mkdir -p release
    cp target/{{target}}/release/git-ce release/git-ce-{{target}}-{{version}}

build-and-release target version:
    just build {{target}}
    just release {{target}} {{version}}

build-release:
    just build-all {{cargo-version}}

build-all version:
    just build-and-release "universal2-apple-darwin" {{version}}
    just build-and-release "x86_64-unknown-linux-musl" {{version}}
    just build-and-release "aarch64-unknown-linux-musl" {{version}}

publish tag:
    echo "Publishing tag {{tag}}"
    gh release create {{tag}} --generate-notes release/*

pre-release:
    cargo check --release --locked --all-targets

sign:
    op read "op://Private/minisign key password/password" | minisign -Sm release/*

release-version:
    just pre-release
    just build-release
    just sign
    just publish "v{{cargo-version}}"
