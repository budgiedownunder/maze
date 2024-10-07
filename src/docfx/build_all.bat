rmdir /s /q _site
rmdir /s /q obj
rmdir /s /q ./rust-doc-tmp

call copy_files.bat
docfx docfx.json
cd ../rust
cargo doc --no-deps --target-dir ../docfx/rust-doc-tmp
cd ../docfx
xcopy "rust-doc-tmp\doc\*" "_site\api\rust" /s /e /y

