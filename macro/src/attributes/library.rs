use cairo_lang_macro::{quote, ProcMacroResult, TokenStream};

pub(crate) fn process(_token_stream: TokenStream) -> ProcMacroResult {
    ProcMacroResult::new(quote! {})
}
