# C# Components 

## Introduction
The following `C#` (`.NET`) components are present:

| Folder      | Component                                                   | Description 
|--------------|------------------------------------------------------------|---------------
| `src/csharp` | [`Maze.Api`](./Maze.Api/README.md)                         | .NET API that sits above  [`Maze.Wasm.Interop`](./Maze.Wasm.Interop/README.md)
|              | [`Maze.Api.Tests`](./Maze.Api.Tests/README.md)             | Unit tests for [`Maze.Api`](./Maze.Api/README.md)
|              | [`Maze.Maui.App`](./Maze.Maui.App/README.md)               | Maze [MAUI](https://dotnet.microsoft.com/md) application
|              | [`Maze.Maui.Controls`](./Maze.Maui.Controls/README.md)     | Maze [MAUI](https://dotnet.microsoft.com/en-usen-us/apps/maui) custom controls              
|              | [`Maze.Maui.Services`](./Maze.Maui.Services/README.md)     | Maze [MAUI](https://dotnet.microsoft.com/en-usen-us/apps/maui) custom services              
|              | [`Maze.Wasm.Interop`](./Maze.Wasm.Interop/README.md)       | .NET interop to `maze_wasm` web assembly
|              | [`Maze.Wasm.Interop.Tests`](./Maze.Wasm.Interop/README.md) | .NET test library for [`Maze.Wasm.Interop`](./Maze.Wasm.Interop/README.md) 

## Getting Started

### Setup
To setup the build and test environment for `.NET`, you first need to install:

- [`.NET 8.0+`](https://dotnet.microsoft.com/en-us/download)

and then run the following from the `csharp` directory:

```
dotnet restore
```

### Build
To build all components, run the following from the `csharp` directory:

Windows:

```
dotnet build Build-Windows.sln
```

Non-Windows:

```
dotnet build Build-Non-Windows.sln
```

### Testing
To test all components, run the following from the `csharp` directory:

Windows:

```
dotnet test Build-Windows.sln
```

Non-Windows:

```
dotnet test Build-Non-Windows.sln
```

### Benchmarking
There are no benchmarking tests currently configured for these components.

### Generating Documentation

#### `.NET`-only 
To generate and view documentation for the project's `.NET` (`C#`) assemblies, with placeholder pages for where the `Rust` documentation would be, run the following from the `./src/docfx` directory:

Windows:

```
build_all.bat
serve.bat
```

Linux/macOS:

```
sh build_all.sh
sh serve.sh
```

#### Combined (`.NET` and `Rust`)
To generate and view combined documentation for the project's `.NET` (`C#`) assemblies and `Rust` crates, refer to the [README](../docfx/README.md) in the `docfx` project.


## Building the Wasmer C API for Android and iOS


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

- Download `wasmer` repo
  - Donwload the `wasmer` repo to the root of your home directory:
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

 
