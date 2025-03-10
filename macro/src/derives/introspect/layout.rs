use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{Expr, TypeClause};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

use super::utils::{
    get_array_item_type, get_tuple_item_types, is_array, is_byte_array, is_tuple,
    is_unsupported_option_type, primitive_type_introspection,
};

/// Build a field layout describing the provided type clause.
pub(crate) fn get_layout_from_type_clause(
    db: &SimpleParserDatabase,
    type_clause: &TypeClause,
) -> anyhow::Result<String> {
    match type_clause.ty(db) {
        Expr::Path(path) => {
            let path_type = path.as_syntax_node().get_text(db);
            build_item_layout_from_type(&path_type)
        }
        Expr::Tuple(expr) => {
            let tuple_type = expr.as_syntax_node().get_text(db);
            build_tuple_layout_from_type(&tuple_type)
        }
        _ => {
            anyhow::bail!("Unexpected expression for variant data type.".to_string());
        }
    }
}

/// Build the array layout describing the provided array type.
/// item_type could be something like `Array<u128>` for example.
pub fn build_array_layout_from_type(item_type: &str) -> anyhow::Result<String> {
    let array_item_type = get_array_item_type(item_type);

    if is_tuple(&array_item_type) {
        let layout = build_item_layout_from_type(&array_item_type)?;
        Ok(format!(
            "dojo::meta::Layout::Array(
                array![
                    {layout}
                ].span()
            )"
        ))
    } else if is_array(&array_item_type) {
        let layout = build_array_layout_from_type(&array_item_type)?;
        Ok(format!(
            "dojo::meta::Layout::Array(
                array![
                    {layout}
                ].span()
            )"
        ))
    } else {
        Ok(format!(
            "dojo::meta::introspect::Introspect::<{}>::layout()",
            item_type
        ))
    }
}

/// Build the tuple layout describing the provided tuple type.
/// item_type could be something like (u8, u32, u128) for example.
pub fn build_tuple_layout_from_type(item_type: &str) -> anyhow::Result<String> {
    let mut tuple_items = vec![];

    for item in get_tuple_item_types(item_type).iter() {
        let layout = build_item_layout_from_type(item)?;
        tuple_items.push(layout);
    }

    Ok(format!(
        "dojo::meta::Layout::Tuple(
            array![
            {}
            ].span()
        )",
        tuple_items.join(",\n")
    ))
}

/// Build the layout describing the provided type.
/// item_type could be any type (array, tuple, struct, ...)
pub fn build_item_layout_from_type(item_type: &str) -> anyhow::Result<String> {
    if is_array(item_type) {
        build_array_layout_from_type(item_type)
    } else if is_tuple(item_type) {
        build_tuple_layout_from_type(item_type)
    } else {
        // For Option<T>, T cannot be a tuple
        if is_unsupported_option_type(item_type) {
            anyhow::bail!("Option<T> cannot be used with tuples. Prefer using a struct.");
        }

        // `usize` is forbidden because its size is architecture-dependent
        if item_type == "usize" {
            anyhow::bail!("Use u32 rather than usize as usize size is architecture dependent.");
        }

        Ok(format!(
            "dojo::meta::introspect::Introspect::<{}>::layout()",
            item_type
        ))
    }
}

pub fn is_custom_layout(layout: &str) -> bool {
    layout.starts_with("dojo::meta::introspect::Introspect::")
}

pub fn generate_cairo_code_for_fixed_layout_with_custom_types(layouts: &[String]) -> String {
    let layouts_repr = layouts
        .iter()
        .map(|l| {
            if is_custom_layout(l) {
                l.to_string()
            } else {
                format!("dojo::meta::Layout::Fixed(array![{l}].span())")
            }
        })
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        "let mut layouts = array![
            {layouts_repr}
        ];
        let mut merged_layout = ArrayTrait::<u8>::new();

        loop {{
            match ArrayTrait::pop_front(ref layouts) {{
                Option::Some(mut layout) => {{
                    match layout {{
                        dojo::meta::Layout::Fixed(mut l) => {{
                            loop {{
                                match SpanTrait::pop_front(ref l) {{
                                    Option::Some(x) => merged_layout.append(*x),
                                    Option::None(_) => {{ break; }}
                                }};
                            }};
                        }},
                        _ => panic!(\"A packed model layout must contain Fixed layouts only.\"),
                    }};
                }},
                Option::None(_) => {{ break; }}
            }};
        }};

        dojo::meta::Layout::Fixed(merged_layout.span())
        ",
    )
}

//
pub fn get_packed_field_layout_from_type_clause(
    db: &dyn SyntaxGroup,
    type_clause: &TypeClause,
) -> anyhow::Result<Vec<String>> {
    match type_clause.ty(db) {
        Expr::Path(path) => {
            let path_type = path.as_syntax_node().get_text(db);
            get_packed_item_layout_from_type(path_type.trim())
        }
        Expr::Tuple(expr) => {
            let tuple_type = expr.as_syntax_node().get_text(db);
            get_packed_tuple_layout_from_type(&tuple_type)
        }
        _ => {
            anyhow::bail!("Unexpected expression for variant data type.");
        }
    }
}

//
pub fn get_packed_item_layout_from_type(item_type: &str) -> anyhow::Result<Vec<String>> {
    if is_array(item_type) || is_byte_array(item_type) {
        anyhow::bail!("Array field cannot be packed.");
    } else if is_tuple(item_type) {
        get_packed_tuple_layout_from_type(item_type)
    } else {
        let primitives = primitive_type_introspection();

        let res = if let Some(p) = primitives.get(item_type) {
            vec![p
                .1
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")]
        } else {
            // as we cannot verify that an enum/struct custom type is packable,
            // we suppose it is and let the user verify this.
            // If it's not the case, the Dojo model layout function will panic.
            vec![format!(
                "dojo::meta::introspect::Introspect::<{}>::layout()",
                item_type
            )]
        };

        Ok(res)
    }
}

//
pub fn get_packed_tuple_layout_from_type(item_type: &str) -> anyhow::Result<Vec<String>> {
    let mut layouts = vec![];

    for item in get_tuple_item_types(item_type).iter() {
        let layout = get_packed_item_layout_from_type(item)?;
        layouts.push(layout);
    }

    Ok(layouts.into_iter().flatten().collect::<Vec<_>>())
}
