#[dojo::model]
pub struct M1 {
    #[key]
    pub k1: u8,
    pub v1: u8,
    pub v2: u32
}

#[derive(IntrospectPacked)]
#[dojo::model]
pub struct M2 {
    #[key]
    pub k1: u8,
    pub v1: u8,
    pub v2: u32
}
