use cairo_lang_syntax::node::{ast, db::SyntaxGroup, kind::SyntaxKind::ExprParenthesized, Terminal, TypedSyntaxNode};

use cairo_lang_macro::{
    inline_macro, quote, ProcMacroResult, TextSpan, Token, TokenStream, TokenTree
};
use cairo_lang_parser::utils::SimpleParserDatabase;
use smol_str::ToSmolStr;

use dojo_types::naming;

use crate::proc_macro_result_ext::ProcMacroResultExt;

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
    
    ProcMacroResult::fail(format!("bytearray_hash: invalid parameter (arg: {token_stream})"))
}

fn process_args(db: &dyn SyntaxGroup, expr: &ast::ExprParenthesized) -> ProcMacroResult {
    if let ast::Expr::String(s) = expr.expr(db) {
        let input = s.text(db).to_string();
        let hash = naming::compute_bytearray_hash(&input);
        let hash = format!("{:#64x}", hash);

        let token = TokenTree::Ident(Token::new(hash.to_smolstr(), TextSpan::call_site()));
        return ProcMacroResult::new(quote! { #token });
    }

    ProcMacroResult::fail(format!("bytearray_hash: invalid parameter type (arg: {})", expr.as_syntax_node().get_text(db)))
}

#[cfg(test)]
mod tests {
    use cairo_lang_macro::{Severity, TokenStream};
    use super::*;

    #[test]
    fn test_with_bad_inputs() {

        // input without parenthesis
        let input = "hello";
        let res = process_bytearray_hash(TokenStream::new(vec![
            TokenTree::Ident(Token::new(input, TextSpan::call_site()))
        ]));

        assert_eq!(res.diagnostics.len(), 1);
        
        assert_eq!(res.diagnostics[0].severity, Severity::Error);
        assert_eq!(res.diagnostics[0].message, "bytearray_hash: invalid parameter (arg: hello)".to_string());

        // bad input type
        let input = "(1234)";
        let res = process_bytearray_hash(TokenStream::new(vec![
            TokenTree::Ident(Token::new(input, TextSpan::call_site()))
        ]));

        assert_eq!(res.diagnostics.len(), 1);
        
        assert_eq!(res.diagnostics[0].severity, Severity::Error);
        assert_eq!(res.diagnostics[0].message, "bytearray_hash: invalid parameter type (arg: (1234))".to_string());
    }

    #[test]
    fn test_with_valid_input() {
        let input = "(\"hello\")";
        let expected = "0x3244ef30a5e431f958f5ee38a0726e8b1997bb7654b164218ac4a01fb9e2646";

        let res = process_bytearray_hash(TokenStream::new(vec![
            TokenTree::Ident(Token::new(input, TextSpan::call_site()))
        ]));

        assert_eq!(res.diagnostics.len(), 0);
        assert_eq!(res.token_stream, TokenStream::new(vec![
            TokenTree::Ident(Token::new(expected, TextSpan::call_site()))
        ]));
    }
}