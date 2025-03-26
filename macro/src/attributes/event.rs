use cairo_lang_macro::{
    quote, Diagnostic, ProcMacroResult, TokenStream,
};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::{ast, helpers::QueryAttrs, TypedSyntaxNode};

use crate::constants::{DOJO_INTROSPECT_DERIVE, DOJO_PACKED_DERIVE, EXPECTED_DERIVE_ATTR_NAMES};
use crate::helpers::{
    self, DiagnosticsExt, DojoChecker, DojoParser, DojoFormatter, DojoTokenizer, Member, ProcMacroResultExt
};

#[derive(Debug)]
pub struct DojoEvent {
    diagnostics: Vec<Diagnostic>,
}

impl DojoEvent {
    pub fn new() -> Self {
        Self {
            diagnostics: vec![],
        }
    }

    pub fn process(token_stream: TokenStream) -> ProcMacroResult {
        let db = SimpleParserDatabase::default();

        if let Some(struct_ast) = DojoParser::parse_and_find_struct(&db, &token_stream) {
            return DojoEvent::process_ast(&db, &struct_ast);
        }

        ProcMacroResult::fail(format!("'dojo::event' must be used on struct only."))
    }

    fn process_ast(db: &SimpleParserDatabase, struct_ast: &ast::ItemStruct) -> ProcMacroResult {
        let mut event = DojoEvent::new();

        let event_name = struct_ast
            .name(db)
            .as_syntax_node()
            .get_text(db)
            .trim()
            .to_string();

        if let Some(failure) = DojoChecker::is_name_valid("event", &event_name) {
            return failure;
        }

        let members = DojoParser::parse_members(
            db,
            &struct_ast.members(db).elements(db),
            &mut event.diagnostics,
        );

        let mut serialized_keys: Vec<String> = vec![];
        let mut serialized_values: Vec<String> = vec![];

        DojoEvent::serialize_keys_and_values(
            &members,
            &mut serialized_keys,
            &mut serialized_values,
        );

        if serialized_keys.is_empty() {
            event
                .diagnostics
                .push_error("Event must define at least one #[key] attribute".into());
        }

        if serialized_values.is_empty() {
            event
                .diagnostics
                .push_error("Event must define at least one member that is not a key".into());
        }

        let members_values = members
            .iter()
            .filter_map(|m| {
                if m.key {
                    None
                } else {
                    Some(DojoFormatter::get_member_declaration(&m.name, &m.ty))
                }
            })
            .collect::<Vec<_>>();

        let derive_attr_names = DojoParser::extract_derive_attr_names(
            db,
            &mut event.diagnostics,
            struct_ast.attributes(db).query_attr(db, "derive"),
        );

        let mut event_value_derive_attr_names = derive_attr_names
            .iter()
            .map(|d| d.to_string())
            .filter(|d| d != DOJO_INTROSPECT_DERIVE && d != DOJO_PACKED_DERIVE)
            .collect::<Vec<String>>();

        let mut missing_derive_attrs = vec![];

        // Ensures events always derive Introspect if not already derived,
        // and do not derive IntrospectPacked.
        if derive_attr_names.contains(&DOJO_PACKED_DERIVE.to_string()) {
            event.diagnostics.push_error(format!(
                "Deriving {DOJO_PACKED_DERIVE} on event is not allowed."
            ));
        }

        if !derive_attr_names.contains(&DOJO_INTROSPECT_DERIVE.to_string()) {
            missing_derive_attrs.push(DOJO_INTROSPECT_DERIVE.to_string());
        }

        // Ensures events always derive required traits.
        EXPECTED_DERIVE_ATTR_NAMES.iter().for_each(|expected_attr| {
            if !derive_attr_names.contains(&expected_attr.to_string()) {
                missing_derive_attrs.push(expected_attr.to_string());
                event_value_derive_attr_names.push(expected_attr.to_string());
            }
        });

        let unique_hash = helpers::compute_unique_hash(
            db,
            &event_name,
            false,
            &struct_ast.members(db).elements(db),
        )
        .to_string();

        let original_struct = DojoTokenizer::rebuild_original_struct(&db, &struct_ast);

        let event_code = DojoEvent::generate_event_code(
            &event_name,
            &members_values.join("\n"),
            &serialized_keys.join("\n"),
            &serialized_values.join("\n"),
            &event_value_derive_attr_names.join(", "),
            &unique_hash,
        );

        let missing_derive_attr = DojoTokenizer::tokenize(&missing_derive_attrs.join(", "));

        ProcMacroResult::finalize(
            quote! {
                // original struct with missing derive attributes
                #[derive(#missing_derive_attr)]
                #original_struct

                // model
                #event_code
            },
            event.diagnostics
        )
    }

