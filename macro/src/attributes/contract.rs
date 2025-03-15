use cairo_lang_macro::{
    quote, Diagnostic, Diagnostics, ProcMacroResult, TextSpan, Token, TokenStream, TokenTree,
};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::Terminal;
use cairo_lang_syntax::node::{
    ast::{self, MaybeModuleBody, OptionReturnTypeClause},
    kind::SyntaxKind::ItemModule,
    TypedSyntaxNode,
};

use crate::utils::{tokenize, DiagnosticsExt, ProcMacroResultExt};
use dojo_types::naming;
use crate::constants::{CONSTRUCTOR_FN, DOJO_INIT_FN};

#[derive(Debug)]
pub struct DojoContract {
    diagnostics: Vec<Diagnostic>,
}

impl DojoContract {
    pub fn process(token_stream: TokenStream) -> ProcMacroResult {
        let db = SimpleParserDatabase::default();
        let (root_node, _diagnostics) = db.parse_token_stream(&token_stream);

        for n in root_node.descendants(&db) {
            if n.kind(&db) == ItemModule {
                let module_ast = ast::ItemModule::from_syntax_node(&db, n);
                return DojoContract::process_ast(&db, &module_ast);
            }
        }

        ProcMacroResult::empty()
    }

    fn process_ast(db: &SimpleParserDatabase, module_ast: &ast::ItemModule) -> ProcMacroResult {
        let mut contract = DojoContract {
            diagnostics: vec![],
        };

        let name = module_ast.name(db).text(db).to_string();

        if !naming::is_name_valid(&name) {
            return ProcMacroResult::fail(format!(
                "The contract name '{name}' can only contain characters (a-z/A-Z), \
                digits (0-9) and underscore (_)."
            ));
        }

        let mut has_event = false;
        let mut has_storage = false;
        let mut has_init = false;
        let mut has_constructor = false;

        if let MaybeModuleBody::Some(body) = module_ast.body(db) {
            let mut body_nodes = body
                .items(db)
                .elements(db)
                .iter()
                .map(|el| {
                    if let ast::ModuleItem::Enum(ref enum_ast) = el {
                        if enum_ast.name(db).text(db).to_string() == "Event" {
                            has_event = true;
                            return contract.merge_event(db, &enum_ast);
                        }
                    } else if let ast::ModuleItem::Struct(ref struct_ast) = el {
                        if struct_ast.name(db).text(db).to_string() == "Storage" {
                            has_storage = true;
                            return contract.merge_storage(db, &struct_ast);
                        }
                    } else if let ast::ModuleItem::FreeFunction(ref fn_ast) = el {
                        let fn_decl = fn_ast.declaration(db);
                        let fn_name = fn_decl.name(db).text(db);

                        if fn_name == CONSTRUCTOR_FN {
                            has_constructor = true;
                            return contract.handle_constructor_fn(db, fn_ast);
                        }

                        if fn_name == DOJO_INIT_FN {
                            has_init = true;
                            return contract.handle_init_fn(db, fn_ast);
                        }
                    }

                    let el = el.as_syntax_node();
                    let el = SyntaxNodeWithDb::new(&el, db);
                    quote! { #el }
                })
                .collect::<Vec<TokenStream>>();

            // TODO RBA: export ctor body / init params+body => add them once here
            if !has_constructor {
                let node = quote! {
                    #[constructor]
                    fn constructor(ref self: ContractState) {
                        self.world_provider.initializer();
                    }
                };

                body_nodes.push(node);
            }

            if !has_init {
                let init_name = TokenTree::Ident(Token::new(DOJO_INIT_FN, TextSpan::call_site()));
                body_nodes.push(quote!{
                    #[abi(per_item)]
                    #[generate_trait]
                    pub impl IDojoInitImpl of IDojoInit {
                        #[external(v0)]
                        fn #init_name(self: @ContractState) {
                            if starknet::get_caller_address() != self.world_provider.world_dispatcher().contract_address {
                                core::panics::panic_with_byte_array(
                                    @format!("Only the world can init contract `{}`, but caller is `{:?}`",
                                    self.dojo_name(),
                                    starknet::get_caller_address(),
                                ));
                            }
                        }
                    }
                });
            }

            if !has_event {
                body_nodes.push(contract.create_event());
            }

            if !has_storage {
                body_nodes.push(contract.create_storage());
            }

            let contract_code = DojoContract::generate_contract_code(&name, body_nodes);
            return ProcMacroResult::new(contract_code)
                .with_diagnostics(Diagnostics::new(contract.diagnostics));
        }

        ProcMacroResult::fail(format!("The contract '{name}' is empty."))
    }

