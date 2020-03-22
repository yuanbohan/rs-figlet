# figlet-rs

[![CI](https://travis-ci.com/yuanbohan/rs-figlet.svg?branch=master)](https://travis-ci.com/yuanbohan/rs-figlet)
[![docs](https://docs.rs/figlet-rs/badge.svg)](https://docs.rs/figlet-rs)
[![crates.io](https://img.shields.io/crates/v/figlet-rs.svg)](https://crates.io/crates/figlet-rs)

A Rust library for [FIGlet](http://www.figlet.org/) to generate ascii art.

# Example

```rust
use figlet_rs::FIGfont;

fn main() {
    let standard_font = FIGfont::standand().unwrap();
    let figure = standard_font.convert("FIGlet");
    assert!(figure.is_some());

    let small_font = FIGfont::from_file("resources/small.flf").unwrap();
    let figure = small_font.convert("FIGlet");
    assert!(figure.is_some());
}
```

# License

rs-figlet is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [COPYRIGHT](COPYRIGHT) for details.
