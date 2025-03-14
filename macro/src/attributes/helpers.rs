use cairo_lang_macro::{quote, Diagnostic, ProcMacroResult, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::kind::SyntaxKind::ItemStruct;
use cairo_lang_syntax::node::Terminal;
use cairo_lang_syntax::node::{ast, ast::Member as MemberAst, TypedSyntaxNode};

use crate::utils::proc_macro_result_ext::ProcMacroResultExt;
use crate::utils::tokenize;

/// Represents a member of a struct.
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub name: String,
    pub ty: String,
    pub key: bool,
}

type ProcessStructFn = fn(&SimpleParserDatabase, TokenStream, &ast::ItemStruct) -> ProcMacroResult;

fn build_struct_without_attrs(
    db: &SimpleParserDatabase,
    struct_ast: &ast::ItemStruct,
) -> TokenStream {
    let visibility = tokenize(&struct_ast.visibility(db).as_syntax_node().get_text(db));
    let name = tokenize(&struct_ast.name(db).as_syntax_node().get_text(db));
    let generics = tokenize(&struct_ast.generic_params(db).as_syntax_node().get_text(db));
    let members = tokenize(
        &struct_ast
            .members(db)
            .elements(db)
            .iter()
            .map(|m| m.as_syntax_node().get_text(db))
            .collect::<Vec<_>>()
            .join(",\n"),
    );

    quote! {
        #visibility struct #name<#generics> {
            #members
        }
    }
}

pub(crate) fn process_struct(token_stream: TokenStream, func: ProcessStructFn) -> ProcMacroResult {
    let db = SimpleParserDatabase::default();
    let (root_node, _diagnostics) = db.parse_token_stream(&token_stream);

    for n in root_node.descendants(&db) {
        if n.kind(&db) == ItemStruct {
            let struct_ast = ast::ItemStruct::from_syntax_node(&db, n);
            let original_struct = build_struct_without_attrs(&db, &struct_ast);

            return func(&db, original_struct, &struct_ast);
        }
    }

    ProcMacroResult::empty()
}

pub(crate) fn parse_members(
    db: &SimpleParserDatabase,
    members: &[MemberAst],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Member> {
    let mut parsing_keys = true;

    members
        .iter()
        .map(|member_ast| {
            let is_key = member_ast.has_attr(db, "key");

            let member = Member {
                name: member_ast.name(db).text(db).to_string(),
                ty: member_ast
                    .type_clause(db)
                    .ty(db)
                    .as_syntax_node()
                    .get_text(db)
                    .trim()
                    .to_string(),
                key: is_key,
            };

            // Make sure all keys are before values in the model.
            if is_key && !parsing_keys {
                diagnostics.push(Diagnostic::error(
                    "Key members must be defined before non-key members.",
                ));
                // Don't return here, since we don't want to stop processing the members after the
                // first error to avoid diagnostics just because the field is
                // missing.
            }

            parsing_keys &= is_key;

            member
        })
        .collect::<Vec<_>>()
}

pub(crate) fn serialize_member_ty(member: &Member, with_self: bool) -> String {
    format!(
        "core::serde::Serde::serialize({}{}, ref serialized);\n",
        if with_self { "self." } else { "@" },
        member.name
    )
}
