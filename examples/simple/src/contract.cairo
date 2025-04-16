#[starknet::interface]
pub trait IActions<T> {
    fn spawn(ref self: T);
    #[cfg(feature: 'dungeon')]
    fn enter_dungeon(ref self: T, dungeon_address: starknet::ContractAddress);
}

#[dojo::contract]
pub mod actions {
    use super::IActions;

    #[cfg(feature: 'dungeon')]
    use super::model::M1;

    // impl: implement functions specified in trait
    #[abi(embed_v0)]
    impl ActionsImpl of IActions<ContractState> {
        // ContractState is defined by system decorator expansion
        fn spawn(ref self: ContractState) {}

        #[cfg(feature: 'dungeon')]
        fn enter_dungeon(ref self: ContractState, dungeon_address: ContractAddress) {}
    }
}
