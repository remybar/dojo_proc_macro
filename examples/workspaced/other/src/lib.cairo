mod introspect;
mod model;
mod event;
mod contract;
mod library;

fn main() {
    let hash = bytearray_hash!("hello");
    println!("hash: {}", hash);
}
