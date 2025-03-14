use cairo_lang_macro::Diagnostic;
use cairo_lang_macro::TokenStream;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::ItemStruct;
use cairo_lang_syntax::node::ast::Member;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};
use starknet::core::utils::get_selector_from_name;

use crate::constants::CAIRO_DELIMITERS;
use crate::derives::helpers;
use crate::utils::DiagnosticsExt;

/// TODO RBA
pub fn process_struct_introspect(
    db: &SimpleParserDatabase,
    diagnostics: &mut Vec<Diagnostic>,
    struct_ast: &ItemStruct,
    is_packed: bool,
) -> TokenStream {
    let derive_attrs = struct_ast.attributes(db).query_attr(db, "derive");

    helpers::check_derive_attrs_conflicts(db, diagnostics, derive_attrs);

    generate_struct_introspect(db, diagnostics, struct_ast, is_packed)
}

/// TODO RBA
fn generate_struct_introspect(
    db: &SimpleParserDatabase,
    diagnostics: &mut Vec<Diagnostic>,
    struct_ast: &ItemStruct,
    is_packed: bool,
) -> TokenStream {
    let struct_name = struct_ast.name(db).text(db).into();
    let struct_size = compute_struct_layout_size(db, struct_ast, is_packed);
    let ty = build_struct_ty(db, &struct_name, struct_ast);

    let layout = if is_packed {
        build_packed_struct_layout(db, diagnostics, struct_ast)
    } else {
        format!(
            "dojo::meta::Layout::Struct(
                array![
                {}
                ].span()
            )",
            build_struct_field_layouts(db, diagnostics, struct_ast)
        )
    };

    let (gen_types, gen_impls) =
        super::generics::build_generic_types_and_impls(db, struct_ast.generic_params(db));

    super::generate_introspect(
        &struct_name,
        &struct_size,
        &gen_types,
        gen_impls,
        &layout,
        &ty,
    )
}

// TODO RBA
fn compute_struct_layout_size(
    db: &SimpleParserDatabase,
    struct_ast: &ItemStruct,
    is_packed: bool,
) -> String {
    let mut cumulated_sizes = 0;
    let mut is_dynamic_size = false;

    let mut sizes = struct_ast
        .members(db)
        .elements(db)
        .into_iter()
        .filter_map(|m| {
            if m.has_attr(db, "key") {
                return None;
            }

            let (sizes, cumulated, is_dynamic) =
                super::size::get_field_size_from_type_clause(db, &m.type_clause(db));

            cumulated_sizes += cumulated;
            is_dynamic_size |= is_dynamic;
            Some(sizes)
        })
        .flatten()
        .collect::<Vec<_>>();
    super::size::build_size_function_body(&mut sizes, cumulated_sizes, is_dynamic_size, is_packed)
}

pub fn build_member_ty(db: &SimpleParserDatabase, member: &Member) -> String {
    let name = member.name(db).text(db).to_string();
    let attrs = if member.has_attr(db, "key") {
        vec!["'key'"]
    } else {
        vec![]
    };

    format!(
        "dojo::meta::introspect::Member {{
            name: '{name}',
            attrs: array![{}].span(),
            ty: {}
        }}",
        attrs.join(","),
        super::ty::build_ty_from_type_clause(db, &member.type_clause(db))
    )
}

fn build_struct_ty(db: &SimpleParserDatabase, name: &String, struct_ast: &ItemStruct) -> String {
    let members_ty = struct_ast
        .members(db)
        .elements(db)
        .iter()
        .map(|m| build_member_ty(db, m))
        .collect::<Vec<_>>();

    format!(
        "dojo::meta::introspect::Ty::Struct(
            dojo::meta::introspect::Struct {{
                name: '{name}',
                attrs: array![].span(),
                children: array![
                {}\n
                ].span()
            }}
        )",
        members_ty.join(",\n")
    )
}

/// build the full layout for every field in the Struct.
pub fn build_struct_field_layouts(
    db: &SimpleParserDatabase,
    diagnostics: &mut Vec<Diagnostic>,
    struct_ast: &ItemStruct,
) -> String {
    let mut members = vec![];

    for member in struct_ast.members(db).elements(db).iter() {
        if member.has_attr(db, "key") {
            let member_type = member.type_clause(db).ty(db).as_syntax_node().get_text(db);

            // Check if the member type uses the `usize` type, either
            // directly or as a nested type (the tuple (u8, usize, u32) for example)
            if type_contains_usize(member_type) {
                diagnostics.push_error(
                    "Use u32 rather than usize for model keys, as usize size is \
                                architecture dependent."
                        .to_string(),
                );
            }

            let field_name = member.name(db).text(db);
            let field_selector = get_selector_from_name(field_name.as_ref()).unwrap();
            let field_layout = super::layout::get_layout_from_type_clause(
                db,
                diagnostics,
                &member.type_clause(db),
            );

            members.push(format!(
                "dojo::meta::FieldLayout {{
                    selector: {field_selector},
                    layout: {field_layout}
                }}"
            ));
        }
    }

    members.join(",\n")
}

fn build_packed_struct_layout(
    db: &SimpleParserDatabase,
    diagnostics: &mut Vec<Diagnostic>,
    struct_ast: &ItemStruct,
) -> String {
    let mut layouts = vec![];

    for member in struct_ast
        .members(db)
        .elements(db)
        .iter()
        .filter(|m| !m.has_attr(db, "key"))
    {
        let layout = super::layout::get_packed_field_layout_from_type_clause(
            db,
            diagnostics,
            &member.type_clause(db),
        );
        layouts.push(layout)
    }

    let layouts = layouts.into_iter().flatten().collect::<Vec<_>>();

    if layouts
        .iter()
        .any(|v| super::layout::is_custom_layout(v.as_str()))
    {
        super::layout::generate_cairo_code_for_fixed_layout_with_custom_types(&layouts)
    } else {
        format!(
            "dojo::meta::Layout::Fixed(
            array![
            {}
            ].span()
        )",
            layouts.join(",")
        )
    }
}

fn type_contains_usize(type_str: String) -> bool {
    type_str.contains("usize")
        && type_str
            .split(CAIRO_DELIMITERS)
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .contains(&"usize")
}
