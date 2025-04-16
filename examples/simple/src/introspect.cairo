#[derive(Drop, Serde, Introspect)]
struct S1 {
    x: u8,
    y: u32,
}

#[derive(Drop, Serde, IntrospectPacked)]
struct S2 {
    x: u8,
    y: u32,
}

#[derive(Drop, Serde, Introspect)]
struct S3<T> {
    x: u8,
    y: T,
}

#[derive(Drop, Serde, Introspect)]
enum E1 {
    A: u32,
    B: Option<u8>,
    C: (u8, u16, u32),
    D: Array<u8>,
    E: S1,
}

#[derive(Drop, Serde, IntrospectPacked)]
enum E2 {
    A: u64,
    B: u64,
}

#[derive(Drop, Serde, Introspect)]
enum E3<T> {
    A: u64,
    B: T,
}
