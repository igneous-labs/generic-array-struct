//! Common functionality used by subcrates

use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, GenericParam, Ident, Visibility};

pub mod errs;
pub mod idents;
pub mod utils;

use errs::{
    panic_only_works_with_structs, panic_only_works_with_structs_with_named_fields,
    panic_req_single_generic,
};

pub const MACRO_NAME: &str = "generic_array_struct";

pub struct GenericArrayStructParams(pub DeriveInput);

/// Accessors
impl GenericArrayStructParams {
    #[inline]
    pub fn struct_vis(&self) -> &Visibility {
        &self.0.vis
    }

    #[inline]
    pub fn struct_ident(&self) -> &Ident {
        &self.0.ident
    }

    #[inline]
    pub fn generic_ident(&self) -> &Ident {
        let mut generic_iter = self.0.generics.params.iter();
        let generic = match generic_iter.next() {
            Some(GenericParam::Type(g)) => g,
            _ => panic_req_single_generic(),
        };
        if generic_iter.next().is_some() {
            panic_req_single_generic();
        }
        &generic.ident
    }

    #[inline]
    pub fn data_struct(&self) -> &DataStruct {
        match &self.0.data {
            Data::Struct(ds) => ds,
            _ => panic_only_works_with_structs(),
        }
    }

    #[inline]
    pub fn data_struct_mut(&mut self) -> &mut DataStruct {
        match &mut self.0.data {
            Data::Struct(ds) => ds,
            _ => panic_only_works_with_structs(),
        }
    }

    #[inline]
    pub fn fields_named(&self) -> &FieldsNamed {
        match &self.data_struct().fields {
            Fields::Named(f) => f,
            _ => panic_only_works_with_structs_with_named_fields(),
        }
    }
}
