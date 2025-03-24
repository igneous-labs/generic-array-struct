#![doc = include_str!("../README.md")]

use heck::ToShoutySnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::{Bracket, Paren, Semi},
    Data, DeriveInput, Expr, ExprPath, Field, Fields, FieldsUnnamed, GenericParam, Ident, Path,
    PathSegment, Type, TypeArray, TypePath, Visibility,
};

struct AttrArgs {
    array_field_vis: Visibility,
}

impl Parse for AttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                array_field_vis: Visibility::Inherited,
            });
        }
        Ok(Self {
            array_field_vis: input.parse()?,
        })
    }
}

/// The main attribute proc macro. See crate docs for usage.
#[proc_macro_attribute]
pub fn generic_array_struct(attr_arg: TokenStream, input: TokenStream) -> TokenStream {
    let AttrArgs { array_field_vis } = parse_macro_input!(attr_arg as AttrArgs);

    let mut input = parse_macro_input!(input as DeriveInput);

    let struct_vis = &input.vis;
    let struct_ident = &input.ident;
    let data_struct = match &mut input.data {
        Data::Struct(ds) => ds,
        _ => panic!("{MACRO_NAME} only works with structs"),
    };
    let fields = match &data_struct.fields {
        Fields::Named(f) => f,
        _ => panic!("{MACRO_NAME} only works with structs with named fields"),
    };
    let mut generic_iter = input.generics.params.iter();
    let generic = match generic_iter.next() {
        Some(GenericParam::Type(g)) => g,
        _ => panic!("{MACRO_NAME} {REQ_SINGLE_GENERIC_TYPE_PARAM_ERRMSG}"),
    };
    if generic_iter.next().is_some() {
        panic!("{MACRO_NAME} {REQ_SINGLE_GENERIC_TYPE_PARAM_ERRMSG}");
    }
    let generic_param_ident = &generic.ident;

    let (n_fields, fields_idx_consts, accessor_mutator_impls, const_with_impls) =
        fields.named.iter().enumerate().fold(
            (0usize, quote! {}, quote! {}, quote! {}),
            |(
                n_fields,
                mut fields_idx_consts,
                mut accessor_mutator_impls,
                mut const_with_impls,
            ),
             (i, field)| {
                let expect_same_generic = match &field.ty {
                    Type::Path(g) => g,
                    _ => panic!("{MACRO_NAME} {REQ_ALL_FIELDS_SAME_GENERIC_TYPE_ERRMSG}"),
                };
                if !expect_same_generic
                    .path
                    .get_ident()
                    .map(|id| id == generic_param_ident)
                    .unwrap_or(false)
                {
                    panic!("{MACRO_NAME} {REQ_ALL_FIELDS_SAME_GENERIC_TYPE_ERRMSG}")
                }

                let field_vis = &field.vis;
                // unwrap-safety: named field checked above
                let field_ident = field.ident.as_ref().unwrap();

                // pub const RGB_IDX_R: usize = 0;
                let idx_ident = field_idx_ident(struct_ident, field_ident);
                fields_idx_consts.extend(quote! {
                    #field_vis const #idx_ident: usize = #i;
                });

                // fn r(), r_mut(), set_r(), with_r()
                let ident_mut = format_ident!("{field_ident}_mut");
                let set_ident = format_ident!("set_{field_ident}");
                let with_ident = format_ident!("with_{field_ident}");
                // preserve attributes such as doc comments on getter method
                let field_attrs = &field.attrs;
                accessor_mutator_impls.extend(quote! {
                    #(#field_attrs)*
                    #[inline]
                    #field_vis const fn #field_ident(&self) -> &T {
                        &self.0[#idx_ident]
                    }

                    #[inline]
                    #field_vis fn #ident_mut(&mut self) -> &mut T {
                        &mut self.0[#idx_ident]
                    }

                    /// Returns the old field value
                    #[inline]
                    #field_vis fn #set_ident(&mut self, val: T) -> T {
                        core::mem::replace(&mut self.0[#idx_ident], val)
                    }

                    #[inline]
                    #field_vis fn #with_ident(mut self, val: T) -> Self {
                        self.0[#idx_ident] = val;
                        self
                    }
                });

                // fn const_with_r()
                let const_with_ident = format_ident!("const_with_{field_ident}");
                const_with_impls.extend(quote! {
                    #[inline]
                    #field_vis const fn #const_with_ident(mut self, val: T) -> Self {
                        self.0[#idx_ident] = val;
                        self
                    }
                });

                (
                    n_fields + 1,
                    fields_idx_consts,
                    accessor_mutator_impls,
                    const_with_impls,
                )
            },
        );

    let len_ident = array_len_ident(struct_ident);

    // finally, replace the struct defn with a single array field tuple struct
    data_struct.fields = Fields::Unnamed(FieldsUnnamed {
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
                    path: path_from_var(generic_param_ident.clone()),
                })),
                semi_token: Semi::default(),
                len: Expr::Path(ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: path_from_var(len_ident.clone()),
                }),
            }),
        })
        .collect(),
    });

    quote! {
        #input

        #struct_vis const #len_ident: usize = #n_fields;

        impl<T> #struct_ident<T> {
            #accessor_mutator_impls
        }

        impl<T: Copy> #struct_ident<T> {
            #const_with_impls
        }

        #fields_idx_consts
    }
    .into()
}

/// e.g. RGB_LEN
fn array_len_ident(struct_ident: &Ident) -> Ident {
    format_ident!("{}_LEN", struct_ident.to_string().to_shouty_snake_case())
}

/// e.g. RGB_IDX_R
fn field_idx_ident(struct_ident: &Ident, field_ident: &Ident) -> Ident {
    format_ident!(
        "{}_IDX_{}",
        struct_ident.to_string().to_shouty_snake_case(),
        field_ident.to_string().to_shouty_snake_case()
    )
}

/// A plain path with a single segment
/// e.g.
/// - `T` (as in generic type param)
/// - `RGB_LEN`
fn path_from_var(ident: Ident) -> Path {
    Path {
        leading_colon: None,
        segments: core::iter::once(PathSegment {
            ident,
            arguments: Default::default(),
        })
        .collect(),
    }
}

const MACRO_NAME: &str = "generic_array_struct";

const REQ_SINGLE_GENERIC_TYPE_PARAM_ERRMSG: &str =
    "only works with structs with a single generic type param";

const REQ_ALL_FIELDS_SAME_GENERIC_TYPE_ERRMSG: &str =
    "requires all fields to have the same generic type";
