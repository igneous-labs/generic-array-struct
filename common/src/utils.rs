use syn::{Ident, Path, PathSegment};

/// Convert an Ident into a plain path with a single segment
/// e.g.
/// - `T` (as in generic type param)
/// - `RGB_LEN`.
///
/// This is required because certain AST nodes require Path
/// instead of Ident e.g. the elem type of an array
#[inline]
pub fn path_from_ident(ident: Ident) -> Path {
    Path {
        leading_colon: None,
        segments: core::iter::once(PathSegment {
            ident,
            arguments: Default::default(),
        })
        .collect(),
    }
}
