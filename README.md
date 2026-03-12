# figlet-rs

[![CI](https://github.com/yuanbohan/rs-figlet/actions/workflows/ci.yml/badge.svg)](https://github.com/yuanbohan/rs-figlet/actions/workflows/ci.yml)
[![docs](https://docs.rs/figlet-rs/badge.svg)](https://docs.rs/figlet-rs)
[![crates.io](https://img.shields.io/crates/v/figlet-rs.svg)](https://crates.io/crates/figlet-rs)

A Rust library for [FIGlet](http://www.figlet.org/) and Toilet fonts to generate ascii art.

The default rendering behavior follows the font's built-in FIGlet layout settings, including
horizontal kerning and smushing. The current output is tested against golden fixtures generated
from local `figlet` and `toilet` binaries, but running tests does not require either tool to be
installed.

## Example

```rust
use figlet_rs::{FIGfont, Toilet};

fn main() {
    let standard_font = FIGfont::standard().unwrap();
    let small_font = FIGfont::small().unwrap();
    let big_font = FIGfont::big().unwrap();
    let slant_font = FIGfont::slant().unwrap();
    let smblock_font = Toilet::smblock().unwrap();
    let mono12_font = Toilet::mono12().unwrap();
    let future_font = Toilet::future().unwrap();
    let wideterm_font = Toilet::wideterm().unwrap();
    let mono9_font = Toilet::mono9().unwrap();

    println!("{}", standard_font.convert("Hello Rust").unwrap());
    println!("{}", small_font.convert("Test").unwrap());
    println!("{}", big_font.convert("Test").unwrap());
    println!("{}", slant_font.convert("Test").unwrap());
    println!("{}", smblock_font.convert("Toilet").unwrap());
    println!("{}", mono12_font.convert("Test").unwrap());
    println!("{}", future_font.convert("Test").unwrap());
    println!("{}", wideterm_font.convert("Test").unwrap());
    println!("{}", mono9_font.convert("Test").unwrap());
}
```

Output:

```text
 _   _      _ _         ____            _
| | | | ___| | | ___   |  _ \ _   _ ___| |_
| |_| |/ _ \ | |/ _ \  | |_) | | | / __| __|
|  _  |  __/ | | (_) | |  _ <| |_| \__ \ |_
|_| |_|\___|_|_|\___/  |_| \_\\__,_|___/\__|


 _____       _
|_   _|__ __| |_
  | |/ -_|_-<  _|
  |_|\___/__/\__|


 _______        _
|__   __|      | |
   | | ___  ___| |_
   | |/ _ \/ __| __|
   | |  __/\__ \ |_
   |_|\___||___/\__|



  ______          __
 /_  __/__  _____/ /_
  / / / _ \/ ___/ __/
 / / /  __(__  ) /_
/_/  \___/____/\__/


▀▛▘  ▗▜    ▐
 ▌▞▀▖▄▐ ▞▀▖▜▀
 ▌▌ ▌▐▐ ▛▀ ▐ ▖
 ▘▝▀ ▀▘▘▝▀▘ ▀


 ▄▄▄▄▄▄▄▄
 ▀▀▀██▀▀▀                        ██
    ██      ▄████▄   ▄▄█████▄  ███████
    ██     ██▄▄▄▄██  ██▄▄▄▄ ▀    ██
    ██     ██▀▀▀▀▀▀   ▀▀▀▀██▄    ██
    ██     ▀██▄▄▄▄█  █▄▄▄▄▄██    ██▄▄▄
    ▀▀       ▀▀▀▀▀    ▀▀▀▀▀▀      ▀▀▀▀



╺┳╸┏━╸┏━┓╺┳╸
 ┃ ┣╸ ┗━┓ ┃
 ╹ ┗━╸┗━┛ ╹

Ｔｅｓｔ


▄▄▄▄▄▄▄                 ▄
   █     ▄▄▄    ▄▄▄   ▄▄█▄▄
   █    █▀  █  █   ▀    █
   █    █▀▀▀▀   ▀▀▀▄    █
   █    ▀█▄▄▀  ▀▄▄▄▀    ▀▄▄


```

## Load A Font File

```rust
use figlet_rs::{FIGfont, Toilet};

fn main() {
    let figlet_font = FIGfont::from_file("resources/small.flf").unwrap();
    let toilet_font = Toilet::from_file("resources/smblock.tlf").unwrap();

    println!("{}", figlet_font.convert("Test").unwrap());
    println!("{}", toilet_font.convert("Toilet").unwrap());
}
```

The FIGlet output matches:

```sh
figlet -f resources/small.flf Test
```

The Toilet output matches:

```sh
toilet -d resources -f smblock.tlf Toilet
```

## Built-in Fonts

The crate bundles these fonts as built-in APIs:

FIGlet:

- `FIGfont::standard()` loads `resources/standard.flf`
- `FIGfont::small()` loads `resources/small.flf`
- `FIGfont::big()` loads `resources/big.flf`
- `FIGfont::slant()` loads `resources/slant.flf`

Toilet:

- `Toilet::smblock()` loads `resources/smblock.tlf`
- `Toilet::mono12()` loads `resources/mono12.tlf`
- `Toilet::future()` loads `resources/future.tlf`
- `Toilet::wideterm()` loads `resources/wideterm.tlf`
- `Toilet::mono9()` loads `resources/mono9.tlf`

Use `FIGfont::from_file(...)` to load custom `.flf` files.

Use `Toilet::from_file(...)` to load custom `.tlf` files, including zip-packaged `.tlf` files.

## Testing

Golden fixtures live in [`tests/fixtures`](./tests/fixtures). They are committed to the repository so
`cargo test` stays stable in environments without local `figlet` or `toilet` binaries.

If you want to refresh the FIGlet fixtures on a machine that already has `figlet`, run:

```sh
./scripts/generate_golden_fixtures.sh
```

If you want to refresh the Toilet fixtures on a machine that already has `toilet`, run:

```sh
./scripts/generate_toilet_fixtures.sh
```

## License

rs-figlet is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [COPYRIGHT](COPYRIGHT) for details.
