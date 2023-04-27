mod proj;
pub use crate::transform::proj::Transformer;

use crate::Crs;

pub trait Transform {
    fn from_crs(from: &Crs, to: &Crs, always_xy: bool /* =true */) -> Self;
    fn transform(&self, x: f64, y: f64) -> (f64, f64);
    fn transform_bounds(
        &self,
        left: f64,
        bottom: f64,
        right: f64,
        top: f64,
        // densify_pts=21,
    ) -> (f64, f64, f64, f64);
}
