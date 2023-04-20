use proj::Proj;

pub type CRS = str;
// pub enum CRS {
//     EPSG(u16),
//     Authority(String, u16),
//     ProjStr(String),
//     ...
// }

pub type Transformer = Proj;

pub trait Transform {
    fn from_crs(from: &CRS, to: &CRS, always_xy: bool /* =true */) -> Self;
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

impl Transform for Proj {
    fn from_crs(from: &CRS, to: &CRS, _always_xy: bool) -> Self {
        Proj::new_known_crs(from, to, None).unwrap()
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

impl Transform for Option<Proj> {
    fn from_crs(from: &CRS, to: &CRS, _always_xy: bool) -> Self {
        Proj::new_known_crs(from, to, None).ok()
    }
    fn transform(&self, x: f64, y: f64) -> (f64, f64) {
        if let Some(transform) = self {
            transform.transform(x, y)
        } else {
            (x, y)
        }
    }
    fn transform_bounds(
        &self,
        left: f64,
        bottom: f64,
        right: f64,
        top: f64,
    ) -> (f64, f64, f64, f64) {
        if let Some(transform) = self {
            transform.transform_bounds(left, bottom, right, top)
        } else {
            (left, bottom, right, top)
        }
    }
}
