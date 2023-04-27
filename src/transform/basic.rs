use crate::transform::Transform;
use crate::Crs;
use std::f64::consts;

#[derive(Debug)]
pub struct BasicTransformer {
    from: Crs,
    to: Crs,
}

impl Transform for BasicTransformer {
    fn from_crs(from: &Crs, to: &Crs, _always_xy: bool) -> Self {
        match (from.as_srid(), to.as_srid()) {
            (4326, 3857) | (3857, 4326) | (4326, 4326) | (3395, 4326) | (4326, 3395) => {
                BasicTransformer {
                    from: from.clone(),
                    to: to.clone(),
                }
            }
            (a, b) => {
                panic!("BasicTransformer does only support transforming WGS84 to Web Mercator ({:?} -> {to:?}) - ({a}->{b}", from);
            }
        }
    }
    fn transform(&self, x: f64, y: f64) -> (f64, f64) {
        if self.from.as_srid() == self.to.as_srid() {
            return (x, y);
        }
        assert!(self.from.as_srid() == 4326 && self.to.as_srid() == 3857);
        lonlat_to_merc(x, y)
    }
    fn transform_bounds(
        &self,
        left: f64,
        bottom: f64,
        right: f64,
        top: f64,
    ) -> (f64, f64, f64, f64) {
        if self.from.as_srid() == self.to.as_srid() {
            return (left, bottom, right, top);
        }
        assert!(self.from.as_srid() == 4326 && self.to.as_srid() == 3857);
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