    fn generate_contract_code(name: &String, body: Vec<TokenStream>) -> TokenStream {
        let content = format!(
            "
    use dojo::contract::components::world_provider::{{world_provider_cpt, world_provider_cpt::InternalTrait as WorldProviderInternal, IWorldProvider}};
    use dojo::contract::components::upgradeable::upgradeable_cpt;
    use dojo::contract::IContract;
    use dojo::meta::IDeployedResource;

    component!(path: world_provider_cpt, storage: world_provider, event: WorldProviderEvent);
    component!(path: upgradeable_cpt, storage: upgradeable, event: UpgradeableEvent);

    #[abi(embed_v0)]
    impl WorldProviderImpl = world_provider_cpt::WorldProviderImpl<ContractState>;
    
    #[abi(embed_v0)]
    impl UpgradeableImpl = upgradeable_cpt::UpgradeableImpl<ContractState>;

    #[abi(embed_v0)]
    pub impl {name}__ContractImpl of IContract<ContractState> {{}}

    #[abi(embed_v0)]
    pub impl {name}__DeployedContractImpl of IDeployedResource<ContractState> {{
        fn dojo_name(self: @ContractState) -> ByteArray {{
            \"{name}\"
        }}
    }}

    #[generate_trait]
    impl {name}InternalImpl of {name}InternalTrait {{
        fn world(self: @ContractState, namespace: @ByteArray) -> dojo::world::storage::WorldStorage {{
            dojo::world::WorldStorageTrait::new(self.world_provider.world_dispatcher(), namespace)
        }}

        fn world_ns_hash(self: @ContractState, namespace_hash: felt252) -> dojo::world::storage::WorldStorage {{
            dojo::world::WorldStorageTrait::new_from_hash(self.world_provider.world_dispatcher(), namespace_hash)
        }}
    }}
");
        let name = tokenize(&name);
        let mut content = TokenStream::new(vec![tokenize(&content)]);
        content.extend(body.into_iter());

        quote! {
            #[starknet::contract]
            pub mod #name {
                #content
            }
        }
    }

