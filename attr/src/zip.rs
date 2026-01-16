use crate::GenericArrayStructParams;

use quote::{format_ident, quote};

/// Outputs the token stream to append
pub(crate) fn impl_zip(params: &GenericArrayStructParams) -> proc_macro2::TokenStream {
    let n_fields = params.fields_named().named.iter().count();

    let struct_id = params.struct_ident();

    let [(us0, ts0, tus0), (us1, ts1, tus1)] = core::array::from_fn(|_| {
        (
            (0..n_fields).map(|i| format_ident!("u{i}")),
            (0..n_fields).map(|i| format_ident!("t{i}")),
            (0..n_fields).map(|i| {
                let t = format_ident!("t{i}");
                let u = format_ident!("u{i}");
                quote! { (#t, #u) }
            }),
        )
    });

    quote! {
        impl<T> #struct_id <T> {
            #[inline]
            pub fn zip<U>(self, #struct_id ([#(#us0),*]): #struct_id <U>) -> #struct_id <(T, U)> {
                let Self([#(#ts0),*]) = self;
                #struct_id ([#(#tus0),*])
            }
        }

        impl<T: Copy> #struct_id <T> {
            #[inline]
            pub const fn const_zip<U: Copy>(self, #struct_id ([#(#us1),*]): #struct_id <U>) -> #struct_id <(T, U)> {
                let Self([#(#ts1),*]) = self;
                #struct_id ([#(#tus1),*])
            }
        }
    }
}
