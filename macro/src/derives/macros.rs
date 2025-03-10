use crate::utils::debug_macro;
use cairo_lang_macro::{derive_macro, ProcMacroResult, TokenStream};

#[derive_macro]
pub fn introspect(token_stream: TokenStream) -> ProcMacroResult {
    let output = super::introspect::process(token_stream, false);

    debug_macro("Introspect", &output);
    output
}

#[derive_macro]
pub fn introspect_packed(token_stream: TokenStream) -> ProcMacroResult {
    let output = super::introspect::process(token_stream, true);

    debug_macro("IntrospectPacked", &output);
    output
}
