use crate::transform::Transform;
use crate::Crs;
use proj::Proj;

pub type ProjTransformer = Proj;

impl Transform for Proj {
    fn from_crs(from: &Crs, to: &Crs, _always_xy: bool) -> Self {
        Proj::new_known_crs(&from.as_known_crs(), &to.as_known_crs(), None).unwrap()
    }
    fn transform(&self, x: f64, y: f64) -> (f64, f64) {
        let result = self.convert((x, y)).unwrap();
        result
    }
    fn transform_bounds(
        &self,
        left: f64,
        bottom: f64,
        right: f64,
        top: f64,
    ) -> (f64, f64, f64, f64) {
        let (left, bottom) = self.convert((left, bottom)).unwrap();
        let (right, top) = self.convert((right, top)).unwrap();
        (left, bottom, right, top)
    }
}

// impl Transform for Option<Proj> {
//     fn from_crs(from: &Crs, to: &Crs, _always_xy: bool) -> Self {
//         Proj::new_known_crs(&from.as_known_crs(), &to.as_known_crs(), None).ok()
//     }
// }
