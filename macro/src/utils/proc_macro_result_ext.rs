use crate::utils::DiagnosticsExt;
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};

pub trait ProcMacroResultExt {
    fn fail(message: String) -> Self;
    fn fail_with_diagnostics(diagnostics: Vec<Diagnostic>) -> Self;
    fn empty() -> Self;
}

impl ProcMacroResultExt for ProcMacroResult {
    fn fail(message: String) -> Self {
        Self::fail_with_diagnostics(Vec::<Diagnostic>::with_error(message))
    }
    fn fail_with_diagnostics(diagnostics: Vec<Diagnostic>) -> Self {
        Self::empty().with_diagnostics(Diagnostics::new(diagnostics))
    }
    fn empty() -> Self {
        ProcMacroResult::new(TokenStream::empty())
    }
}
