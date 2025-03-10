/// Just to simulate dojo-core
pub mod dojo {
    pub mod meta {
        pub mod introspect;
        use introspect::*;

        pub mod layout;
        use layout::*;
    }
    pub mod utils;
}

#[derive(Introspect)]
struct S1 {
    x: u8,
    y: u32,
}

#[derive(IntrospectPacked)]
struct S2 {
    x: u8,
    y: u32,
}

#[derive(Introspect)]
struct S3<T> {
    x: u8,
    y: T,
}

#[derive(Introspect)]
enum E1 {
    A: u32,
    B: Option<u8>,
    C: (u8, u16, u32),
    D: Array<u8>,
    E: S1,
}

#[derive(IntrospectPacked)]
enum E2 {
    A: u64,
    B: u64,
}

#[derive(Introspect)]
enum E3<T> {
    A: u64,
    B: T,
}


fn main() {
    let hash = bytearray_hash!("hello");
    println!("hash: {}", hash);
}
