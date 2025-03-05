use cairo_lang_syntax::node::{ast, db::SyntaxGroup, kind::SyntaxKind::ExprParenthesized, Terminal, TypedSyntaxNode};

use cairo_lang_macro::{
    inline_macro, quote, ProcMacroResult, TextSpan, TokenStream, TokenTree, Token
};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cainome::cairo_serde::{ByteArray, CairoSerde};
use smol_str::ToSmolStr;
use starknet_crypto::poseidon_hash_many;

use dojo_types::naming;

#[inline_macro]
pub fn bytearray_hash(token_stream: TokenStream) -> ProcMacroResult {
    process_bytearray_hash(token_stream)
}

fn process_bytearray_hash(token_stream: TokenStream) -> ProcMacroResult {
    let db = SimpleParserDatabase::default();
    let (root_node, _diagnostics) = db.parse_token_stream_expr(&token_stream);

    for n in root_node.descendants(&db) {
        if n.kind(&db) == ExprParenthesized {
            let node = ast::ExprParenthesized::from_syntax_node(&db, n);
            return process_args(&db, &node);
        }
    }
    
    // TODO RBA: raise an error
    ProcMacroResult::new(quote! {})
}

fn process_args(db: &dyn SyntaxGroup, expr: &ast::ExprParenthesized) -> ProcMacroResult {
    let tokens = match expr.expr(db) {
        ast::Expr::String(s) => {
            let input = s.text(db).to_string();
            let hash = naming::compute_bytearray_hash(input);
            let hash = format!("{:#64x}", hash);

            let token = TokenTree::Ident(Token::new(hash.to_smolstr(), TextSpan::call_site()));
            quote! { #token }
        },
        _ => {
            panic!("bytearray_hash: argument not supported")
        }
    };

    ProcMacroResult::new(tokens)
}
