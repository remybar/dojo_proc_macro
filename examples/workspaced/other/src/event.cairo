#[derive(Introspect)]
#[dojo::event]
pub struct E1 {
    #[key]
    pub k1: u8,
    pub v1: u8,
    pub v2: u32
}

#[dojo::event]
pub struct E2 {
    #[key]
    pub k1: u8,
    pub v1: u8,
    pub v2: u32
}
