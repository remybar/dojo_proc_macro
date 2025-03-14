use cairo_lang_macro::{quote, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::kind::SyntaxKind::{ItemEnum, ItemStruct};
use cairo_lang_syntax::node::{ast, TypedSyntaxNode};

use crate::utils::{proc_macro_result_ext::ProcMacroResultExt, tokenize};

mod enums;
mod generics;
mod layout;
mod size;
mod structs;

mod ty;
mod utils;

pub(crate) fn process(token_stream: TokenStream, is_packed: bool) -> ProcMacroResult {
    let db = SimpleParserDatabase::default();
    let (root_node, _diagnostics) = db.parse_token_stream(&token_stream);
    let mut diagnostics = vec![];

    for n in root_node.descendants(&db) {
        match n.kind(&db) {
            ItemStruct => {
                let struct_ast = ast::ItemStruct::from_syntax_node(&db, n);
                let token = structs::process_struct_introspect(
                    &db,
                    &mut diagnostics,
                    &struct_ast,
                    is_packed,
                );
                return ProcMacroResult::new(token).with_diagnostics(Diagnostics::new(diagnostics));
            }
            ItemEnum => {
                let enum_ast = ast::ItemEnum::from_syntax_node(&db, n);
                let token =
                    enums::process_enum_introspect(&db, &mut diagnostics, &enum_ast, is_packed);
                return ProcMacroResult::new(token).with_diagnostics(Diagnostics::new(diagnostics));
            }
            _ => {}
        }
    }

    ProcMacroResult::fail("derive Introspect: unsupported syntax node.".to_string())
}

/// Generate the introspect impl for a Struct or an Enum,
/// based on its name, size, layout and Ty.
pub(crate) fn generate_introspect(
    name: &String,
    size: &str,
    generic_types: &[String],
    generic_impls: String,
    layout: &str,
    ty: &str,
) -> TokenStream {
    let impl_decl = if generic_types.is_empty() {
        format!("{name}Introspect of dojo::meta::introspect::Introspect<{name}>")
    } else {
        format!(
            "{name}Introspect<{generic_impls}> of dojo::meta::introspect::Introspect<{name}<{}>>",
            generic_types.join(", ")
        )
    };
    let impl_decl = tokenize(&impl_decl);

    let size = tokenize(size);
    let layout = tokenize(layout);
    let ty = tokenize(ty);

    quote! {
        impl #impl_decl {
            #[inline(always)]
            fn size() -> Option<usize> {
                #size
            }

            fn layout() -> dojo::meta::Layout {
                #layout
            }

            #[inline(always)]
            fn ty() -> dojo::meta::introspect::Ty {
                #ty
            }
        }
    }
}
