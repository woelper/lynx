cargo install cargo-bump
cargo bump patch --git-tag
git add Cargo.toml
git push --tags
git push