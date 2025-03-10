use cairo_lang_macro::{quote, ProcMacroResult, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::kind::SyntaxKind::{ItemEnum, ItemStruct};
use cairo_lang_syntax::node::{ast, TypedSyntaxNode};

use crate::utils::tokenize;

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

    for n in root_node.descendants(&db) {
        match n.kind(&db) {
            ItemStruct => {
                let struct_ast = ast::ItemStruct::from_syntax_node(&db, n);
                return structs::process_struct_introspect(&db, &struct_ast, is_packed);
            }
            ItemEnum => {
                let enum_ast = ast::ItemEnum::from_syntax_node(&db, n);
                return enums::process_enum_introspect(&db, &enum_ast, is_packed);
            }
            _ => {}
        }
    }

    ProcMacroResult::new(quote! {})
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
) -> ProcMacroResult {
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

    ProcMacroResult::new(quote! {
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
    })
}
