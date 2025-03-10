use cairo_lang_macro::{ProcMacroResult, TextSpan, Token, TokenTree};

pub mod diagnostic_ext;
pub use diagnostic_ext::*;

pub mod proc_macro_result_ext;
pub use proc_macro_result_ext::*;

pub fn tokenize(s: &str) -> TokenTree {
    TokenTree::Ident(Token::new(s, TextSpan::call_site()))
}

pub fn debug_macro(element: &str, res: &ProcMacroResult) {
    if std::env::var("DOJO_DEBUG_MACRO").is_ok() {
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
