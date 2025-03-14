use cairo_lang_macro::{quote, ProcMacroResult, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast;

pub(crate) fn process(
    _db: &SimpleParserDatabase,
    _original_struct: TokenStream,
    _struct_ast: &ast::ItemStruct,
) -> ProcMacroResult {
    ProcMacroResult::new(quote! {})
}
