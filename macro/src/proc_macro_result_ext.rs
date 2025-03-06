use cairo_lang_macro::{Diagnostics, ProcMacroResult, quote};
use crate::diagnostic_ext::DiagnosticsExt;

pub trait ProcMacroResultExt {
    fn fail(message: String) -> Self;
}

impl ProcMacroResultExt for ProcMacroResult {
    fn fail(message: String) -> Self {
        ProcMacroResult::new(quote!{}).with_diagnostics(Diagnostics::with_error(message))
    }
}
