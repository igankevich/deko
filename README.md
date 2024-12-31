# deko

[![Crates.io Version](https://img.shields.io/crates/v/deko)](https://crates.io/crates/deko)
[![Docs](https://docs.rs/deko/badge.svg)](https://docs.rs/deko)
[![dependency status](https://deps.rs/repo/github/igankevich/deko/status.svg)](https://deps.rs/repo/github/igankevich/deko)

![Deko icon.](https://raw.githubusercontent.com/igankevich/rust-docs-assets/master/deko/deko.png)

A decoder that automatically detects compression format (gzip, bzip2, xz, zstd) via external crates.
Includes an encoder for the same formats as well.


## Introduction

`deko` is a library that offers `AnyDecoder` and `AnyEcnoder` structs
that can decompress/compress the data from/to a variaty formats via the corresponding crates.
The format is automatically detected via _magic bytes_ — signatures at the start of the file.

Currently the following formats are supported:
- gzip, zlib via [flate2](https://docs.rs/flate2/latest/flate2/);
- bzip via [bzip2](https://docs.rs/bzip2/latest/bzip2/);
- xz via [xz](https://docs.rs/xz/latest/xz/);
- zstd via [zstd](https://docs.rs/zstd/latest/zstd/);

Unused formats can be disabled via crate's features.
By default all formats are enabled.


## Examples

```rust
use deko::Format;
use deko::bufread::AnyDecoder;
use deko::write::{AnyEncoder, Compression};
use std::io::Read;
use std::io::Write;

let mut writer = AnyEncoder::new(Vec::new(), Format::Gz, Compression::Best).unwrap();
writer.write_all(b"Hello world").unwrap();
let compressed_data = writer.finish().unwrap();
let mut reader = AnyDecoder::new(&compressed_data[..]);
let mut string = String::new();
reader.read_to_string(&mut string);
assert_eq!("Hello world", string);
```
