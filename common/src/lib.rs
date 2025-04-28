//! Common functionality used by subcrates

use quote::format_ident;
use syn::Ident;

pub fn with_ident(field_ident: &Ident) -> Ident {
    format_ident!("with_{field_ident}")
}
