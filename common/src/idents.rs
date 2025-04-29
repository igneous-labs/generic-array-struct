use heck::ToShoutySnakeCase;
use quote::format_ident;
use syn::Ident;

/// e.g. `with_x`
#[inline]
pub fn with_ident(field_ident: &Ident) -> Ident {
    format_ident!("with_{field_ident}")
}

/// e.g. `set_x`
#[inline]
pub fn set_ident(field_ident: &Ident) -> Ident {
    format_ident!("set_{field_ident}")
}

/// e.g. `x_mut`
#[inline]
pub fn ident_mut(field_ident: &Ident) -> Ident {
    format_ident!("{field_ident}_mut")
}

/// e.g. `const_with_x`
#[inline]
pub fn const_with_ident(field_ident: &Ident) -> Ident {
    format_ident!("const_with_{field_ident}")
}

/// e.g. RGB_LEN
#[inline]
pub fn array_len_ident(struct_ident: &Ident) -> Ident {
    format_ident!("{}_LEN", struct_ident.to_string().to_shouty_snake_case())
}

/// e.g. RGB_IDX_R
#[inline]
pub fn field_idx_ident(struct_ident: &Ident, field_ident: &Ident) -> Ident {
    format_ident!(
        "{}_IDX_{}",
        struct_ident.to_string().to_shouty_snake_case(),
        field_ident.to_string().to_shouty_snake_case()
    )
}
