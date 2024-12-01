rm -rf _site
rm -rf obj
rm -rf rust-doc-tmp
rm -rf web-doc-tmp

sh copy_files.sh
docfx docfx-non-windows.json

cd ../rust
cargo doc --no-deps --target-dir ../docfx/rust-doc-tmp

cd ../docfx
cp -r rust-doc-tmp/doc/* _site/api/rust/

cd ../rust/maze_openapi_generator
cargo run
redocly build-docs openapi.json -o ../../docfx/web-doc-tmp/doc/maze_rest/index.html --config ../../docfx/redocly.yaml

cd ../../docfx
cp -r web-doc-tmp/doc/* _site/api/web/


