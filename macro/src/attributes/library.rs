use cairo_lang_macro::{quote, Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::Terminal;
use cairo_lang_syntax::node::{
    ast::{self, MaybeModuleBody},
    kind::SyntaxKind::ItemModule,
    TypedSyntaxNode,
};

use crate::constants::{CONSTRUCTOR_FN, DOJO_INIT_FN};
use crate::helpers::{DojoTokenizer, ProcMacroResultExt};
use dojo_types::naming;

#[derive(Debug)]
pub struct DojoLibrary {
    diagnostics: Vec<Diagnostic>,
}

impl DojoLibrary {
    pub fn process(token_stream: TokenStream) -> ProcMacroResult {
        let db = SimpleParserDatabase::default();
        let (root_node, _diagnostics) = db.parse_token_stream(&token_stream);

        for n in root_node.descendants(&db) {
            if n.kind(&db) == ItemModule {
                let module_ast = ast::ItemModule::from_syntax_node(&db, n);
                return DojoLibrary::process_ast(&db, &module_ast);
            }
        }

        ProcMacroResult::fail(format!("'dojo::library' must be used on module only."))
    }

    fn process_ast(db: &SimpleParserDatabase, module_ast: &ast::ItemModule) -> ProcMacroResult {
        let mut library = DojoLibrary {
            diagnostics: vec![],
        };

        let name = module_ast.name(db).text(db).to_string();

        if !naming::is_name_valid(&name) {
            return ProcMacroResult::fail(format!(
                "The library name '{name}' can only contain characters (a-z/A-Z), \
                digits (0-9) and underscore (_)."
            ));
        }

        let mut has_event = false;
        let mut has_storage = false;
        let mut has_init = false;
        let mut has_constructor = false;

        if let MaybeModuleBody::Some(body) = module_ast.body(db) {
            let mut body_nodes: Vec<_> = body
                .items(db)
                .elements(db)
                .iter()
                .map(|el| {
                    if let ast::ModuleItem::Enum(ref enum_ast) = el {
                        if enum_ast.name(db).text(db).to_string() == "Event" {
                            has_event = true;
                            return library.merge_event(db, enum_ast.clone());
                        }
                    } else if let ast::ModuleItem::Struct(ref struct_ast) = el {
                        if struct_ast.name(db).text(db).to_string() == "Storage" {
                            has_storage = true;
                            return library.merge_storage(db, struct_ast.clone());
                        }
                    } else if let ast::ModuleItem::FreeFunction(ref fn_ast) = el {
                        let fn_decl = fn_ast.declaration(db);
                        let fn_name = fn_decl.name(db).text(db);

                        if fn_name == CONSTRUCTOR_FN {
                            has_constructor = true;
                        }

                        if fn_name == DOJO_INIT_FN {
                            has_init = true;
                        }
                    }

                    let el = el.as_syntax_node();
                    let el = SyntaxNodeWithDb::new(&el, db);
                    quote! { #el }
                })
                .collect::<Vec<TokenStream>>();

            if has_constructor {
                return ProcMacroResult::fail(format!(
                    "The library {name} cannot have a constructor"
                ));
            }

            if has_init {
                return ProcMacroResult::fail(format!(
                    "The library {name} cannot have a dojo_init"
                ));
            }

            if !has_event {
                body_nodes.push(library.create_event())
            }

            if !has_storage {
                body_nodes.push(library.create_storage())
            }

            let library_code = DojoLibrary::generate_library_code(&name, body_nodes);
            return ProcMacroResult::new(library_code)
                .with_diagnostics(Diagnostics::new(library.diagnostics));
        }

        ProcMacroResult::fail(format!("The library '{name}' is empty."))
    }

    fn generate_library_code(name: &String, body: Vec<TokenStream>) -> TokenStream {
        let content = format!(
            "use dojo::contract::components::world_provider::{{world_provider_cpt, IWorldProvider}};
    use dojo::contract::ILibrary;
    use dojo::meta::IDeployedResource;

    component!(path: world_provider_cpt, storage: world_provider, event: WorldProviderEvent);
   
    #[abi(embed_v0)]
    impl WorldProviderImpl = world_provider_cpt::WorldProviderImpl<ContractState>;
   
    #[abi(embed_v0)]
    pub impl {name}__LibraryImpl of ILibrary<ContractState> {{}}

    #[abi(embed_v0)]
    pub impl {name}__DeployedLibraryImpl of IDeployedResource<ContractState> {{
        fn dojo_name(self: @ContractState) -> ByteArray {{
            \"{name}\"
        }}
    }}

    #[generate_trait]
    impl {name}InternalImpl of {name}InternalTrait {{
        fn world(self: @ContractState, namespace: @ByteArray) -> dojo::world::storage::WorldStorage {{
            dojo::world::WorldStorageTrait::new(self.world_provider.world_dispatcher(), namespace)
        }}
    }}");

        let name = DojoTokenizer::tokenize(&name);
        let mut content = TokenStream::new(vec![DojoTokenizer::tokenize(&content)]);
        content.extend(body.into_iter());

        quote! {
            #[starknet::contract]
            pub mod #name {
                #content
            }
        }
    }

    pub fn merge_event(
        &mut self,
        db: &SimpleParserDatabase,
        enum_ast: ast::ItemEnum,
    ) -> TokenStream {
        let variants = enum_ast.variants(db).as_syntax_node();
        let variants = SyntaxNodeWithDb::new(&variants, db);

        quote! {
            #[event]
            #[derive(Drop, starknet::Event)]
            enum Event {
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
                #[flat]
                WorldProviderEvent: world_provider_cpt::Event,
            }
        }
    }

    pub fn merge_storage(
        &mut self,
        db: &SimpleParserDatabase,
        struct_ast: ast::ItemStruct,
    ) -> TokenStream {
        let members = struct_ast.members(db).as_syntax_node();
        let members = SyntaxNodeWithDb::new(&members, db);

        quote! {
            #[storage]
            struct Storage {
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
                world_provider: world_provider_cpt::Storage,
            }
        }
    }
}
