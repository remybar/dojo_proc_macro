//mod introspect;
mod model;

fn main() {
    let hash = bytearray_hash!("hello");
    println!("hash: {}", hash);
}
