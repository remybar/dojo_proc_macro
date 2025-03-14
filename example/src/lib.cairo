mod introspect;
mod model;
mod event;
mod contract;

fn main() {
    let hash = bytearray_hash!("hello");
    println!("hash: {}", hash);
}
