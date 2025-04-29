#![doc = include_str!("../README.md")]

use std::iter::once;

use generic_array_struct_common::{
    idents::{array_len_ident, field_idx_ident},
    utils::path_from_ident,
    GenericArrayStructParams,
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    token::{Colon, Const},
    AngleBracketedGenericArguments, ConstParam, DeriveInput, Expr, ExprLit, GenericArgument,
    GenericParam, Generics, Ident, Lit, LitBool, Token, Type, TypeParam, TypePath,
};

/// The main attribute proc macro. See crate docs for usage.
#[proc_macro_derive(GenericArrayStructBuilder)]
pub fn generic_array_struct_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let params = GenericArrayStructParams(input);

    let n_fields = params.fields_named().named.iter().count();
    let generic_id = params.generic_ident();
    let struct_id = params.struct_ident();
    let builder_id = format_ident!("{}Builder", struct_id);

    let mut res = quote! {};
    let mut drop_impl = quote! {};
    params
        .fields_named()
        .named
        .iter()
        .enumerate()
        .for_each(|(i, field)| {
            let params = generic_params(generic_id, n_fields, Some(i));
            let [gen_args_false, gen_args_true] =
                [false, true].map(|hole| generic_args(generic_id, n_fields, Some((i, hole))));
            // unwrap-safety: named field checked above by params.fields_named()
            let field_id = field.ident.as_ref().unwrap();
            let idx_id = field_idx_ident(struct_id, field_id);
            // TOOD: this might need to be a const-expr instead
            let cgid_i = cgid(i);

            res.extend(quote! {
                impl #params #builder_id #gen_args_false {
                    #[inline]
                    pub fn with_x(
                        mut self,
                        val: T,
                    ) -> #builder_id #gen_args_true {
                        // use raw array indices instead of mut references to preserve const
                        self.0.0[#idx_id] = core::mem::MaybeUninit::new(val);
                        unsafe {
                            core::mem::transmute_copy::<_, _>(
                                &core::mem::ManuallyDrop::new(self)
                            )
                        }
                    }
                }
            });
            drop_impl.extend(quote! {
                if #cgid_i {
                    unsafe {
                        self.0.0[#idx_id].assume_init_drop();
                    }
                }
            });
        });

    let new_builder_id = format_ident!("New{builder_id}");
    let all_false_gen_args = generic_args_fill(generic_id, n_fields, false);
    let just_param = ident_to_gen_param(generic_id.clone());
    let len_id = array_len_ident(struct_id);
    let all_gen_params = generic_params(generic_id, n_fields, None);
    let all_gen_args = generic_args(generic_id, n_fields, None);

    res.extend(quote! {
        #[repr(transparent)]
        pub struct #builder_id #all_gen_params (#struct_id <core::mem::MaybeUninit<#generic_id>>);

        pub type #new_builder_id #just_param = #builder_id #all_false_gen_args;

        impl<T> #new_builder_id <T> {
            const _UNINIT: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();

            #[inline]
            pub const fn start() -> Self {
                Self(#struct_id([Self::_UNINIT; #len_id]))
            }
        }

        impl #all_gen_params Drop for #builder_id #all_gen_args {
            fn drop(&mut self) {
                #drop_impl
            }
        }
    });

    res.into()
}

/// e.g.
///
/// - `generic_args(T, 3, Some((1, true)))` generates:
///   `<T, S0, true, S2>`
/// - `generic_args(T, 3, None)` generates:
///    `<T, S0, S1, S2>`
fn generic_args(
    generic_ident: &Ident,
    n_fields: usize,
    hole: Option<(usize, bool)>,
) -> AngleBracketedGenericArguments {
    AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: Token![<](generic_ident.span()),
        args: once(ident_to_gen_arg(generic_ident.clone()))
            .chain((0..n_fields).map(|i| {
                let (idx, b) = match hole {
                    None => return ident_to_gen_arg(cgid(i)),
                    Some(h) => h,
                };
                if i == idx {
                    GenericArgument::Const(Expr::Lit(ExprLit {
                        attrs: Vec::new(),
                        lit: Lit::Bool(LitBool::new(b, generic_ident.span())),
                    }))
                } else {
                    ident_to_gen_arg(cgid(i))
                }
            }))
            .collect(),
        gt_token: Token![>](generic_ident.span()),
    }
}

/// e.g. `generic_params(T, 3, Some(1))` generates:
/// `<T, const S0: bool, const S2: bool>`
fn generic_params(generic_ident: &Ident, n_fields: usize, omit: Option<usize>) -> Generics {
    Generics {
        lt_token: Some(Token![<](generic_ident.span())),
        params: once(GenericParam::Type(TypeParam {
            attrs: Vec::new(),
            ident: generic_ident.clone(),
            colon_token: None,
            bounds: Punctuated::new(),
            eq_token: None,
            default: None,
        }))
        .chain((0..n_fields).filter_map(|i| {
            if omit == Some(i) {
                None
            } else {
                Some(GenericParam::Const(ConstParam {
                    attrs: Vec::new(),
                    const_token: Const(generic_ident.span()),
                    ident: cgid(i),
                    colon_token: Colon(generic_ident.span()),
                    ty: Type::Path(TypePath {
                        qself: None,
                        path: path_from_ident(format_ident!("bool")),
                    }),
                    eq_token: None,
                    default: None,
                }))
            }
        }))
        .collect(),
        gt_token: Some(Token![>](generic_ident.span())),
        where_clause: None,
    }
}

/// e.g. `generic_args_fill(T, 3, true)` generates:
/// `<T, true, true, true>`
fn generic_args_fill(
    generic_ident: &Ident,
    n_fields: usize,
    fill: bool,
) -> AngleBracketedGenericArguments {
    AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: Token![<](generic_ident.span()),
        args: once(ident_to_gen_arg(generic_ident.clone()))
            .chain((0..n_fields).map(|_i| {
                GenericArgument::Const(Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit: Lit::Bool(LitBool::new(fill, generic_ident.span())),
                }))
            }))
            .collect(),
        gt_token: Token![>](generic_ident.span()),
    }
}

fn ident_to_gen_param(generic_ident: Ident) -> GenericParam {
    GenericParam::Type(TypeParam {
        attrs: Vec::new(),
        ident: generic_ident,
        colon_token: None,
        bounds: Punctuated::new(),
        eq_token: None,
        default: None,
    })
}

fn ident_to_gen_arg(generic_ident: Ident) -> GenericArgument {
    GenericArgument::Type(Type::Path(TypePath {
        qself: None,
        path: path_from_ident(generic_ident),
    }))
}

/// const generic ident.
/// e.g. `S0` as in `const S0: bool`
fn cgid(idx: usize) -> syn::Ident {
    format_ident!("S{idx}")
}
