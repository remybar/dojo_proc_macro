//mod introspect;
//mod model;
mod event;

fn main() {
    let hash = bytearray_hash!("hello");
    println!("hash: {}", hash);
}
