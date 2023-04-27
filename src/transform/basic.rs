use crate::transform::Transform;
use crate::Crs;
use std::f64::consts;

#[derive(Debug)]
pub struct BasicTransformer;

impl Transform for BasicTransformer {
    fn from_crs(from: &Crs, to: &Crs, _always_xy: bool) -> Self {
        if from.as_srid() == 4326 && to.as_srid() == 3857 {
            BasicTransformer {}
        } else {
            panic!("BasicTransformer does only support transforming WGS84 to Web Mercator");
        }
    }
    fn transform(&self, x: f64, y: f64) -> (f64, f64) {
        lonlat_to_merc(x, y)
    }
    fn transform_bounds(
        &self,
        left: f64,
        bottom: f64,
        right: f64,
        top: f64,
    ) -> (f64, f64, f64, f64) {
        let (minx, miny) = lonlat_to_merc(left, top);
        let (maxx, maxy) = lonlat_to_merc(right, bottom);
        (minx, miny, maxx, maxy)
    }
}

/// Returns the Spherical Mercator (x, y) in meters
pub fn lonlat_to_merc(lon: f64, lat: f64) -> (f64, f64) {
    // from mod web_mercator in grid_test
    //lng, lat = truncate_lnglat(lng, lat)
    let x = 6378137.0 * lon.to_radians();
    let y = 6378137.0 * ((consts::PI * 0.25) + (0.5 * lat.to_radians())).tan().ln();
    (x, y)
}
