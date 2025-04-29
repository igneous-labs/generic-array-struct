#![doc = include_str!("../README.md")]

use builder::impl_builder;
use generic_array_struct_common::{
    errs::panic_req_all_fields_same_generic,
    idents::{
        array_len_ident, const_with_ident, field_idx_ident, ident_mut, set_ident, with_ident,
    },
    utils::path_from_ident,
    GenericArrayStructParams,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::{Bracket, Paren, Semi},
    DeriveInput, Expr, ExprPath, Field, Fields, FieldsUnnamed, Ident, Type, TypeArray, TypePath,
    Visibility,
};

mod builder;

struct AttrArgs {
    array_field_vis: Visibility,
    should_gen_builder: bool,
}

impl Parse for AttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let should_gen_builder = if input.peek(Ident) {
            let id: Ident = input.parse()?;
            if id != "builder" {
                panic!("Expected token `builder`")
            } else {
                true
            }
        } else {
            false
        };

        if input.is_empty() {
            return Ok(Self {
                array_field_vis: Visibility::Inherited,
                should_gen_builder,
            });
        }

        let array_field_vis = input.parse()?;
        Ok(Self {
            array_field_vis,
            should_gen_builder,
        })
    }
}

/// The main attribute proc macro. See crate docs for usage.
#[proc_macro_attribute]
pub fn generic_array_struct(attr_arg: TokenStream, input: TokenStream) -> TokenStream {
    let AttrArgs {
        array_field_vis,
        should_gen_builder,
    } = parse_macro_input!(attr_arg as AttrArgs);

    let input = parse_macro_input!(input as DeriveInput);
    let mut params = GenericArrayStructParams(input);

    let mut fields_idx_consts = quote! {};
    let mut accessor_mutator_impls = quote! {};
    let mut const_with_impls = quote! {};
    let n_fields =
        params
            .fields_named()
            .named
            .iter()
            .enumerate()
            .fold(0usize, |n_fields, (i, field)| {
                let expect_same_generic = match &field.ty {
                    Type::Path(g) => g,
                    _ => panic_req_all_fields_same_generic(),
                };
                if !expect_same_generic
                    .path
                    .get_ident()
                    .map(|id| id == params.generic_ident())
                    .unwrap_or(false)
                {
                    panic_req_all_fields_same_generic();
                }

                let field_vis = &field.vis;
                // unwrap-safety: named field checked above
                let field_ident = field.ident.as_ref().unwrap();

                // pub const RGB_IDX_R: usize = 0;
                let idx_ident = field_idx_ident(params.struct_ident(), field_ident);
                fields_idx_consts.extend(quote! {
                    #field_vis const #idx_ident: usize = #i;
                });

                // fn r(), r_mut(), set_r(), with_r()
                let id_mut = ident_mut(field_ident);
                let set_id = set_ident(field_ident);
                let with_id = with_ident(field_ident);
                // preserve attributes such as doc comments on getter method
                let field_attrs = &field.attrs;
                accessor_mutator_impls.extend(quote! {
                    #(#field_attrs)*
                    #[inline]
                    #field_vis const fn #field_ident(&self) -> &T {
                        &self.0[#idx_ident]
                    }

                    #[inline]
                    #field_vis fn #id_mut(&mut self) -> &mut T {
                        &mut self.0[#idx_ident]
                    }

                    /// Returns the old field value
                    #[inline]
                    #field_vis fn #set_id(&mut self, val: T) -> T {
                        core::mem::replace(&mut self.0[#idx_ident], val)
                    }

                    #[inline]
                    #field_vis fn #with_id(mut self, val: T) -> Self {
                        self.0[#idx_ident] = val;
                        self
                    }
                });

                // fn const_with_r()
                let const_with_id = const_with_ident(field_ident);
                const_with_impls.extend(quote! {
                    #[inline]
                    #field_vis const fn #const_with_id(mut self, val: T) -> Self {
                        self.0[#idx_ident] = val;
                        self
                    }
                });

                n_fields + 1
            });

    let len_ident = array_len_ident(params.struct_ident());

    let struct_vis = params.struct_vis();
    let struct_ident = params.struct_ident();
    let mut res = quote! {
        #struct_vis const #len_ident: usize = #n_fields;

        impl<T> #struct_ident<T> {
            #accessor_mutator_impls
        }

        impl<T: Copy> #struct_ident<T> {
            #const_with_impls
        }

        #fields_idx_consts
    };

    if should_gen_builder {
        res.extend(impl_builder(&params));
    }

    // finally, replace the struct defn with a single array field tuple struct
    params.data_struct_mut().fields = Fields::Unnamed(FieldsUnnamed {
        paren_token: Paren::default(),
        unnamed: core::iter::once(Field {
            vis: array_field_vis,
            attrs: Vec::new(),
            mutability: syn::FieldMutability::None,
            ident: None,
            colon_token: None,
            ty: Type::Array(TypeArray {
                bracket_token: Bracket::default(),
                elem: Box::new(Type::Path(TypePath {
                    qself: None,
                    path: path_from_ident(params.generic_ident().clone()),
                })),
                semi_token: Semi::default(),
                len: Expr::Path(ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: path_from_ident(len_ident.clone()),
                }),
            }),
        })
        .collect(),
    });

    // extend with original input with modified struct defn
    let GenericArrayStructParams(input) = params;
    res.extend(quote! { #input });

    res.into()
}
