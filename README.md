# `parenv`

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

```rust
use std::{net::SocketAddr, path::PathBuf};

use parenv::Environment;

#[derive(Debug, Environment)]
#[parenv(prefix = "ENV_")]
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

[![asciicast](https://asciinema.org/a/689791.svg)](https://asciinema.org/a/689791)
