# White Balance

A lightweight PhotoCraft ABI v1 filter for correcting warm/cool color casts and green/magenta tint.

## Controls

- **Temperature:** -100 to +100. Positive values warm the image (more red, less blue); negative values cool it (less red, more blue).
- **Tint:** -100 to +100. Positive values move toward magenta; negative values move toward green.
- Both controls default to zero, which leaves pixels unchanged.

## Processing

The filter applies simple channel gains without allocating additional buffers. It supports RGB and Indexed-as-RGB inputs. Grayscale and Duotone-as-grayscale are accepted but unchanged because they contain no independent RGB channels. Alpha is preserved, fully transparent pixels are skipped, and gain values are bounded away from zero.

## Build and test

```sh
rustup target add wasm32-unknown-unknown
cargo test --manifest-path plugins/white-balance/Cargo.toml
cargo build --manifest-path plugins/white-balance/Cargo.toml --release --target wasm32-unknown-unknown
```

A successful build does not prove host integration. Install and run the filter inside PhotoCraft before marking it ready for users.
