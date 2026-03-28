# `Maze.Interop` Assembly

## Introduction

The `Maze.Interop` .NET assembly is written in `C#` and provides interop between .NET applications and the `maze_wasm.wasm` Web Assembly. Its purpose is to allow the WebAssembly's functionality to be used from with .NET applications, without needing to be concerned with the underlying .NET to WebAssembly interop specifics.

It exposes a singleton instance of a `MazeInterop` object, which can be accessed via:

```csharp
var instance = MazeInterop.GetInstance();
```

The `GetInstance()` function enforces the singleton instance and, on initialisation, loads the `maze_wasm` WebAssembly and any required function entry points within it. If any functions are found to be missing, an exception will be thrown. By default, `GetInstance()` will use `Wasmtime` as the interop `ConnectionType`, but the caller can override this if required. On `Android`, the `Wasmer` connection type must be used (it bundles a static version of the Wasmer library). On iOS physical devices, the `Native` connection type must be used — this P/Invokes directly into the `maze_c` native `staticlib` (`libmaze_c.a`) and requires no WebAssembly runtime. The different native runtimes can be found in the `runtimes` sub-directory.

Once the instance is obtained, the caller can execute methods to create and interact with `maze_wasm` objects. It is important that any object pointers that are returned to the caller are released using the appropriate `MazeInterop` method. For example, to create a new maze, resize it to 10 rows by 5 columns and display the number of rows and columns, the following code can be used:

```csharp
var interop = MazeInterop.GetInstance();
UInt32 mazeWasmPtr = interop.NewMazeWasm();
interop.MazeWasmResize(mazeWasmPtr, 10, 5);
var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
Console.WriteLine($"New dimensions = {rowCount} rows x {colCount} columns");
interop.FreeMazeWasm(mazeWasmPtr);
```

Notice that this code uses `FreeMazeWasm()` to release the `MazeWasm` pointer after it is no longer needed. If this is not done, a memory leak will occur within the `maze_wasm` Web Assembly.

## Getting Started

### Setup
To setup the build environment, run the following from the `Maze.Interop` directory:

```
dotnet restore
```

### Build
To build the `Maze.Interop` assembly, run the following from the `Maze.Interop` directory:

```
dotnet build
```

### Testing
Testing can be performed via the [`Maze.Interop.Tests`](../Maze.Interop.Tests/README.md) assembly.

### Wasmer C API Notes

#### 1. Building the C API for Android using `Windows Subsystem for Linux (WSL)`

- Install `WSL` (if not already present)
- Set Up `WSL` Environment (from bash prompt)
- Update package manager and install necessary tools:
    ```
    sudo apt update
    sudo apt upgrade -y
    udo apt install build-essential curl git clang lld
    ```
- Install Rust for Linux:
    ```
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
    rustup update    
    ```
- Add the `x86_64-linux-android` target:
    ```
    rustup target add x86_64-linux-android
    ```
