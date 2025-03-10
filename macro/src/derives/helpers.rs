use anyhow;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::attribute::structured::AttributeArgVariant;
use cairo_lang_syntax::attribute::structured::AttributeStructurize;
use cairo_lang_syntax::node::ast;
use cairo_lang_syntax::node::ast::Attribute;
use cairo_lang_syntax::node::Terminal;

use crate::constants::{DOJO_INTROSPECT_DERIVE, DOJO_PACKED_DERIVE};

/// Extracts the names of the derive attributes from the given attributes.
///
/// # Examples
///
/// Derive usage should look like this:
///
/// ```no_run,ignore
/// #[derive(Introspect)]
/// struct MyStruct {}
/// ```
///
/// And this function will return `["Introspect"]`.
pub fn extract_derive_attr_names(db: &SimpleParserDatabase, attrs: Vec<Attribute>) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            let args = attr.clone().structurize(db).args;
            if args.is_empty() {
                None
            } else {
                Some(args.into_iter().filter_map(|a| {
                    if let AttributeArgVariant::Unnamed(ast::Expr::Path(path)) = a.variant {
                        if let [ast::PathSegment::Simple(segment)] = &path.elements(db)[..] {
                            Some(segment.ident(db).text(db).to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }))
            }
        })
        .flatten()
        .collect::<Vec<_>>()
}

/// TODO RBA
pub fn check_derive_attrs_conflicts(
    db: &SimpleParserDatabase,
    attrs: Vec<Attribute>,
) -> anyhow::Result<()> {
    let attr_names = extract_derive_attr_names(db, attrs);

    if attr_names.contains(&DOJO_INTROSPECT_DERIVE.to_string())
        && attr_names.contains(&DOJO_PACKED_DERIVE.to_string())
    {
        anyhow::bail!(
            "{} and {} attributes cannot be used at a same time.",
            DOJO_INTROSPECT_DERIVE,
            DOJO_PACKED_DERIVE
        )
    } else {
        Ok(())
    }
}
