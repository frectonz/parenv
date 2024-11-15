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
