# `parenv`

[![crates.io version](https://img.shields.io/crates/v/parenv.svg)](https://crates.io/crates/parenv) [![crates.io license](https://img.shields.io/crates/l/parenv.svg)](https://crates.io/crates/parenv) [![crates.io downloads](https://img.shields.io/crates/d/parenv.svg)](https://crates.io/crates/parenv) [![docs.rs documentation](https://img.shields.io/docsrs/parenv.svg)](https://docs.rs/parenv)

Environment variable parser with a clap style derive macro and elm style error reporting.

## Installation

```bash
cargo add parenv
```

## Usage

Here are some important features you should know about.

- `parenv` relies on the `FromStr` trait to parse environment variables into the specified type.
- The documentation comment on each field is used as the description for the corresponding environment variable.
- To make a field optional, wrap the type with an `Option`.
- To set a prefix value, set the attribute `#[parenv(prefix = "ENV_")]` on your struct.
- To set a suffix value, set the attribute `#[parenv(suffix = "_ARG")]` on your struct.

```rust
use std::{net::SocketAddr, path::PathBuf};

use parenv::Environment;

#[derive(Debug, Environment)]
#[parenv(prefix = "ENV_", suffix = "_ARG")]
struct Env {
    /// The cat
    cat: Option<u8>,
    /// The dog
    dog: SocketAddr,
    /// The file
    file: PathBuf,
}

fn main() {
    let env = Env::parse();
    dbg!(env.cat, env.dog, env.file);
}
```

## Demo

[![asciicast](https://asciinema.org/a/691817.svg)](https://asciinema.org/a/691817)
