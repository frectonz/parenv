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
