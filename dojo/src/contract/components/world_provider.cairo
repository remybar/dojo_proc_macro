#[derive(Serde, Drop)]
struct IWorldDispatcher{}

#[starknet::interface]
pub trait IWorldProvider<T> {
    fn world_dispatcher(self: @T) -> IWorldDispatcher;
}

#[starknet::component]
pub mod world_provider_cpt {
    use super::IWorldDispatcher;

    #[storage]
    pub struct Storage {}

    #[embeddable_as(WorldProviderImpl)]
    pub impl WorldProvider<
        TContractState, +HasComponent<TContractState>,
    > of super::IWorldProvider<ComponentState<TContractState>> {
        fn world_dispatcher(self: @ComponentState<TContractState>) -> IWorldDispatcher {
            IWorldDispatcher{}
        }
    }

    #[generate_trait]
    pub impl InternalImpl<
        TContractState, +HasComponent<TContractState>,
    > of InternalTrait<TContractState> {
        fn initializer(ref self: ComponentState<TContractState>) {
        }
    }
}