    fn generate_event_code(
        type_name: &String,
        members_values: &String,
        serialized_keys: &String,
        serialized_values: &String,
        event_value_derive_attr_names: &String,
        unique_hash: &String,
    ) -> TokenStream {
        let content = format!(
        "// EventValue on it's own does nothing since events are always emitted and
// never read from the storage. However, it's required by the ABI to
// ensure that the event definition contains both keys and values easily distinguishable.
// Only derives strictly required traits.
#[derive({event_value_derive_attr_names})]
pub struct {type_name}Value {{
    {members_values}
}}

pub impl {type_name}Definition of dojo::event::EventDefinition<{type_name}> {{
    #[inline(always)]
    fn name() -> ByteArray {{
        \"{type_name}\"
    }}
}}

pub impl {type_name}ModelParser of dojo::model::model::ModelParser<{type_name}> {{
    fn serialize_keys(self: @{type_name}) -> Span<felt252> {{
        let mut serialized = core::array::ArrayTrait::new();
        {serialized_keys}
        core::array::ArrayTrait::span(@serialized)
    }}
    fn serialize_values(self: @{type_name}) -> Span<felt252> {{
        let mut serialized = core::array::ArrayTrait::new();
        {serialized_values}
        core::array::ArrayTrait::span(@serialized)
    }}
}}

pub impl {type_name}EventImpl = dojo::event::event::EventImpl<{type_name}>;

#[starknet::contract]
pub mod e_{type_name} {{
    use super::{type_name};
    use super::{type_name}Value;

    #[storage]
    struct Storage {{}}

    #[abi(embed_v0)]
    impl {type_name}__DeployedEventImpl = dojo::event::component::IDeployedEventImpl<ContractState, {type_name}>;

    #[abi(embed_v0)]
    impl {type_name}__StoredEventImpl = dojo::event::component::IStoredEventImpl<ContractState, {type_name}>;

     #[abi(embed_v0)]
    impl {type_name}__EventImpl = dojo::event::component::IEventImpl<ContractState, {type_name}>;

    #[abi(per_item)]
    #[generate_trait]
    impl {type_name}Impl of I{type_name} {{
        // Ensures the ABI contains the Event struct, since it's never used
        // by systems directly.
        #[external(v0)]
        fn ensure_abi(self: @ContractState, event: {type_name}) {{
            let _event = event;
        }}

        // Outputs EventValue to allow a simple diff from the ABI compared to the
        // event to retrieved the keys of an event.
        #[external(v0)]
        fn ensure_values(self: @ContractState, value: {type_name}Value) {{
            let _value = value;
        }}

        // Ensures the generated contract has a unique classhash, using
        // a hardcoded hash computed on event and member names.
        #[external(v0)]
        fn ensure_unique(self: @ContractState) {{
            let _hash = {unique_hash};
        }}
    }}
}}"
    );
        TokenStream::new(vec![DojoTokenizer::tokenize(&content)])
    }

    pub fn serialize_keys_and_values(
        members: &[Member],
        serialized_keys: &mut Vec<String>,
        serialized_values: &mut Vec<String>,
    ) {
        members.iter().for_each(|member| {
            if member.key {
                serialized_keys.push(DojoFormatter::serialize_member_ty(member, true));
            } else {
                serialized_values.push(DojoFormatter::serialize_member_ty(member, true));
            }
        });
    }
}
