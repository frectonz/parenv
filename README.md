# `parenv`

Environment variable parser with a clap style derive macro and elm style error reporting.

## Installation

```bash
cargo add parenv
```

## Usage

The following demonstrates a simple usage example. `parenv` relies on the `FromStr` trait to parse environment variables into the specified type.
The documentation comment on each field is used as the description for the corresponding environment variable.

```rust
use std::{net::SocketAddr, path::PathBuf};

use parenv::Environment;

#[derive(Debug, Environment)]
struct Env {
    /// The cat
    cat: u8,
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

[![asciicast](https://asciinema.org/a/689713.svg)](https://asciinema.org/a/689713)
