use cairo_lang_macro::{Diagnostic, Diagnostics, Severity};

pub trait DiagnosticsExt {
    fn with_error(message: String) -> Self;
}

impl DiagnosticsExt for Diagnostics {
    fn with_error(message: String) -> Self {
        Self::new(vec![Diagnostic::error(message)])
    }
}

pub trait DiagnosticExt {
    fn to_pretty_string(&self) -> String;
}

impl DiagnosticExt for Diagnostic {
    fn to_pretty_string(&self) -> String {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };

        format!("[{severity}] {}", self.message)
    }
}
