# rlwinmdec [![Latest Version]][crates.io] [![Api Rustdoc]][rustdoc] ![Rust Version]

[Latest Version]: https://img.shields.io/crates/v/rlwinmdec.svg
[crates.io]: https://crates.io/crates/rlwinmdec
[Api Rustdoc]: https://img.shields.io/badge/api-rustdoc-blue.svg
[rustdoc]: https://docs.rs/rlwinmdec
[Rust Version]: https://img.shields.io/badge/rust-1.58+-blue.svg?maxAge=3600


PowerPC Rlwinm/Rlwimi/Rlwmn Decoder Tool

## Usage

```rs
use rlwinmdec::*;

fn main(){
  if let Some(result) = decode("rlwinm r3, r4, 3, 4, 5") {
	println!(result);
  }
}
```
