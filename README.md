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
    let figure = standard_font.convert("Hello Rust");
    assert!(figure.is_some());
    println!("{}", figure.unwrap());
}
```

![figlet-sample](./figlet-sample.png)

# License

rs-figlet is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [COPYRIGHT](COPYRIGHT) for details.
