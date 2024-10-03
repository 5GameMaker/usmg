# A magic game name for which I haven't thought of yet

## Building

*Building is completely untested on Windows and all release binaries are cross-compiled!*

*If any of the following dependencies are out of date, update them. Building with outdated packages is not supported!*
*(Looking at you, Debian)*

To build this project you need:
- Bash or bash-compatible shell (usually preinstalled)
- Rust (`$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- - Native and `wasm32-unknown-unknown` toolchains (`$ cargo target add <toolchain>`)
- NPM & NodeJS (https://nodejs.org/)
- SDL2 native libraries
- Just (`$ cargo install just`)
- wasm-pack (`$ cargo install wasm-pack`)
- wasm-bindgen (`$ cargo install wasm-bindgen-cli`)
- Java (preferrably 17 or later)
- Android SDK + NDK (API level 34 or later) (Set as `ANDROID_HOME` and `ANDROID_NDK_ROOT` respectively)

Additionally, all the following is needed for contributing to the project:
- Voidsprite (`$ git clone https://aur.archlinux.org/voidsprite-git.git && cd voidsprite-git && makepkg -si` or https://github.com/counter185/voidsprite)

Run `just check` to check if everything's installed (there is no version check yet, feel free to RP).

For additional documentation on available tasks run `just`.

You can run all the commands manually (if you're brave enough), but I would personally recommend to just use `just`.

## Project structure

- `/app`
> The actual game.

- `/assets`
> The game assets.
>
> Contributing to this, please *commit both .png and .voidsn files*. .png files are a subject for removal
> if voidsprite introduces a cli interface.

- `/foss-licenses`
> Some of the open-source licenses of the projects this project depends on.

- `/desktop`
> A desktop SDL2-based interface.

- `/server`
> A game client host + game server.

- `/web`
> A web interface.

## Licensing

This project is licensed under AGPL 3.0 (or later).
