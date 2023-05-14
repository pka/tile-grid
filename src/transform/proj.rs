use crate::transform::{Error, Result, Transform};
use crate::Crs;
use proj::Proj;

pub type ProjTransformer = Proj;

impl Transform for Proj {
    fn from_crs(from: &Crs, to: &Crs, _always_xy: bool) -> Result<Self> {
        Proj::new_known_crs(&from.as_known_crs(), &to.as_known_crs(), None)
            .map_err(|_e| Error::TransformationUnsupported(from.clone(), to.clone()))
    }
    fn transform(&self, x: f64, y: f64) -> Result<(f64, f64)> {
        Ok(self.convert((x, y))?)
    }
    fn transform_bounds(
        &self,
        left: f64,
        bottom: f64,
        right: f64,
        top: f64,
    ) -> Result<(f64, f64, f64, f64)> {
        let (left, bottom) = self.convert((left, bottom))?;
        let (right, top) = self.convert((right, top))?;
        Ok((left, bottom, right, top))
    }
}

impl From<proj::ProjError> for Error {
    fn from(err: proj::ProjError) -> Self {
        Error::TransformationError(err.to_string())
    }
}

// impl Transform for Option<Proj> {
//     fn from_crs(from: &Crs, to: &Crs, _always_xy: bool) -> Self {
//         Proj::new_known_crs(&from.as_known_crs(), &to.as_known_crs(), None).ok()
//     }
// }
