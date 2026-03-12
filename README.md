# figlet-rs

[![CI](https://github.com/yuanbohan/rs-figlet/actions/workflows/ci.yml/badge.svg)](https://github.com/yuanbohan/rs-figlet/actions/workflows/ci.yml)
[![docs](https://docs.rs/figlet-rs/badge.svg)](https://docs.rs/figlet-rs)
[![crates.io](https://img.shields.io/crates/v/figlet-rs.svg)](https://crates.io/crates/figlet-rs)

A Rust library for [FIGlet](http://www.figlet.org/) to generate ascii art.

The default rendering behavior follows the font's built-in FIGlet layout settings, including
horizontal kerning and smushing. The current output is tested against golden fixtures generated
from `figlet 2.2.5`, but running tests does not require `figlet` to be installed.

## Example

```rust
use figlet_rs::FIGfont;

fn main() {
    let standard_font = FIGfont::standard().unwrap();
    let small_font = FIGfont::small().unwrap();
    let big_font = FIGfont::big().unwrap();
    let slant_font = FIGfont::slant().unwrap();

    println!("{}", standard_font.convert("Hello Rust").unwrap());
    println!("{}", small_font.convert("Test").unwrap());
    println!("{}", big_font.convert("Test").unwrap());
    println!("{}", slant_font.convert("Test").unwrap());
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
```

## Load A Font File

```rust
use figlet_rs::FIGfont;

fn main() {
    let font = FIGfont::from_file("resources/small.flf").unwrap();
    println!("{}", font.convert("Test").unwrap());
}
```

The default spacing behavior matches:

```sh
figlet -f resources/small.flf Test
```

## Bundled Fonts

The crate bundles these fonts as built-in APIs, so downstream users do not need to manage font
files manually:

- `FIGfont::standard()` loads `resources/standard.flf`
- `FIGfont::small()` loads `resources/small.flf`
- `FIGfont::big()` loads `resources/big.flf`
- `FIGfont::slant()` loads `resources/slant.flf`

If you want to load a custom `.flf` file from disk, use `FIGfont::from_file(...)`.

## Testing

Golden fixtures live in [`tests/fixtures`](./tests/fixtures). They are committed to the repository so
`cargo test` stays stable in environments without `figlet`.

If you want to refresh those fixtures on a machine that already has `figlet`, run:

```sh
./scripts/generate_golden_fixtures.sh
```

## License

rs-figlet is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [COPYRIGHT](COPYRIGHT) for details.
