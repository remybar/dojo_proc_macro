use cairo_lang_macro::{quote, TextSpan, Token, TokenStream, TokenTree};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::{ast, TypedSyntaxNode};

pub struct DojoTokenizer {}

impl DojoTokenizer {
    pub fn tokenize(s: &str) -> TokenTree {
        TokenTree::Ident(Token::new(s, TextSpan::call_site()))
    }

    pub fn rebuild_original_struct(
        db: &SimpleParserDatabase,
        struct_ast: &ast::ItemStruct,
    ) -> TokenStream {
        let visibility = Self::tokenize(&struct_ast.visibility(db).as_syntax_node().get_text(db));
        let name = Self::tokenize(&struct_ast.name(db).as_syntax_node().get_text(db));
        let generics = Self::tokenize(&struct_ast.generic_params(db).as_syntax_node().get_text(db));
        let members = Self::tokenize(
            &struct_ast
                .members(db)
                .elements(db)
                .iter()
                .map(|m| m.as_syntax_node().get_text(db))
                .collect::<Vec<_>>()
                .join(",\n"),
        );

        quote! {
            #visibility struct #name<#generics> {
                #members
            }
        }
    }
}
