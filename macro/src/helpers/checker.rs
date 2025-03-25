use cairo_lang_macro::Diagnostic;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::Attribute;

use crate::constants::{DOJO_INTROSPECT_DERIVE, DOJO_PACKED_DERIVE};
use crate::helpers::{DiagnosticsExt, DojoParser};

pub struct DojoChecker {}

impl DojoChecker {
    pub fn check_derive_conflicts(
        db: &SimpleParserDatabase,
        diagnostics: &mut Vec<Diagnostic>,
        attrs: Vec<Attribute>,
    ) {
        let attr_names = DojoParser::extract_derive_attr_names(db, diagnostics, attrs);

        if attr_names.contains(&DOJO_INTROSPECT_DERIVE.to_string())
            && attr_names.contains(&DOJO_PACKED_DERIVE.to_string())
        {
            diagnostics.push_error(
                format!("{DOJO_INTROSPECT_DERIVE} and {DOJO_PACKED_DERIVE} attributes cannot be used at a same time.")
            );
        }
    }
}
