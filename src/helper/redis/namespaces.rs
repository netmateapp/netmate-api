use super::namespace::Namespace;

macro_rules! namespace {
    ($const_name:ident, $str_literal:expr) => {
        pub const $const_name: Namespace = Namespace::of($str_literal);
    }
}

namespace!(SUPERTAGS_NAMESPACE, "sptgs");
namespace!(SUBTAGS_NAMESPACE, "sbtgs");