use quote::quote;

use crate::GenericArrayStructParams;

/// Outputs the token stream to append
pub(crate) fn impl_trymap(params: &GenericArrayStructParams) -> proc_macro2::TokenStream {
    let struct_id = params.struct_ident();

    quote! {
        impl<T> #struct_id <T> {
            #[inline]
            pub fn try_map_opt<B, F>(
                self,
                mut f: F,
            ) -> Option<#struct_id <B>> where F: FnMut(T) -> Option<B> {
                let mut res: #struct_id <core::mem::MaybeUninit<B>>
                    = #struct_id (core::array::from_fn(|_| core::mem::MaybeUninit::uninit()));
                let written = self.0.into_iter().zip(res.0.iter_mut()).try_fold(
                    0usize,
                    |written, (val, rmut)| {
                        rmut.write(f(val).ok_or(written)?);
                        Ok(written + 1)
                    }
                );
                if let Err(written) = written {
                    res.0.iter_mut().take(written).for_each(
                        |mu| unsafe { mu.assume_init_drop() }
                    );
                    None
                } else {
                    Some(#struct_id(
                        unsafe {
                            core::mem::transmute_copy::<_, _>(
                                &core::mem::ManuallyDrop::new(res.0)
                            )
                        }
                    ))
                }
            }

            #[inline]
            pub fn try_map_res<B, E, F>(
                self,
                mut f: F,
            ) -> Result<#struct_id <B>, E> where F: FnMut(T) -> Result<B, E> {
                let mut res: #struct_id <core::mem::MaybeUninit<B>>
                    = #struct_id (core::array::from_fn(|_| core::mem::MaybeUninit::uninit()));
                let written = self.0.into_iter().zip(res.0.iter_mut()).try_fold(
                    0usize,
                    |written, (val, rmut)| {
                        rmut.write(f(val).map_err(|e| (e, written))?);
                        Ok(written + 1)
                    }
                );
                if let Err((e, written)) = written {
                    res.0.iter_mut().take(written).for_each(
                        |mu| unsafe { mu.assume_init_drop() }
                    );
                    Err(e)
                } else {
                    Ok(#struct_id (
                        unsafe {
                            core::mem::transmute_copy::<_, _>(
                                &core::mem::ManuallyDrop::new(res.0)
                            )
                        }
                    ))
                }
            }
        }
    }
}
