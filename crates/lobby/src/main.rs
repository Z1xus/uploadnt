#[cfg(not(target_arch = "wasm32"))]
mod server;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    server::main()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
