mod basic;
#[cfg(feature = "projtransform")]
mod proj;

use crate::Crs;
pub use basic::*;
use core::fmt::Debug;

#[cfg(feature = "projtransform")]
pub type Transformer = crate::transform::proj::ProjTransformer;
#[cfg(not(feature = "projtransform"))]
pub type Transformer = BasicTransformer;

/// Basic set of coordinate transformation operations
pub trait Transform: Sized + Debug {
    fn from_crs(from: &Crs, to: &Crs, always_xy: bool /* =true */) -> Result<Self>;
    fn transform(&self, x: f64, y: f64) -> Result<(f64, f64)>;
    fn transform_bounds(
        &self,
        left: f64,
        bottom: f64,
        right: f64,
        top: f64,
        // densify_pts=21,
    ) -> Result<(f64, f64, f64, f64)>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unsupported transformation from `{0:?}` to `{1:?}`")]
    TransformationUnsupported(Crs, Crs),
    #[error("{0}")]
    TransformationError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

// Handle optional Transformer as null transformation
impl Transform for Option<Transformer> {
    fn from_crs(from: &Crs, to: &Crs, always_xy: bool) -> Result<Self> {
        Ok(Some(Transformer::from_crs(from, to, always_xy)?))
    }
    fn transform(&self, x: f64, y: f64) -> Result<(f64, f64)> {
        if let Some(transform) = self {
            transform.transform(x, y)
        } else {
            Ok((x, y))
        }
    }
    fn transform_bounds(
        &self,
        left: f64,
        bottom: f64,
        right: f64,
        top: f64,
    ) -> Result<(f64, f64, f64, f64)> {
        if let Some(transform) = self {
            transform.transform_bounds(left, bottom, right, top)
        } else {
            Ok((left, bottom, right, top))
        }
    }
}
