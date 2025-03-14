//! ResourceMetadata model.
//!
#[derive(Drop, Serde, PartialEq, Clone, Debug, Default)]
pub struct ResourceMetadata {
    #[key]
    pub resource_id: felt252,
    pub metadata_uri: ByteArray,
    pub metadata_hash: felt252,
}

pub fn default_address() -> starknet::ContractAddress {
    starknet::contract_address_const::<0>()
}

pub fn default_class_hash() -> starknet::ClassHash {
    starknet::class_hash::class_hash_const::<0>()
}
