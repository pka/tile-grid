use crate::transform::{Error, Result, Transform};
use crate::Crs;
use std::f64::consts;

#[derive(Clone, Debug)]
pub struct BasicTransformer {
    from: Crs,
    to: Crs,
}

impl Transform for BasicTransformer {
    fn from_crs(from: &Crs, to: &Crs, _always_xy: bool) -> Result<Self> {
        match (from.as_srid(), to.as_srid()) {
            (4326, 3857) | (3857, 4326) | (4326, 4326) | (3395, 4326) | (4326, 3395) => {
                Ok(BasicTransformer {
                    from: from.clone(),
                    to: to.clone(),
                })
            }
            (_a, _b) => {
                return Err(Error::TransformationUnsupported(from.clone(), to.clone()));
            }
        }
    }
    fn transform(&self, x: f64, y: f64) -> Result<(f64, f64)> {
        if self.from.as_srid() == self.to.as_srid() {
            return Ok((x, y));
        }
        if self.from.as_srid() != 4326 || self.to.as_srid() != 3857 {
            return Err(Error::TransformationUnsupported(
                self.from.clone(),
                self.to.clone(),
            ));
        }
        Ok(lonlat_to_merc(x, y))
    }
    fn transform_bounds(
        &self,
        left: f64,
        bottom: f64,
        right: f64,
        top: f64,
    ) -> Result<(f64, f64, f64, f64)> {
        if self.from.as_srid() == self.to.as_srid() {
            return Ok((left, bottom, right, top));
        }
        if self.from.as_srid() != 4326 || self.to.as_srid() != 3857 {
            return Err(Error::TransformationUnsupported(
                self.from.clone(),
                self.to.clone(),
            ));
        }
        let (minx, miny) = lonlat_to_merc(left, top);
        let (maxx, maxy) = lonlat_to_merc(right, bottom);
        Ok((minx, miny, maxx, maxy))
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

/// Returns the upper left (lon, lat) of a tile
pub(crate) fn merc_tile_ul(xtile: u32, ytile: u32, zoom: u8) -> (f64, f64) {
    let n = (zoom as f64).exp2();
    let lon_deg = xtile as f64 / n * 360.0 - 180.0;
    let lat_rad = (consts::PI * (1.0 - 2.0 * ytile as f64 / n)).sinh().atan();
    let lat_deg = lat_rad.to_degrees();
    (lon_deg, lat_deg)
}
