use cairo_lang_macro::{inline_macro, ProcMacroResult, TokenStream};

#[inline_macro]
pub fn bytearray_hash(token_stream: TokenStream) -> ProcMacroResult {
    super::bytearray_hash::process(token_stream)
}
