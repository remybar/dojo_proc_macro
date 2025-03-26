use cairo_lang_macro::ProcMacroResult;

use crate::helpers::DiagnosticExt;

pub fn debug_macro(element: &str, res: &ProcMacroResult) {
    let element = element.to_ascii_uppercase();

    if std::env::var("DOJO_DEBUG_MACRO").is_ok() || std::env::var(format!("DOJO_DEBUG_{element}_MACRO")).is_ok() {
        let content = format!("content:\n{}", res.token_stream);
        let diagnostics = if res.diagnostics.is_empty() {
            "".to_string()
        } else {
            format!(
                "diagnostics:\n{}",
                res.diagnostics
                    .iter()
                    .map(|d| d.to_pretty_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        println!("\n*> MACRO {element}\n>>>>>>>>>>>>>>>>>>>>>>>>>>>\n{content}\n{diagnostics}\n<<<<<<<<<<<<<<<<<<<<<<<<<<<");
    }
}
