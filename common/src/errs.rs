use super::MACRO_NAME;

const REQ_SINGLE_GENERIC_TYPE_PARAM_ERRMSG: &str =
    "only works with structs with a single generic type param";

const REQ_ALL_FIELDS_SAME_GENERIC_TYPE_ERRMSG: &str =
    "requires all fields to have the same generic type";

const ONLY_WORKS_WITH_STRUCTS_ERRMSG: &str = "only works with structs";

const ONLY_WORKS_WITH_STRUCTS_WITH_NAMED_FIELDS_ERRMSG: &str =
    "only works with structs with named fields";

/// Panic with `err` error message
#[inline]
pub fn proc_macro_error(err: &str) -> ! {
    panic!("{MACRO_NAME} {err}")
}

#[inline]
pub fn panic_req_single_generic() -> ! {
    proc_macro_error(REQ_SINGLE_GENERIC_TYPE_PARAM_ERRMSG)
}

#[inline]
pub fn panic_req_all_fields_same_generic() -> ! {
    proc_macro_error(REQ_ALL_FIELDS_SAME_GENERIC_TYPE_ERRMSG)
}

#[inline]
pub(crate) fn panic_only_works_with_structs() -> ! {
    proc_macro_error(ONLY_WORKS_WITH_STRUCTS_ERRMSG)
}

#[inline]
pub(crate) fn panic_only_works_with_structs_with_named_fields() -> ! {
    proc_macro_error(ONLY_WORKS_WITH_STRUCTS_WITH_NAMED_FIELDS_ERRMSG)
}
