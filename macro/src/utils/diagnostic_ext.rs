use cairo_lang_macro::{Diagnostic, Diagnostics};

pub trait DiagnosticsExt {
    fn with_error(message: String) -> Self;
}

impl DiagnosticsExt for Diagnostics {
    fn with_error(message: String) -> Self {
        Self::new(vec![Diagnostic::error(message)])
    }
}
