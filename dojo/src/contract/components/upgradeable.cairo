use starknet::ClassHash;

#[starknet::interface]
pub trait IUpgradeable<T> {
    fn upgrade(ref self: T, new_class_hash: ClassHash);
}

#[starknet::component]
pub mod upgradeable_cpt {
    use starknet::ClassHash;
    use dojo::contract::components::world_provider::IWorldProvider;

    #[storage]
    pub struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        Upgraded: Upgraded,
    }

    #[derive(Drop, starknet::Event)]
    pub struct Upgraded {
        pub class_hash: ClassHash,
    }

    #[embeddable_as(UpgradeableImpl)]
    impl Upgradeable<
        TContractState, +HasComponent<TContractState>, +IWorldProvider<TContractState>,
    > of super::IUpgradeable<ComponentState<TContractState>> {
        fn upgrade(ref self: ComponentState<TContractState>, new_class_hash: ClassHash) {
        }
    }
}
