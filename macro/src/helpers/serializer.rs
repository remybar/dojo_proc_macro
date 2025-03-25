use super::Member;

pub struct DojoSerializer {}

impl DojoSerializer {
    pub(crate) fn serialize_member_ty(member: &Member, with_self: bool) -> String {
        format!(
            "core::serde::Serde::serialize({}{}, ref serialized);\n",
            if with_self { "self." } else { "@" },
            member.name
        )
    }
}