    /// If a constructor is provided, we should keep the user statements.
    /// We only inject the world provider initializer.
    fn handle_constructor_fn(
        &mut self,
        db: &SimpleParserDatabase,
        fn_ast: &ast::FunctionWithBody,
    ) -> TokenStream {
        if !is_valid_constructor_params(db, &fn_ast) {
            self.diagnostics.push_error(
                "The constructor must have exactly one parameter, which is `ref self: \
                    ContractState`. Add a `dojo_init` function instead if you need to \
                    initialize the contract with parameters."
                    .to_string(),
            );
        }

        let ctor_decl = fn_ast.declaration(db).as_syntax_node();
        let ctor_decl = SyntaxNodeWithDb::new(&ctor_decl, db);

        let ctor_body = fn_ast.body(db).as_syntax_node();
        let ctor_body = SyntaxNodeWithDb::new(&ctor_body, db);

        quote! {
            #[constructor]
            #ctor_decl {
                self.world_provider.initializer();
                #ctor_body
            }
        }
    }

    fn handle_init_fn(
        &mut self,
        db: &SimpleParserDatabase,
        fn_ast: &ast::FunctionWithBody,
    ) -> TokenStream {
        if let OptionReturnTypeClause::ReturnTypeClause(_) =
            fn_ast.declaration(db).signature(db).ret_ty(db)
        {
            self.diagnostics.push_error(format!(
                "The {} function cannot have a return type.",
                DOJO_INIT_FN
            ));
        }

        let fn_decl = fn_ast.declaration(db).as_syntax_node();
        let fn_decl = SyntaxNodeWithDb::new(&fn_decl, db);

        quote! {
            #[abi(per_item)]
            #[generate_trait]
            pub impl IDojoInitImpl of IDojoInit {
                #[external(v0)]
                #fn_decl {
                    if starknet::get_caller_address() != self.world_provider.world_dispatcher().contract_address {
                        core::panics::panic_with_byte_array(
                            @format!(
                                "Only the world can init contract `{}`, but caller is `{:?}`",
                                self.dojo_name(),
                                starknet::get_caller_address()
                            )
                        );
                    }
                }
            }
        }
    }

    pub fn merge_event(
        &mut self,
        db: &SimpleParserDatabase,
        enum_ast: &ast::ItemEnum,
    ) -> TokenStream {
        let variants = enum_ast.variants(db).as_syntax_node();
        let variants = SyntaxNodeWithDb::new(&variants, db);

        quote! {
            #[event]
            #[derive(Drop, starknet::Event)]
            enum Event {
                UpgradeableEvent: upgradeable_cpt::Event,
                WorldProviderEvent: world_provider_cpt::Event,
                #variants
            }
        }
    }

    pub fn create_event(&mut self) -> TokenStream {
        quote! {
            #[event]
            #[derive(Drop, starknet::Event)]
            enum Event {
                UpgradeableEvent: upgradeable_cpt::Event,
                WorldProviderEvent: world_provider_cpt::Event,
            }
        }
    }

    pub fn merge_storage(
        &mut self,
        db: &SimpleParserDatabase,
        struct_ast: &ast::ItemStruct,
    ) -> TokenStream {
        let members = struct_ast.members(db).as_syntax_node();
        let members = SyntaxNodeWithDb::new(&members, db);

        quote! {
            #[storage]
            struct Storage {
                #[substorage(v0)]
                upgradeable: upgradeable_cpt::Storage,
                #[substorage(v0)]
                world_provider: world_provider_cpt::Storage,
                #members
            }
        }
    }

    pub fn create_storage(&mut self) -> TokenStream {
        quote! {
            #[storage]
            struct Storage {
                #[substorage(v0)]
                upgradeable: upgradeable_cpt::Storage,
                #[substorage(v0)]
                world_provider: world_provider_cpt::Storage,
            }
        }
    }
}

/// Checks if the constructor parameters are valid.
/// We only allow one parameter for the constructor, which is the contract state,
/// since `dojo_init` is called by the world after every resource has been deployed.
fn is_valid_constructor_params(db: &SimpleParserDatabase, fn_ast: &ast::FunctionWithBody) -> bool {
    let params = fn_ast
        .declaration(db)
        .signature(db)
        .parameters(db)
        .elements(db);
    params.len() == 1
        && params
            .first()
            .unwrap()
            .as_syntax_node()
            .get_text(db)
            .contains("ref self: ContractState")
}

/* TODO RBA/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_constructor_params_ok() {
        assert!(is_valid_constructor_params("ref self: ContractState"));
        assert!(is_valid_constructor_params("ref self: ContractState "));
        assert!(is_valid_constructor_params(" ref self: ContractState"));
    }

    #[test]
    fn test_is_valid_constructor_params_not_ok() {
        assert!(!is_valid_constructor_params(""));
        assert!(!is_valid_constructor_params("self: ContractState"));
        assert!(!is_valid_constructor_params("ref self: OtherState"));
        assert!(!is_valid_constructor_params("ref self: ContractState, other: felt252"));
        assert!(!is_valid_constructor_params("other: felt252"));
    }
}
*/
