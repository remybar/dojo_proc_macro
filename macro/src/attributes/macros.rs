use crate::utils::debug_macro;
use cairo_lang_macro::{attribute_macro, ProcMacroResult, TokenStream};

use super::contract::DojoContract;
use super::library::DojoLibrary;

#[attribute_macro(parent = "dojo")]
pub fn model(_args: TokenStream, token_stream: TokenStream) -> ProcMacroResult {
    let output = super::helpers::process_struct(token_stream, super::model::process);

    debug_macro("dojo::model", &output);
    output
}

#[attribute_macro(parent = "dojo")]
pub fn event(_args: TokenStream, token_stream: TokenStream) -> ProcMacroResult {
    let output = super::helpers::process_struct(token_stream, super::event::process);

    debug_macro("dojo::event", &output);
    output
}

#[attribute_macro(parent = "dojo")]
pub fn contract(_args: TokenStream, token_stream: TokenStream) -> ProcMacroResult {
    let output = DojoContract::process(token_stream);

    debug_macro("dojo::contract", &output);
    output
}

#[attribute_macro(parent = "dojo")]
pub fn library(_args: TokenStream, token_stream: TokenStream) -> ProcMacroResult {
    let output = DojoLibrary::process(token_stream);

    debug_macro("dojo::library", &output);
    output
}
