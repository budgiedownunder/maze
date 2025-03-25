rmdir /s /q _site
rmdir /s /q obj
rmdir /s /q rust-doc-tmp
rmdir /s /q js-doc-tmp
rmdir /s /q web-doc-tmp

call copy_files.bat
docfx metadata docfx.json

@REM type api\net\toc.yml

powershell -ExecutionPolicy Bypass -File cleanup_tocs.ps1

docfx build docfx.json

cd ../rust
cargo doc --locked --no-deps --target-dir ../docfx/rust-doc-tmp

cd ../docfx
xcopy "rust-doc-tmp\doc\*" "_site\api\rust" /s /e /y

cd ../rust/maze_wasm
cargo doc --locked --no-deps --features "wasm-bindgen" --target-dir ../../docfx/js-doc-tmp

cd ../../docfx
xcopy "js-doc-tmp\doc\*" "_site\api\js" /s /e /y

cd ../rust/maze_openapi_generator
cargo run --locked

call redocly build-docs openapi.json -o ../../docfx/web-doc-tmp/doc/maze_rest/index.html --config ../../docfx/redocly.yaml

cd ../../docfx
xcopy "web-doc-tmp\doc\*" "_site\api\web" /s /e /y


