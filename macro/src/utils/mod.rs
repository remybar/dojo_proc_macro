use cairo_lang_macro::{ProcMacroResult, TextSpan, Token, TokenTree};
use cairo_lang_syntax::node::{ast::Member as MemberAst, Terminal, TypedSyntaxNode};
use dojo_types::naming::compute_bytearray_hash;
use starknet_crypto::{poseidon_hash_many, Felt};

pub mod diagnostic_ext;
use cairo_lang_parser::utils::SimpleParserDatabase;
pub use diagnostic_ext::*;

pub mod proc_macro_result_ext;
pub use proc_macro_result_ext::*;

pub(crate) fn tokenize(s: &str) -> TokenTree {
    TokenTree::Ident(Token::new(s, TextSpan::call_site()))
}

pub(crate) fn debug_macro(element: &str, res: &ProcMacroResult) {
    if std::env::var("DOJO_DEBUG_MACRO").is_ok() {
        let content = format!("content:\n{}", res.token_stream);
        let diagnostics = if res.diagnostics.is_empty() {
            "".to_string()
        } else {
            format!(
                "diagnostics:\n{}",
                res.diagnostics
                    .iter()
                    .map(|d| d.to_pretty_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        println!("\n*> MACRO {element}\n>>>>>>>>>>>>>>>>>>>>>>>>>>>\n{content}\n{diagnostics}\n<<<<<<<<<<<<<<<<<<<<<<<<<<<");
    }
}

/// Compute a unique hash based on the element name and types and names of members.
/// This hash is used in element contracts to ensure uniqueness.
pub(crate) fn compute_unique_hash(
    db: &SimpleParserDatabase,
    element_name: &str,
    is_packed: bool,
    members: &[MemberAst],
) -> Felt {
    let mut hashes = vec![
        if is_packed { Felt::ONE } else { Felt::ZERO },
        compute_bytearray_hash(element_name),
    ];
    hashes.extend(
        members
            .iter()
            .map(|m| {
                poseidon_hash_many(&[
                    compute_bytearray_hash(&m.name(db).text(db).to_string()),
                    compute_bytearray_hash(
                        m.type_clause(db)
                            .ty(db)
                            .as_syntax_node()
                            .get_text(db)
                            .trim(),
                    ),
                ])
            })
            .collect::<Vec<_>>(),
    );
    poseidon_hash_many(&hashes)
}
