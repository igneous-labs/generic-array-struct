use quote::{format_ident, quote};
use syn::{Attribute, Ident, Visibility};

use crate::GenericArrayStructParams;

/// Outputs the token stream to append
pub(crate) fn impl_destr(
    params: &GenericArrayStructParams,
    struct_vis: &Visibility,
) -> proc_macro2::TokenStream {
    let field_idents = params
        .fields_named()
        .named
        .iter()
        .map(|f| f.ident.as_ref().expect("all fields should be named"));
    let field_seq = quote! {
        #(#field_idents),*
    };

    let struct_id = params.struct_ident();
    let generic_ident = params.generic_ident();
    let destr_id = format_ident!("{struct_id}Destr");
    let og_fields = params.data_struct().fields.iter();
    let attrs = params.attrs().iter().filter(|a| is_attr_compat(a));

    quote! {
        #(#attrs)*
        #struct_vis struct #destr_id <#generic_ident> {
            #(#og_fields),*
        }

        impl<T> #struct_id <T> {
            #[inline]
            pub fn from_destr(#destr_id { #field_seq }: #destr_id <T>) -> Self {
                Self([ #field_seq ])
            }

            #[inline]
            pub fn into_destr(self) -> #destr_id <T> {
                let Self([ #field_seq ]) = self;
                #destr_id { #field_seq }
            }
        }

        impl<T: Copy> #struct_id <T> {
            #[inline]
            pub const fn const_from_destr(#destr_id { #field_seq }: #destr_id <T>) -> Self {
                Self([ #field_seq ])
            }

            #[inline]
            pub const fn const_into_destr(self) -> #destr_id <T> {
                let Self([ #field_seq ]) = self;
                #destr_id { #field_seq }
            }
        }


    }
}

fn is_attr_compat(attr: &Attribute) -> bool {
    // #[repr(transparent)] incompatible
    if attr
        .path()
        .segments
        .first()
        .is_some_and(|s| s.ident == "repr")
    {
        // unwrap-safety: failure means malformed #[repr(...)] attribute
        let chosen_repr: Ident = attr.parse_args().unwrap();
        if chosen_repr == "transparent" {
            return false;
        }
    }

    true
}
