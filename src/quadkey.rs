use crate::{
    tile::{BoundingBox, Coords, Tile},
    tms::{TileMatrix, TileMatrixSetInst},
    Crs,
};
use serde_json::json;
use std::f64::consts::PI;

/// Parse the *tile arg of module functions
/// Copy from https://github.com/mapbox/mercantile/blob/master/mercantile/__init__.py
/// # Arguments
/// * `tile` - Tile or sequence of int
///            May be be either an instance of Tile or 3 ints, X, Y, Z.
pub(crate) fn parse_tile_arg(args: Vec<i64>) -> Tile {
    if args.len() == 1 {
        //args = args[0]
    }
    if args.len() == 3 {
        return Tile::new(args[0], args[1], args[2] as u8);
    } else {
        panic!(
            "the tile argument may have 1 or 3 values. Note that zoom is a keyword-only argument"
        )
    }
}

/// Coefficient to convert the coordinate reference system (CRS)
/// units into meters (metersPerUnit).
//
// See http://docs.ogc.org/is/17-083r4/17-083r4.html#6-1-1-1-%C2%A0-tile-matrix-in-a-two-dimensional-space
// From note g in <http://docs.opengeospatial.org/is/17-083r2/17-083r2.html#table_2>:
//     If the CRS uses meters as units of measure for the horizontal dimensions,
//     then metersPerUnit=1; if it has degrees, then metersPerUnit=2pa/360
//     (a is the Earth maximum radius of the ellipsoid).
pub fn meters_per_unit(crs: &Crs) -> f64 {
    const SEMI_MAJOR_METRE: f64 = 6378137.0; /* crs.ellipsoid.semi_major_metre */
    let unit_name = if crs.as_srid() == 4326 {
        "degree" // FIXME: crs.axis_info[0].unit_name;
    } else {
        "metre"
    };
    match unit_name {
        "metre" => 1.0,
        "degree" => 2.0 * PI * SEMI_MAJOR_METRE / 360.0,
        "foot" => 0.3048,
        "US survey foot" => 0.30480060960121924,
        _ => panic!("CRS {crs:?} with Unit Name `{}` is not supported, please fill an issue in developmentseed/morecantile", unit_name),
    }
}

/// Create a GeoJSON feature from a bbox.
pub fn bbox_to_feature(west: f64, south: f64, east: f64, north: f64) -> serde_json::Value {
    json!({
        "type": "Polygon",
        "coordinates": [
            [[west, south], [west, north], [east, north], [east, south], [west, south]]
        ],
    })
}

pub const DEFAULT_BBOX_PREC: u8 = 5;

/// Check if a point is in a bounding box.
pub fn point_in_bbox(point: Coords, bbox: BoundingBox, precision: u8 /* = 5 */) -> bool {
    fn round_to_prec(number: f64, precision: u8) -> f64 {
        let factor = 10.0_f64.powi(precision as i32);
        (number * factor).round() / factor
    }
    round_to_prec(point.x, precision) >= round_to_prec(bbox.left, precision)
        && round_to_prec(point.x, precision) <= round_to_prec(bbox.right, precision)
        && round_to_prec(point.y, precision) >= round_to_prec(bbox.bottom, precision)
        && round_to_prec(point.y, precision) <= round_to_prec(bbox.top, precision)
}

/// Check if a number is a power of 2
fn is_power_of_two(number: u64) -> bool {
    number & number.saturating_sub(1) == 0 && number != 0
}

/// Check if a Tile Matrix Set supports quadkeys
pub(crate) fn check_quadkey_support(tms: &Vec<TileMatrix>) -> bool {
    tms.iter().enumerate().take(tms.len() - 1).all(|(i, t)| {
        t.matrix_width == t.matrix_height
            && is_power_of_two(t.matrix_width.into())
            && (u64::from(t.matrix_width) * 2) == u64::from(tms[i + 1].matrix_width)
    })
}

impl TileMatrixSetInst {
    /// Get the quadkey of a tile
    ///
    /// # Arguments
    /// * `tile` : instance of Tile
    pub fn quadkey(&self, tile: Tile) -> String {
        if !self.is_quadtree {
            panic!("This Tile Matrix Set doesn't support 2 x 2 quadkeys.");
        }

        let t = tile;
        let mut qk = vec![];
        // for z in range(t.z, self.minzoom, -1)
        for z in (self.minzoom() + 1..=t.z).rev() {
            let mut digit = 0;
            let mask = 1 << (z - 1);
            if t.x & mask != 0 {
                digit += 1;
            }
            if t.y & mask != 0 {
                digit += 2;
            }
            qk.push(digit.to_string());
        }

        qk.join("")
    }

    /// Get the tile corresponding to a quadkey
    ///
    /// # Arguments
    /// * `qk` - A quadkey string.
    pub fn quadkey_to_tile(&self, qk: &str) -> Tile {
        if !self.is_quadtree {
            panic!("This Tile Matrix Set doesn't support 2 x 2 quadkeys.");
        }

        if qk.len() == 0 {
            return Tile::new(0, 0, 0);
        }

        let mut xtile = 0;
        let mut ytile = 0;
        let mut z = 0;
        for (i, digit) in qk.chars().rev().enumerate() {
            z = i as u8;
            let mask = 1 << i;
            if digit == '1' {
                xtile = xtile | mask;
            } else if digit == '2' {
                ytile = ytile | mask;
            } else if digit == '3' {
                xtile = xtile | mask;
                ytile = ytile | mask;
            } else if digit != '0' {
                panic!("Unexpected quadkey digit: {}", digit);
            }
        }

        Tile::new(xtile, ytile, z + 1)
    }
}
