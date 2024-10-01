use super::namespace::Namespace;

macro_rules! namespace {
    ($const_name:ident, $str_literal:expr) => {
        pub const $const_name: Namespace = Namespace::of($str_literal);
    }
}

namespace!(TAG_LIST, "tls");
namespace!(SUPER, "sup");
namespace!(EQUIVALENT, "eq");
namespace!(SUB, "sub");
//namespace!(SUPERTAGS_NAMESPACE, "sptgs");
//namespace!(EQUIVALENT_TAGS_NAMESPACE, "eqtgs");
//namespace!(SUBTAGS_NAMESPACE, "sbtgs");