rm -rf _site
rm -rf obj
rm -rf rust-doc-tmp

sh copy_files.sh
docfx docfx.json
cd ../rust
cargo doc --no-deps --target-dir ../docfx/rust-doc-tmp
cd ../docfx
cp -r rust-doc-tmp/doc/* _site/api/rust/