- Download and extract the Android NDK for Linux:
    - From the [Android NDK Download page](https://developer.android.com/ndk/downloads), get the Linux 64-bit (x86) version (as of now this is `android-ndk-r27c-linux.zip`).
    - Extract it to a directory in your WSL environment to the `android-sdk` directory in the root of the home directory (do this from within the WSL bash prompt), e.g.:
    ```
    mkdir -p ~/android-ndk
    unzip android-ndk-27c-linux.zip -d android-ndk
    ```

    Note: iF `unzip` is not installed, install it with:
    ```
    sudo apt update
    sudo apt install unzip
    ```
    - Verify the extraction:
    ```
    ls android-ndk
    ```
    The directory should contain the NDK files, including the `toolchains/llvm/prebuilt/linux-x86_64/bin` folder.
    - Set executable permissions for NDK Binaries:
    ```
    cd android-ndk/android-ndk-r27c/toolchains/llvm/prebuilt/linux-x86_64/bin
    chmod +x *
    ```
    This ensures all files in the `bin` directory are executable.

    - Configure the `Rust` build environment (replacine )
    - Open `.bashrc` for edit:
        ```
        nano ~/.bashrc
        ```
    - Add the following lines:
        ```
        # Android 
        export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER=/home/<USER_NAME>/android-ndk/<NDK_VERSION_FOLDER>/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android21-clang
        export RUSTFLAGS="-C link-args=-L/home/<USER_NAME>/android-ndk/<NDK_VERSION_FOLDER>/platforms/android-21/arch-x86_64/usr/lib"
        export ANDROID_NDK_HOME=/home/<USER_NAME>/android-ndk/<NDK_VERSION_FOLDER>
        export TOOLCHAIN=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64
        export PATH=$TOOLCHAIN/bin:$PATH
        ```
        where:

        - `<USER_NAME>` is your `WSL` username
        - `<NDK_VERSION_FOLDER>` is the folder contain the version-specific `NDK` files e.g. `android-ndk-r27c`

    - Save your changes, exit `nano` and then apply the changes by running:
        ```
        source ~/.bashrc
        ```

   - Test the `Rust` build environment by creating a simple app as follows:
     - Create a new project:
       ```
       cargo new --bin hello
       cd hello
       cargo run
       ```
       This should successfully build for Linux and run, displaying a `Hello, world!` message
     - Now check whether the build succeeds for the `x86_64-linux-android` target, as follows:
       ```
       cargo build --release --target x86_64-linux-android
       ```
     - This should succeed and result in Android files being generated under the `target\x86_64-linux-android\release` sub-directory (e.g. `hello`, `hello.d`)

     - If both builds are successful, this confirms the `Rust` build environment is working for both the `Linux` (`WSL`) and `\x86_64-linux-android` targets and we are ready to focus on building the `wasmer` C API 

    - Download the `wasmer` repo to the root of your home directory:
      ```
      cd ~
      git clone https://github.com/wasmerio/wasmer.git    
      ```
    - Build Wasmer C API for Linux
      - We do this to confirm that the C API builds successfully for the local `Linux` environment:
        ```
        cd ~/wasmer/lib/c-api
        cargo clean
        cargo build --release
        ```
      This should build without error, although you may see some warnings towards the end  (e.g. relating to "warning: unexpected `cfg` condition value: `dylib`") which can be ignored

      Confirm that the directory `~/wasmer/target/release` contains the build outputs including `libwasmer.a`, `libwasmer.d` and `libwasmer.so`. 

  - Build Wasmer C API for Android X86_64

    - To build for the `x86_64-linux-android` target, run the following:
      ```
      cd ~/wasmer/lib/c-api
      cargo clean
      cargo build --release --target x86_64-linux-android
      ```
    - This should result in `libwasmer.a`, `libwasmer.d` and `libwasmer.so` (along with others) being generated in the `~/wasmer/target/x86_64-linux-android/release` directory. The file of interest is `libwasmer.so`, which should be copied into `Maze.Maui.App/runtimes/android-x64/native`.


#### 2. Building the C API for iOS Simulator (on macOS)

- Ensure Xcode is installed for building iOS libraries

- Verify `Xcode` and command-line tools:
    ```
    xcodebuild -version
    ```
    Install the `XCode` command-lien tools if not already installed:
    ```
    xcode-select --install
    ```
- Check Simulator SDK
    ```
    xcrun --sdk iphonesimulator --show-sdk-path
    ```
    This should return the path to the simulator SDK, such as:
    ```
    /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk
    ```
- Ensure Clang is available:
    ```
    xcrun --sdk iphonesimulator -f clang
    ```
    This should return the path to the clang executable, such as:
    ```
    /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang
    ```
- Set Environment Variables:
    ```
    export SDKROOT=$(xcrun --sdk iphonesimulator --show-sdk-path)
    export CC="$(xcrun --sdk iphonesimulator -f clang)"
    export CFLAGS="-arch arm64 -isysroot $SDKROOT"
    export LDFLAGS="-arch arm64 -isysroot $SDKROOT"
    ```
- Download the `wasmer` repo to the root of your home directory:
    ```
    cd ~
    git clone https://github.com/wasmerio/wasmer.git    
    ```
- Install Rust for iOS:
    ```
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

- Add the `aarch64-apple-ios-sim` target:
    ```
    rustup target add aarch64-apple-ios-sim
    ```

- Build Wasmer C API for MacOS:
    - We do this to confirm that the C API builds successfully for the local `MacOS` environment:
    ```
    cd ~/wasmer/lib/c-api
    cargo clean
    cargo build --release
    ```
    This should build without error, although you may see some warnings towards the end  (e.g. relating to "warning: unexpected `cfg` condition value: `dylib`") which can be ignored

    Confirm that the directory `~/wasmer/target/release` contains the build outputs including `libwasmer.a`, `libwasmer.d` and `libwasmer.dylib`. 

- Build Wasmer C API for iOS Simulator using the  `aarch64-apple-ios-sim` target
    ```
    cd ~/wasmer/lib/c-api
    cargo clean
    cargo build --release --target aarch64-apple-ios-sim --features c-api
    ```
    This should result in `libwasmer.a`, `libwasmer.d` and `libwasmer.dylib` (along with others) being generated in the `~/wasmer/target/aarch64-apple-ios-sim/release` directory. The file of interest is the static library file `libwasmer.a`, which should be copied into `Maze.Maui.App/runtimes/ios-sim-arm64/native`.


#### 3. Building the Native Library for iOS Physical Devices (`libmaze_c.a`)

iOS physical devices enforce W^X (Write XOR Execute) — Apple's code-signing policy prohibits JIT-style executable page creation at runtime. Wasmer's Cranelift backend requires this and will crash on device with `EXC_BAD_ACCESS / CODESIGNING 2`. The `maze_c` native Rust staticlib is used instead, compiled directly into the app via `DllImport("__Internal")`.

Building requires a Mac with Xcode and a Rust toolchain.

- Add the iOS device target:
    ```
    rustup target add aarch64-apple-ios
    ```

- Cross-compile `maze_c` for iOS device:
    ```
    cd src/rust
    cargo build --release --target aarch64-apple-ios -p maze_c
    ```

- Copy the resulting `libmaze_c.a` into the interop runtimes directory:
    ```
    cp target/aarch64-apple-ios/release/libmaze_c.a \
       ../csharp/Maze.Interop/runtimes/ios-arm64/native/libmaze_c.a
    ```

- Verify it targets the correct platform (should show `platform 2`):
    ```
    otool -l runtimes/ios-arm64/native/libmaze_c.a | grep platform
    ```

The `MazeNativeConnector` C# class (`MazeNativeConnector.cs`) calls these symbols via `[DllImport("__Internal")]` when `ConnectionType.Native` is selected. See [`maze_c`](../../../rust/maze_c/README.md) for the full C API reference.
