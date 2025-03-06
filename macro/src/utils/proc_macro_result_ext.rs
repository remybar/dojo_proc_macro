use crate::utils::DiagnosticsExt;
use cairo_lang_macro::{quote, Diagnostics, ProcMacroResult};

pub trait ProcMacroResultExt {
    fn fail(message: String) -> Self;
}

impl ProcMacroResultExt for ProcMacroResult {
    fn fail(message: String) -> Self {
        ProcMacroResult::new(quote! {}).with_diagnostics(Diagnostics::with_error(message))
    }
}
