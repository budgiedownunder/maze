rmdir /s /q _site
rmdir /s /q obj
rmdir /s /q rust-doc-tmp
rmdir /s /q js-doc-tmp

call copy_files.bat
docfx docfx.json

cd ../rust
cargo doc --no-deps --target-dir ../docfx/rust-doc-tmp

cd ../docfx
xcopy "rust-doc-tmp\doc\*" "_site\api\rust" /s /e /y

cd ../rust/maze_wasm
cargo doc --no-deps --features "wasm-bindgen" --target-dir ../../docfx/js-doc-tmp

cd ../../docfx
xcopy "js-doc-tmp\doc\*" "_site\api\js" /s /e /y


