#!/usr/bin/env just --justfile

# A welcomer
default:
    just --list

# Make sure environment has all the necessary tools (building only)
check-build:
    which cargo # Rust compiler
    which npm # For building web
    which node # For building web
    which wasm-pack # For building web
    which wasm-bindgen # For building web
    which javac || echo -e "\e[0;33m'javac' is not available!\e[0m" # For building for Android
    [[ -v ANDROID_HOME ]] || echo -e "\e[0;33mAndroid SDK is not available!\e[0m" # For building for Android
    [[ -v ANDROID_NDK_ROOT ]] || echo -e "\e[0;33mAndroid NDK is not available!\e[0m" # For building for Android

# Make sure environment has all the necessary tools
check: check-build
    which mprocs || echo -e "\e[0;33m'mprocs' is not installed!\e[0m" # For running multiple execuables at the same time
    which voidsprite # For creating and modifying textures

# Run desktop build
desktop:
    just desktop/run

# Run web build + server
web:
    just web/build
    just server/run

# Run both desktop and server
desktop-full:
    mprocs "just desktop" "just web"

# Remove build artifacts
clean:
    rm -fr build
    cargo clean
    rm -fr web/{dist,pkg,node_modules}

