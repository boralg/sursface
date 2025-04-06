# Sursface

Sursface is a small cross-platform rendering library designed to simplify the development of WebGPU applications, without compromising developer control.

It builds upon well-established APIs (`wgpu` and `winit`) to wrap over the core lifecycle operations of a graphical app, while re-exposing the full capabilities of these APIs unchanged. Sursface's goal is to enable writing plain WebGPU code with less hassle.

## Supported Platforms
- **Cross-Platform Desktop**: Compile and run on Linux or Windows.
- **WASM**: Automatically generate JavaScript bindings for the `wasm32-unknown` Nix target, making your apps trivially embeddable into webpages.

## Features
- **Ready-made WebGPU Surface**: Sursface provides the entrypoint to your graphical app by supplying a platform window configured to be used as a surface by `wgpu`.
- **Event Loop**: Sursface abstracts away the specifics of the event loop, whether it's running on desktop or in the browser. You simply need to provide the function pointer to your frame-by-frame logic, and Sursface will regularly call it for you.
- **No New Tech**: Once the setup is done, keep using `wgpu` as if `sursface` wasn't there.
- **Default Logic (WIP)**: Sursface will offer opt-in shortcuts for some elementary operations through simple functions, further reducing graphical boilerplate.

## Building Sursface

### Prerequisites

To compile Sursface, ensure you have the following tools installed: Rust, Cargo, and optionally [Nix](https://nixos.org/download.html) (recommended).

### Installation

Clone the repository:

```sh
git clone https://github.com/boralg/sursface.git
cd sursface
```

### Building with Cargo

You can compile Sursface on your native platform using Cargo:

```sh
cargo build --release
```

### Building with Nix

To build a binary inside the Nix store, ready to run locally:

```sh
nix build .
```

### Cross-Compiling with Nix

Sursface supports cross-compilation to various targets. Here are some examples:

```sh
nix build .#x86_64-linux
nix build .#x86_64-windows
nix build .#wasm32-unknown
```

## Releases

Releases consist of executable binaries of the examples. These are automatically built for each new release, and can be directly downloaded, unzipped, and ran, without any installation process.

## Running Examples

Examples can also be run directly with `cargo`:

```sh
cargo run --bin <example_name>
```

For instance:

```sh
cargo run --bin hello_triangle
```