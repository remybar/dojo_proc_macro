use std::collections::HashSet;

use cairo_lang_macro::{quote, Diagnostics, ProcMacroResult, TextSpan, Token, TokenStream, TokenTree};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{ast, TypedSyntaxNode};
use dojo_types::naming;

use super::helpers::{self, Member};
use crate::constants::{DOJO_INTROSPECT_DERIVE, DOJO_PACKED_DERIVE, EXPECTED_DERIVE_ATTR_NAMES};
use crate::derives;
use crate::utils::tokenize;
use crate::utils::{proc_macro_result_ext::ProcMacroResultExt, DiagnosticsExt};


pub(crate) fn process(db: &SimpleParserDatabase, original_struct: TokenStream, struct_ast: &ast::ItemStruct) -> ProcMacroResult {
    let mut diagnostics = vec![];

    let model_type = struct_ast
        .name(db)
        .as_syntax_node()
        .get_text(db)
        .trim()
        .to_string();

    if !naming::is_name_valid(&model_type) {
        return ProcMacroResult::fail(format!(
            "The model name '{model_type}' can only contain characters (a-z/A-Z), \
            digits (0-9) and underscore (_)."
        ));
    }

    let mut values: Vec<Member> = vec![];
    let mut keys: Vec<Member> = vec![];
    let mut members_values: Vec<String> = vec![];
    let mut key_types: Vec<String> = vec![];
    let mut key_attrs: Vec<String> = vec![];

    let mut serialized_keys: Vec<String> = vec![];
    let mut serialized_values: Vec<String> = vec![];

    // The impl constraint for a model `MemberStore` must be defined for each member type.
    // To avoid double, we keep track of the processed types to skip the double impls.
    let mut model_member_store_impls_processed: HashSet<String> = HashSet::new();
    let mut model_member_store_impls: Vec<String> = vec![];

    let members =
        helpers::parse_members(db, &struct_ast.members(db).elements(db), &mut diagnostics);

    members.iter().for_each(|member| {
        if member.key {
            keys.push(member.clone());
            key_types.push(member.ty.clone());
            key_attrs.push(format!("*self.{}", member.name.clone()));
            serialized_keys.push(helpers::serialize_member_ty(member, true));
        } else {
            values.push(member.clone());
            serialized_values.push(helpers::serialize_member_ty(member, true));
            members_values.push(format!("pub {}: {},\n", member.name, member.ty));

            if !model_member_store_impls_processed.contains(&member.ty.to_string()) {
                model_member_store_impls.extend(vec![
                    format!(
                        "+dojo::model::storage::MemberModelStorage<S, {}, {}>",
                        model_type, member.ty
                    ),
                    format!(
                        "+dojo::model::storage::MemberModelStorage<S, {}Value, {}>",
                        model_type, member.ty
                    ),
                    format!(
                        "+dojo::model::members::MemberStore::<S, {}Value, {}>",
                        model_type, member.ty
                    ),
                ]);

                model_member_store_impls_processed.insert(member.ty.to_string());
            }
        }
    });

    if keys.is_empty() {
        diagnostics.push_error("Model must define at least one #[key] attribute".into());
    }

    if values.is_empty() {
        diagnostics.push_error("Model must define at least one member that is not a key".into());
    }

    if !diagnostics.is_empty() {
        return ProcMacroResult::fail_with_diagnostics(diagnostics);
    }

    let (keys_to_tuple, key_type) = if keys.len() > 1 {
        (
            format!("({})", key_attrs.join(", ")),
            format!("({})", key_types.join(", ")),
        )
    } else {
        (
            key_attrs.first().unwrap().to_string(),
            key_types.first().unwrap().to_string(),
        )
    };

    let derive_attr_names = derives::helpers::extract_derive_attr_names(
        db,
        &mut diagnostics,
        struct_ast.attributes(db).query_attr(db, "derive"),
    );

    // Build the list of derive attributes to set on "ModelValue" struct.
    let mut model_value_derive_attr_names = derive_attr_names
        .iter()
        .map(|d| d.to_string())
        .filter(|d| d != DOJO_INTROSPECT_DERIVE && d != DOJO_PACKED_DERIVE)
        .collect::<Vec<String>>();

    let mut missing_derive_attr_names = vec![];

    // If Introspect or IntrospectPacked derive attribute is not set for the model,
    // use Introspect by default.
    if !derive_attr_names.contains(&DOJO_INTROSPECT_DERIVE.to_string())
        && !derive_attr_names.contains(&DOJO_PACKED_DERIVE.to_string())
    {
        missing_derive_attr_names.push(DOJO_INTROSPECT_DERIVE.to_string());
    }

    // Add missing expected derive attributes for "Model" struct.
    EXPECTED_DERIVE_ATTR_NAMES.iter().for_each(|expected_attr| {
        let attr = expected_attr.to_string();

        if !derive_attr_names.contains(&attr) {
            missing_derive_attr_names.push(attr.clone());
            model_value_derive_attr_names.push(attr);
        }
    });
    
    let model_value_derive_attr_names = model_value_derive_attr_names.join(", ");

    let is_packed = derive_attr_names.contains(&DOJO_PACKED_DERIVE.to_string());

    let unique_hash = crate::utils::compute_unique_hash(
        db,
        &model_type,
        is_packed,
        &struct_ast.members(db).elements(db),
    )
    .to_string();

    let model_code = generate_model_code(
        &model_type,
        &model_value_derive_attr_names,
        &members_values.join(""),
        &key_type,
        &keys_to_tuple,
        &serialized_keys.join(""),
        &serialized_values.join(""),
        &unique_hash,
    );
  
    let missing_derive_attr = TokenTree::Ident(Token::new(missing_derive_attr_names.join(", "), TextSpan::call_site()));

    ProcMacroResult::new(quote! {
        // original struct with missing derive attributes
        #[derive(#missing_derive_attr)]
        #original_struct

        // model
        #model_code
    })
    .with_diagnostics(Diagnostics::new(diagnostics))
}

