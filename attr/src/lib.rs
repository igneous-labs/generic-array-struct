use heck::ToShoutySnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input,
    token::{Bracket, Paren, Pub, Semi},
    Data, DeriveInput, Expr, ExprPath, Field, Fields, FieldsUnnamed, GenericParam, Ident, Path,
    PathSegment, Type, TypeArray, TypePath, Visibility,
};

#[proc_macro_attribute]
pub fn generic_array_struct(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let struct_vis = &input.vis;
    let struct_ident = &input.ident;
    let Data::Struct(data_struct) = &mut input.data else {
        panic!("{MACRO_NAME} only works with structs");
    };
    let Fields::Named(fields) = &data_struct.fields else {
        panic!("{MACRO_NAME} only works with structs with named fields");
    };
    let mut generic_iter = input.generics.params.iter();
    let Some(GenericParam::Type(generic)) = generic_iter.next() else {
        panic!("{MACRO_NAME} {REQ_SINGLE_GENERIC_TYPE_PARAM_ERRMSG}");
    };
    if generic_iter.next().is_some() {
        panic!("{MACRO_NAME} {REQ_SINGLE_GENERIC_TYPE_PARAM_ERRMSG}");
    }
    let generic_param_ident = &generic.ident;

    let (n_fields, fields_idx_consts, get_set_with_impls, const_with_impls) =
        fields.named.iter().enumerate().fold(
            (0usize, quote! {}, quote! {}, quote! {}),
            |(n_fields, mut fields_idx_consts, mut get_set_with_impls, mut const_with_impls),
             (i, field)| {
                if !field.attrs.is_empty() {
                    panic!("{MACRO_NAME} does not work with field attributes");
                }
                let Type::Path(expect_same_generic) = &field.ty else {
                    panic!("{MACRO_NAME} {REQ_ALL_FIELDS_SAME_GENERIC_TYPE_ERRMSG}")
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

                // fn r(), set_r(), with_r()
                let set_ident = format_ident!("set_{field_ident}");
                let with_ident = format_ident!("with_{field_ident}");
                get_set_with_impls.extend(quote! {
                    #[inline]
                    #field_vis const fn #field_ident(&self) -> &T {
                        &self.0[#idx_ident]
                    }

                    /// Returns the old field valuee
                    #[inline]
                    #field_vis const fn #set_ident(&mut self, val: T) -> T {
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
                    get_set_with_impls,
                    const_with_impls,
                )
            },
        );

    let len_ident = array_len_ident(struct_ident);

    // finally, replace the struct defn with a single array field tuple struct
    data_struct.fields = Fields::Unnamed(FieldsUnnamed {
        paren_token: Paren::default(),
        unnamed: core::iter::once(Field {
            attrs: Vec::new(),
            vis: Visibility::Public(Pub::default()),
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
            #get_set_with_impls
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
/// e.g. `T`, `RGB_LEN`
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