fn generate_model_code(
    model_type: &String,
    model_value_derive_attr_names: &String,
    members_values: &String,
    key_type: &String,
    keys_to_tuple: &String,
    serialized_keys: &String,
    serialized_values: &String,
    unique_hash: &String,
) -> TokenStream {
    let content = format!(
        "#[derive({model_value_derive_attr_names})]
pub struct {model_type}Value {{
    {members_values}
}}

type {model_type}KeyType = {key_type};

pub impl {model_type}KeyParser of dojo::model::model::KeyParser<{model_type}, {model_type}KeyType> {{
    #[inline(always)]
    fn parse_key(self: @{model_type}) -> {model_type}KeyType {{
        {keys_to_tuple}
    }}
}}

impl {model_type}ModelValueKey of dojo::model::model_value::ModelValueKey<{model_type}Value, {model_type}KeyType> {{
}}

// Impl to get the static definition of a model
pub mod m_{model_type}_definition {{
    use super::{model_type};
    pub impl {model_type}DefinitionImpl<T> of dojo::model::ModelDefinition<T>{{
        #[inline(always)]
        fn name() -> ByteArray {{
            \"{model_type}\"
        }}

        #[inline(always)]
        fn layout() -> dojo::meta::Layout {{
            dojo::meta::Introspect::<{model_type}>::layout()
        }}

        #[inline(always)]
        fn schema() -> dojo::meta::introspect::Struct {{
            if let dojo::meta::introspect::Ty::Struct(s) = dojo::meta::Introspect::<{model_type}>::ty() {{
                s
            }}
            else {{
                panic!(\"Model {model_type}: invalid schema.\")
            }}
        }}

        #[inline(always)]
        fn size() -> Option<usize> {{
            dojo::meta::Introspect::<{model_type}>::size()
        }}
    }}
}}

pub impl {model_type}Definition = m_{model_type}_definition::{model_type}DefinitionImpl<{model_type}>;
pub impl {model_type}ModelValueDefinition = m_{model_type}_definition::{model_type}DefinitionImpl<{model_type}Value>;

pub impl {model_type}ModelParser of dojo::model::model::ModelParser<{model_type}> {{
    fn serialize_keys(self: @{model_type}) -> Span<felt252> {{
        let mut serialized = core::array::ArrayTrait::new();
        {serialized_keys}
        core::array::ArrayTrait::span(@serialized)
    }}
    fn serialize_values(self: @{model_type}) -> Span<felt252> {{
        let mut serialized = core::array::ArrayTrait::new();
        {serialized_values}
        core::array::ArrayTrait::span(@serialized)
    }}
}}

pub impl {model_type}ModelValueParser of dojo::model::model_value::ModelValueParser<{model_type}Value> {{
    fn serialize_values(self: @{model_type}Value) -> Span<felt252> {{
        let mut serialized = core::array::ArrayTrait::new();
        {serialized_values}
        core::array::ArrayTrait::span(@serialized)
    }}
}}

pub impl {model_type}ModelImpl = dojo::model::model::ModelImpl<{model_type}>;
pub impl {model_type}ModelValueImpl = dojo::model::model_value::ModelValueImpl<{model_type}Value>;

#[starknet::contract]
pub mod m_{model_type} {{
    use super::{model_type};
    use super::{model_type}Value;

    #[storage]
    struct Storage {{}}

    #[abi(embed_v0)]
    impl {model_type}__DojoDeployedModelImpl = dojo::model::component::IDeployedModelImpl<ContractState, {model_type}>;

    #[abi(embed_v0)]
    impl {model_type}__DojoStoredModelImpl = dojo::model::component::IStoredModelImpl<ContractState, {model_type}>;

    #[abi(embed_v0)]
    impl {model_type}__DojoModelImpl = dojo::model::component::IModelImpl<ContractState, {model_type}>;

    #[abi(per_item)]
    #[generate_trait]
    impl {model_type}Impl of I{model_type} {{
        // Ensures the ABI contains the Model struct, even if never used
        // into as a system input.
        #[external(v0)]
        fn ensure_abi(self: @ContractState, model: {model_type}) {{
            let _model = model;
        }}

        // Outputs ModelValue to allow a simple diff from the ABI compared to the
        // model to retrieved the keys of a model.
        #[external(v0)]
        fn ensure_values(self: @ContractState, value: {model_type}Value) {{
            let _value = value;
        }}

        // Ensures the generated contract has a unique classhash, using
        // a hardcoded hash computed on model and member names.
        #[external(v0)]
        fn ensure_unique(self: @ContractState) {{
            let _hash = {unique_hash};
        }}
    }}
}}"
    );
    TokenStream::new(vec![tokenize(&content)])
}
