use crate::quadkey::{check_quadkey_support, meters_per_unit, point_in_bbox, DEFAULT_BBOX_PREC};
use crate::tile::{BoundingBox, Coords, Tile};
use crate::transform::{Transform, Transformer, CRS};
use serde::Deserialize;

type NumType = f64; // Union[float, int]
type BoundsType = [NumType; 2];
const LL_EPSILON: f64 = 1e-11;
const WGS84_CRS: &str = "EPSG:4326"; // CRS::from_epsg(4326);

/// A geographic or projected coordinate reference system.
type CRSType = String;

// impl CRSType {
//     /// validator for the type.
//     fn get_validators() -> impl Iterator<Item = fn()> {
//         yield Self::validate
//     }
//
//     /// Validate CRS.
//     pub fn validate(value: CRSType) -> CRS {
//         If input is a string we tranlate it to CRS
//         if !value.is_instance_of(CRS) {
//             return CRS.from_user_input(value)
//         }
//         value
//     }
//
//     /// Update default schema.
//     fn modify_schema(field_schema: ()) {
//         field_schema.update(
//             anyOf=[
//                 {"type": "pyproj.CRS"},
//                 {"type": "string", "minLength": 1, "maxLength": 65536},
//             ],
//             examples=[
//                 "CRS.from_epsg(4326)",
//                 "http://www.opengis.net/def/crs/EPSG/0/3978",
//                 "urn:ogc:def:crs:EPSG::2193",
//             ],
//         )
//     }
//
//     fn __repr__(&self) {
//         format!("CRS({})", super().__repr__())
//     }
// }

fn crs_to_uri(crs: &CRS) -> String {
    let authority = "EPSG".to_string();
    let code = 0; // FIXME: crs.proj_info().id.unwrap();
    let version = "0".to_string();
    // attempt to grab the authority, version, and code from the CRS
    // let authority_code = crs.to_authority(20);
    // if let Some(authority_code) = authority_code {
    //     let (authority, code) = authority_code;
    //     // if we have a version number in the authority, split it out
    //     if authority.contains("_") {
    //         let (authority, version) = authority.split("_");
    //     }
    // }
    format!(
        "http://www.opengis.net/def/crs/{}/{}{}",
        authority, version, code
    )
}

fn crs_axis_inverted(crs: &CRS) -> bool {
    // Check if CRS has inverted AXIS (lat,lon) instead of (lon,lat).
    let abbrev = ""; // FIXME: crs.axis_info[0].abbrev.to_uppercase();
    abbrev == "Y" || abbrev == "LAT" || abbrev == "N"
}

/// Bounding box
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TMSBoundingBox {
    #[serde(rename = "type")]
    pub type_: String,
    pub crs: CRSType,
    pub lower_corner: BoundsType,
    pub upper_corner: BoundsType,
}

impl TMSBoundingBox {
    pub fn new() -> TMSBoundingBox {
        Self {
            type_: "BoundingBoxType".to_string(),
            crs: CRSType::default(),
            lower_corner: BoundsType::default(),
            upper_corner: BoundsType::default(),
        }
    }
}

// class Config:
//     """Configure TMSBoundingBox."""
//     arbitrary_types_allowed = True
//     json_encoders = {CRS: lambda v: CRS_to_uri(v)}

impl Default for TMSBoundingBox {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TileMatrix {
    #[serde(rename = "type")]
    pub type_: String,
    pub title: Option<String>,
    #[serde(rename = "abstract")]
    pub abstract_: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub identifier: String,
    pub scale_denominator: f64,
    pub top_left_corner: BoundsType,
    pub tile_width: u16,
    pub tile_height: u16,
    pub matrix_width: u64,
    pub matrix_height: u64,
}

/// Tile matrix set
#[derive(Deserialize, Clone, Debug)]
#[serde_with::skip_serializing_none]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TileMatrixSetData {
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    #[serde(rename = "abstract")]
    pub abstract_: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub identifier: String,
    #[serde(rename = "supportedCRS")]
    pub supported_crs: CRSType,
    pub well_known_scale_set: Option<String>, // Url
    pub bounding_box: Option<TMSBoundingBox>,
    pub tile_matrix: Vec<TileMatrix>,
}

// https://github.com/georust/ogcapi/blob/main/ogcapi-types/src/tiles/tms.rs#L22
/// A definition of a tile matrix set following the Tile Matrix Set standard.
/// For tileset metadata, such a description (in `tileMatrixSet` property) is
/// only required for offline use, as an alternative to a link with a
// /// `http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme` relation type.
// #[serde_with::serde_as]
// #[serde_with::skip_serializing_none]
// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(rename_all = "camelCase")]
// pub struct TileMatrixSet {
//     #[serde(flatten)]
//     pub title_description_keywords: TitleDescriptionKeywords,
//     /// Tile matrix set identifier. Implementation of 'identifier'
//     pub id: String,
//     /// Reference to an official source for this TileMatrixSet
//     pub uri: Option<String>,
//     /// Coordinate Reference System (CRS)
//     #[serde_as(as = "DisplayFromStr")]
//     pub crs: Crs,
//     pub ordered_axes: Option<Vec<String>>,
//     /// Reference to a well-known scale set
//     pub well_known_scale_set: Option<String>,
//     /// Minimum bounding rectangle surrounding the tile matrix set, in the
//     /// supported CRS
//     pub bounding_box: Option<BoundingBox2D>,
//     /// Describes scale levels and its tile matrices
//     pub tile_matrices: Vec<TileMatrix>,
// }

#[derive(Debug)]
pub struct TileMatrixSet {
    pub tms: TileMatrixSetData,
    pub is_quadtree: bool,
    // CRS transformation attributes
    geographic_crs: CRSType, // default=WGS84_CRS
    to_geographic: Option<Transformer>,
    from_geographic: Option<Transformer>,
}

impl TileMatrixSet {
    /// Create PyProj transforms and check if TileMatrixSet supports quadkeys.
    pub fn init(data: TileMatrixSetData) -> Self {
        let is_quadtree = check_quadkey_support(&data.tile_matrix);
        let geographic_crs = WGS84_CRS.to_string(); // data.get("_geographic_crs", WGS84_CRS)
        let to_geographic = Some(Transformer::from_crs(
            &data.supported_crs,
            &geographic_crs,
            true,
        ));
        let from_geographic = Some(Transformer::from_crs(
            &geographic_crs,
            &data.supported_crs,
            true,
        ));
        // except ProjError:
        //     warnings.warn(
        //         "Could not create coordinate Transformer from input CRS to the given geographic CRS"
        //         "some methods might not be available.",
        //         UserWarning,
        Self {
            tms: data,
            is_quadtree,
            geographic_crs,
            to_geographic,
            from_geographic,
        }
    }

    // @validator("tileMatrix")
    /// Sort matrices by identifier
    fn sort_tile_matrices(v: Vec<TileMatrix>) -> Vec<TileMatrix> {
        let mut v = v.clone();
        v.sort_by(|a, b| {
            a.identifier
                .parse::<u8>()
                .unwrap()
                .cmp(&b.identifier.parse::<u8>().unwrap())
        });
        v
    }

    /// Iterate over matrices
    pub fn matrices(&self) -> &Vec<TileMatrix> {
        &self.tms.tile_matrix
    }

    // def __repr__(self):
    //     """Simplify default pydantic model repr."""
    //     return f"<TileMatrixSet title='{self.title}' identifier='{self.identifier}'>"

    /// Fetch CRS from epsg
    pub fn crs(&self) -> &CRS {
        &self.tms.supported_crs
    }

    // def rasterio_crs(self):
    //     """Return rasterio CRS."""
    //
    //     import rasterio
    //     from rasterio.env import GDALVersion
    //
    //     if GDALVersion.runtime().major < 3:
    //         return rasterio.crs.CRS.from_wkt(self.crs.to_wkt(WktVersion.WKT1_GDAL))
    //     else:
    //         return rasterio.crs.CRS.from_wkt(self.crs.to_wkt())

    /// TileMatrixSet minimum TileMatrix identifier
    pub fn minzoom(&self) -> u8 {
        self.tms.tile_matrix[0].identifier.parse::<u8>().unwrap()
    }
    /// TileMatrixSet maximum TileMatrix identifier
    pub fn maxzoom(&self) -> u8 {
        self.tms.tile_matrix[self.tms.tile_matrix.len() - 1]
            .identifier
            .parse::<u8>()
            .unwrap()
    }
    /// Check if CRS has inverted AXIS (lat,lon) instead of (lon,lat).
    fn invert_axis(&self) -> bool {
        crs_axis_inverted(self.crs())
    }

    /// Construct a custom TileMatrixSet.
    ///
    /// # Arguments
    /// * `crs` - Tile Matrix Set coordinate reference system
    /// * `extent` - Bounding box of the Tile Matrix Set, (left, bottom, right, top).
    /// * `tile_width` - Width of each tile of this tile matrix in pixels (default is 256).
    /// * `tile_height` - Height of each tile of this tile matrix in pixels (default is 256).
    /// * `matrix_scale` - Tiling schema coalescence coefficient (default: [1, 1] for EPSG:3857).
    ///     Should be set to [2, 1] for EPSG:4326.
    ///     see: <http:///docs.opengeospatial.org/is/17-083r2/17-083r2.html#14>
    /// * `extent_crs` - pyproj.CRS
    ///     Extent's coordinate reference system, as a pyproj CRS object.
    ///     (default: same as input crs)
    /// * `minzoom` - Tile Matrix Set minimum zoom level (default is 0).
    /// * `maxzoom` - Tile Matrix Set maximum zoom level (default is 24).
    /// * `title` - Tile Matrix Set title (default is 'Custom TileMatrixSet')
    /// * `identifier` - Tile Matrix Set identifier (default is 'Custom')
    /// * `geographic_crs` - Geographic (lat,lon) coordinate reference system (default is EPSG:4326)
    pub fn custom(
        extent: Vec<f64>,
        crs: &CRS,
        tile_width: u16,               // = 256,
        tile_height: u16,              // = 256,
        matrix_scale: Option<Vec<u8>>, // = None,
        extent_crs: Option<&CRS>,      // = None,
        minzoom: u8,                   // = 0,
        maxzoom: u8,                   // = 24,
        title: &str,                   // = "Custom TileMatrixSet",
        identifier: &str,              // = "Custom",
        geographic_crs: &CRS,          // = WGS84_CRS,
    ) -> Self {
        let matrix_scale = matrix_scale.unwrap_or(vec![1, 1]);

        let mut tms: TileMatrixSetData = TileMatrixSetData {
            type_: "TileMatrixSetType".to_string(),
            title: title.to_string(),
            abstract_: None,
            keywords: None,
            identifier: identifier.to_string(),
            supported_crs: crs.to_string(),
            well_known_scale_set: None,
            bounding_box: None,
            tile_matrix: Vec::new(),
        };

        let is_inverted = crs_axis_inverted(crs);

        tms.bounding_box = Some(if is_inverted {
            TMSBoundingBox {
                type_: "BoundingBoxType".to_string(),
                crs: extent_crs.unwrap_or(crs).to_string(),
                lower_corner: [extent[1], extent[0]],
                upper_corner: [extent[3], extent[2]],
            }
        } else {
            TMSBoundingBox {
                type_: "BoundingBoxType".to_string(),
                crs: extent_crs.unwrap_or(crs).to_string(),
                lower_corner: [extent[0], extent[1]],
                upper_corner: [extent[2], extent[3]],
            }
        });

        let left = extent[0];
        let bottom = extent[1];
        let right = extent[2];
        let top = extent[3];
        let bbox = if let Some(extent_crs) = extent_crs {
            let transform = Transformer::from_crs(extent_crs, crs, true);
            let (left, bottom, right, top) =
                transform.transform_bounds(left, bottom, right, top /* Some(21) */);
            BoundingBox::new(left, bottom, right, top)
        } else {
            BoundingBox::new(left, bottom, right, top)
        };

        let x_origin = if !is_inverted { bbox.left } else { bbox.top };
        let y_origin = if !is_inverted { bbox.top } else { bbox.left };

        let width = (bbox.right - bbox.left).abs();
        let height = (bbox.top - bbox.bottom).abs();
        let mpu = meters_per_unit(crs);
        for zoom in minzoom..maxzoom + 1 {
            let res = f64::max(
                width
                    / (tile_width as f64 * matrix_scale[0] as f64)
                    / 2_u64.pow(zoom as u32) as f64,
                height
                    / (tile_height as f64 * matrix_scale[1] as f64)
                    / 2_u64.pow(zoom as u32) as f64,
            );
            tms.tile_matrix.push(TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: zoom.to_string(),
                scale_denominator: res * mpu as f64 / 0.00028,
                top_left_corner: [x_origin, y_origin],
                tile_width,
                tile_height,
                matrix_width: matrix_scale[0] as u64 * 2_u64.pow(zoom as u32),
                matrix_height: matrix_scale[1] as u64 * 2_u64.pow(zoom as u32),
            });
        }

        let mut tms = TileMatrixSet::init(tms);
        tms.geographic_crs = geographic_crs.to_string();
        tms
    }

    /// Return the TileMatrix for a specific zoom.
    pub fn matrix(&self, zoom: u8) -> TileMatrix {
        for m in &self.tms.tile_matrix {
            if m.identifier == zoom.to_string() {
                return m.clone();
            }
        }

        let matrix_scale = (1..self.tms.tile_matrix.len())
            .map(|idx| {
                (self.tms.tile_matrix[idx].scale_denominator
                    / self.tms.tile_matrix[idx - 1].scale_denominator)
                    .round() // FIXME: round ndigits=2
            })
            .collect::<Vec<_>>();
        if matrix_scale.len() > 1 {
            // TODO: always true, error in morecantile?
            // panic!(
            //     "TileMatrix not found for level: {} - Unable to construct tileMatrix for TMS with variable scale",
            //     zoom
            // );
        }

        println!(
            "TileMatrix not found for level: {} - Creating values from TMS Scale.",
            zoom
        );

        let mut tile_matrix = self.tms.tile_matrix.last().unwrap().clone();
        let factor = 1.0 / matrix_scale[0];
        while tile_matrix.identifier != zoom.to_string() {
            tile_matrix = TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: (tile_matrix.identifier.parse::<i32>().unwrap() + 1).to_string(),
                scale_denominator: tile_matrix.scale_denominator / factor,
                top_left_corner: tile_matrix.top_left_corner,
                tile_width: tile_matrix.tile_width,
                tile_height: tile_matrix.tile_height,
                matrix_width: (tile_matrix.matrix_width as f64 * factor).round() as u64,
                matrix_height: (tile_matrix.matrix_height as f64 * factor).round() as u64,
            };
        }

        tile_matrix
    }

    /// Tile resolution for a TileMatrix.
    //
    // From note g in <http://docs.opengeospatial.org/is/17-083r2/17-083r2.html#table_2>:
    //   The pixel size of the tile can be obtained from the scaleDenominator
    //   by multiplying the later by 0.28 10-3 / metersPerUnit.
    fn resolution(&self, matrix: &TileMatrix) -> f64 {
        matrix.scale_denominator * 0.28e-3 / meters_per_unit(self.crs()) as f64
    }

    /// Get TMS zoom level corresponding to a specific resolution.
    ///
    /// # Arguments
    /// * `res` - Resolution in TMS unit.
    /// * `max_z` - Maximum zoom level (default is tms maxzoom).
    /// * `zoom_level_strategy` - Strategy to determine zoom level (same as in GDAL 3.2).
    ///         LOWER will select the zoom level immediately below the theoretical computed non-integral zoom level.
    ///         On the contrary, UPPER will select the immediately above zoom level.
    ///         Defaults to AUTO which selects the closest zoom level.
    ///         ref: <https://gdal.org/drivers/raster/cog.html#raster-cog>
    /// * `min_z` - Minimum zoom level (default is tms minzoom).
    ///
    /// # Returns:
    /// * TMS zoom for a given resolution.
    ///
    /// # Examples:
    /// `zoom_for_res(430.021)`
    pub fn zoom_for_res(
        &self,
        res: f64,
        max_z: Option<u8>,
        zoom_level_strategy: &str,
        min_z: Option<u8>,
    ) -> u8 {
        let max_z = max_z.unwrap_or(self.maxzoom());
        let min_z = min_z.unwrap_or(self.minzoom());
        let mut zoom_level = min_z;
        let mut matrix_res = 0.0; // TODO: check default
        for zoom_level in min_z..=max_z {
            matrix_res = self.resolution(&self.matrix(zoom_level));
            if res > matrix_res || (res - matrix_res).abs() / matrix_res <= 1e-8 {
                break;
            }
        }
        if zoom_level > 0 && (res - matrix_res).abs() / matrix_res > 1e-8 {
            if zoom_level_strategy.to_lowercase() == "lower" {
                zoom_level = u8::max(zoom_level - 1, min_z);
            } else if zoom_level_strategy.to_lowercase() == "upper" {
                zoom_level = u8::min(zoom_level, max_z);
            } else if zoom_level_strategy.to_lowercase() == "auto" {
                if (self.resolution(&self.matrix(u8::max(zoom_level - 1, min_z))) / res)
                    < (res / matrix_res)
                {
                    zoom_level = u8::max(zoom_level - 1, min_z);
                }
            } else {
                panic!(
                    "Invalid strategy: {}. Should be one of lower|upper|auto",
                    zoom_level_strategy
                );
            }
        }
        zoom_level
    }

    /// Transform point(x,y) to geographic longitude and latitude.
    fn lnglat(&self, x: f64, y: f64, truncate: bool /* =False */) -> Coords {
        // Default: truncate=False
        let inside = point_in_bbox(Coords::new(x, y), self.xy_bbox(), DEFAULT_BBOX_PREC);
        if !inside {
            println!(
                "Point ({x}, {y}) is outside TMS bounds {:?}.",
                self.xy_bbox(),
            );
        }

        let (mut lng, mut lat) = self.to_geographic.transform(x, y);

        if truncate {
            (lng, lat) = self.truncate_lnglat(lng, lat);
        }

        return Coords::new(lng, lat);
    }

    /// Transform geographic longitude and latitude coordinates to TMS CRS
    pub fn xy(&self, lng: f64, lat: f64, truncate: bool /* =False */) -> Coords {
        let mut lng = lng;
        let mut lat = lat;
        if truncate {
            (lng, lat) = self.truncate_lnglat(lng, lat);
        }

        let inside = point_in_bbox(Coords::new(lng, lat), self.xy_bbox(), DEFAULT_BBOX_PREC);
        if !inside {
            println!(
                "Point ({lng}, {lat}) is outside TMS bounds {:?}.",
                self.xy_bbox()
            );
        }

        let (x, y) = self.from_geographic.transform(lng, lat);

        return Coords::new(x, y);
    }

    /// Truncate geographic coordinates to TMS geographic bbox.
    //
    // Adapted from <https://github.com/mapbox/mercantile/blob/master/mercantile/__init__.py>
    pub fn truncate_lnglat(&self, lng: f64, lat: f64) -> (f64, f64) {
        let mut lng = lng;
        let mut lat = lat;
        let bbox = self.bbox();
        if lng > bbox.right {
            lng = bbox.right;
        } else if lng < bbox.left {
            lng = bbox.left;
        }

        if lat > bbox.top {
            lat = bbox.top;
        } else if lat < bbox.bottom {
            lat = bbox.bottom;
        }

        (lng, lat)
    }

    /// Get the tile containing a Point (in TMS CRS).
    ///
    /// # Arguments
    /// * `xcoord`, ycoord - A `X` and `Y` pair in TMS coordinate reference system.
    /// * `zoom` - The zoom level.
    fn tile_(&self, xcoord: f64, ycoord: f64, zoom: u8) -> Tile {
        let matrix = self.matrix(zoom);
        let res = self.resolution(&matrix);

        let origin_x: f64 = if self.invert_axis() {
            matrix.top_left_corner[1]
        } else {
            matrix.top_left_corner[0]
        };
        let origin_y = if self.invert_axis() {
            matrix.top_left_corner[0]
        } else {
            matrix.top_left_corner[1]
        };

        let xtile = if !xcoord.is_infinite() {
            ((xcoord - origin_x) / (res * matrix.tile_width as f64)).floor()
        } else {
            0.0
        };
        let ytile = if !ycoord.is_infinite() {
            ((origin_y - ycoord) / (res * matrix.tile_height as f64)).floor()
        } else {
            0.0
        };

        // avoid out-of-range tiles
        let xtile = if xtile < 0.0 { 0 } else { xtile as u64 };

        let ytile = if ytile < 0.0 { 0 } else { ytile as u64 };

        let xtile = if xtile > matrix.matrix_width {
            matrix.matrix_width
        } else {
            xtile
        };

        let ytile = if ytile > matrix.matrix_height {
            matrix.matrix_height
        } else {
            ytile
        };

        Tile::new(xtile as i64, ytile as i64, zoom)
    }

    /// Get the tile for a given geographic longitude and latitude pair.
    ///
    /// # Arguments
    /// * `lng`, `lat` : A longitude and latitude pair in geographic coordinate reference system.
    /// * `zoom` : The zoom level.
    /// * `truncate` : Whether or not to truncate inputs to limits of TMS geographic bounds.
    pub fn tile(&self, lng: f64, lat: f64, zoom: u8, truncate: bool /* =False */) -> Tile {
        let xy = self.xy(lng, lat, truncate);
        self.tile_(xy.x, xy.y, zoom)
    }

    /// Return the upper left coordinate of the tile in TMS coordinate reference system.
    ///
    /// # Arguments
    /// * `tile`: (x, y, z) tile coordinates or a Tile object we want the upper left coordinates of.
    pub fn ul_(&self, tile: &Tile) -> Coords {
        let t = tile;
        let matrix = self.matrix(t.z);
        let res = self.resolution(&matrix);

        let origin_x = if self.invert_axis() {
            matrix.top_left_corner[1]
        } else {
            matrix.top_left_corner[0]
        };
        let origin_y = if self.invert_axis() {
            matrix.top_left_corner[0]
        } else {
            matrix.top_left_corner[1]
        };

        let xcoord = origin_x + t.x as f64 * res * matrix.tile_width as f64;
        let ycoord = origin_y - t.y as f64 * res * matrix.tile_height as f64;
        Coords::new(xcoord, ycoord)
    }

    /// Return the bounding box of the tile in TMS coordinate reference system.
    ///
    /// # Arguments
    /// * `tile`: Tile object we want the bounding box of.
    pub fn xy_bounds(&self, tile: &Tile) -> BoundingBox {
        let t = tile; // parse_tile_arg(tile);

        let top_left = self.ul_(&t);
        let bottom_right = self.ul_(&Tile::new(t.x + 1, t.y + 1, t.z));
        BoundingBox::new(top_left.x, bottom_right.y, bottom_right.x, top_left.y)
    }

    /// Return the upper left coordinates of the tile in geographic coordinate reference system.
    ///
    /// # Arguments
    /// * `tile` - (x, y, z) tile coordinates or a Tile object we want the upper left geographic coordinates of.
    pub fn ul(&self, tile: &Tile) -> Coords {
        let t = tile; // parse_tile_arg(tile);

        let xy = self.ul_(&t);
        self.lnglat(xy.x, xy.y, false)
    }

    /// Return the bounding box of the tile in geographic coordinate reference system.
    ///
    /// # Arguments
    /// * `tile` - Tile object we want the bounding box of.
    pub fn bounds(&self, tile: &Tile) -> BoundingBox {
        let t = tile; // parse_tile_arg(tile);

        let top_left = self.ul(t);
        let bottom_right = self.ul(&Tile::new((t.x + 1) as i64, (t.y + 1) as i64, t.z));
        BoundingBox::new(top_left.x, bottom_right.y, bottom_right.x, top_left.y)
    }

    /// Return TMS bounding box in TileMatrixSet's CRS.
    pub fn xy_bbox(&self) -> BoundingBox {
        let (left, bottom, right, top) = if let Some(bounding_box) = &self.tms.bounding_box {
            let (left, bottom) = if self.invert_axis() {
                (&bounding_box.lower_corner[1], &bounding_box.lower_corner[0])
            } else {
                (&bounding_box.lower_corner[0], &bounding_box.lower_corner[1])
            };
            let (right, top) = if self.invert_axis() {
                (&bounding_box.upper_corner[1], &bounding_box.upper_corner[0])
            } else {
                (&bounding_box.upper_corner[0], &bounding_box.upper_corner[1])
            };
            if bounding_box.crs != self.crs() {
                let transform = Transformer::from_crs(&bounding_box.crs, &self.crs(), true);
                let (left, bottom, right, top) =
                    transform.transform_bounds(*left, *bottom, *right, *top /* , Some(21) */);
                (left, bottom, right, top)
            } else {
                (*left, *bottom, *right, *top)
            }
        } else {
            let zoom = self.minzoom();
            let matrix = self.matrix(zoom);
            let top_left = self.ul_(&Tile::new(0, 0, zoom));
            let bottom_right = self.ul_(&Tile::new(
                matrix.matrix_width as i64,
                matrix.matrix_height as i64,
                zoom,
            ));
            (top_left.x, bottom_right.y, bottom_right.x, top_left.y)
        };
        BoundingBox {
            left,
            bottom,
            right,
            top,
        }
    }

    /// Return TMS bounding box in geographic coordinate reference system.
    pub fn bbox(&self) -> BoundingBox {
        let xy_bbox = self.xy_bbox();
        let bbox = self.to_geographic.transform_bounds(
            xy_bbox.left,
            xy_bbox.bottom,
            xy_bbox.right,
            xy_bbox.top,
        );
        BoundingBox::new(bbox.0, bbox.1, bbox.2, bbox.3)
    }

    /// Check if a bounds intersects with the TMS bounds.
    pub fn intersect_tms(&self, bbox: &BoundingBox) -> bool {
        let tms_bounds = self.xy_bbox();
        bbox.left < tms_bounds.right
            && bbox.right > tms_bounds.left
            && bbox.top > tms_bounds.bottom
            && bbox.bottom < tms_bounds.top
    }

    /// Get the tiles overlapped by a geographic bounding box
    //
    // Original code from <https://github.com/mapbox/mercantile/blob/master/mercantile/__init__.py#L424>
    ///
    /// # Arguments
    /// * `west`, `south`, `east`, `north` - Bounding values in decimal degrees (geographic CRS).
    /// * `zooms` - One or more zoom levels.
    /// * `truncate` : Whether or not to truncate inputs to web mercator limits.
    ///
    /// # Notes
    /// A small epsilon is used on the south and east parameters so that this
    /// function yields exactly one tile when given the bounds of that same tile.
    pub fn tiles(
        &self,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
        zooms: &[u8],
        truncate: bool, /* = False */
    ) -> impl Iterator<Item = Tile> {
        let mut tiles: Vec<Tile> = Vec::new();
        let bbox = self.bbox();
        let bboxes = if west > east {
            vec![
                (bbox.left, south, east, north),
                (west, south, bbox.right, north),
            ]
        } else {
            vec![(west, south, east, north)]
        };
        for bb in bboxes {
            let w = bb.0.max(bbox.left);
            let s = bb.1.max(bbox.bottom);
            let e = bb.2.min(bbox.right);
            let n = bb.3.min(bbox.top);
            for z in zooms {
                let ul_tile = self.tile(w + LL_EPSILON, n - LL_EPSILON, *z, truncate);
                let lr_tile = self.tile(e - LL_EPSILON, s + LL_EPSILON, *z, truncate);
                for i in ul_tile.x..lr_tile.x + 1 {
                    for j in ul_tile.y..lr_tile.y + 1 {
                        tiles.push(Tile::new(i, j, *z));
                    }
                }
            }
        }
        tiles.into_iter()
    }

    // def feature(
    //     self,
    //     tile: Tile,
    //     fid: Optional[str] = None,
    //     props: Optional[Dict] = None,
    //     buffer: Optional[NumType] = None,
    //     precision: Optional[int] = None,
    //     projected: bool = False,
    // ) -> Dict:
    //     """
    //     Get the GeoJSON feature corresponding to a tile.
    //
    //     Originally from <https://github.com/mapbox/mercantile/blob/master/mercantile/__init__.py>
    //
    //     Parameters
    //     ----------
    //     tile : Tile or sequence of int
    //         May be be either an instance of Tile or 3 ints, X, Y, Z.
    //     fid : str, optional
    //         A feature id.
    //     props : dict, optional
    //         Optional extra feature properties.
    //     buffer : float, optional
    //         Optional buffer distance for the GeoJSON polygon.
    //     precision: float
    //         If >= 0, geometry coordinates will be rounded to this number of decimal,
    //         otherwise original coordinate values will be preserved (default).
    //     projected : bool, optional
    //         Return coordinates in TMS projection. Default is false.
    //
    //     Returns
    //     -------
    //     dict
    //
    //     """
    //     west, south, east, north = self.xy_bounds(tile)
    //
    //     if not projected:
    //         west, south, east, north = self._to_geographic.transform_bounds(
    //             west, south, east, north, densify_pts=21
    //         )
    //
    //     if buffer:
    //         west -= buffer
    //         south -= buffer
    //         east += buffer
    //         north += buffer
    //
    //     if precision and precision >= 0:
    //         west, south, east, north = (
    //             round(v, precision) for v in (west, south, east, north)
    //         )
    //
    //     bbox = [min(west, east), min(south, north), max(west, east), max(south, north)]
    //     geom = bbox_to_feature(west, south, east, north)
    //
    //     xyz = str(tile)
    //     feat: Dict[str, Any] = {
    //         "type": "Feature",
    //         "bbox": bbox,
    //         "id": xyz,
    //         "geometry": geom,
    //         "properties": {
    //             "title": f"XYZ tile {xyz}",
    //             "grid_name": self.identifier,
    //             "grid_crs": self.crs.to_string(),
    //         },
    //     }
    //
    //     if projected:
    //         warnings.warn(
    //             "CRS is no longer part of the GeoJSON specification."
    //             "Other projection than EPSG:4326 might not be supported.",
    //             UserWarning,
    //         )
    //         feat.update(
    //             {"crs": {"type": "EPSG", "properties": {"code": self.crs.to_epsg()}}}
    //         )
    //
    //     if props:
    //         feat["properties"].update(props)
    //
    //     if fid is not None:
    //         feat["id"] = fid
    //
    //     return feat

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

    /// Return TileMatrix Extrema.
    ///
    /// # Arguments
    /// * `zoom` - The zoom level.
    fn minmax(&self, zoom: u8) -> MinMax {
        let m = self.matrix(zoom);
        MinMax {
            x_min: 0,
            x_max: (m.matrix_width - 1) as i64,
            y_min: 0,
            y_max: (m.matrix_height - 1) as i64,
        }
    }

    /// Check if a tile is valid.
    pub fn is_valid(&self, tile: &Tile) -> bool {
        let t = tile; // parse_tile_arg(tile);

        if t.z < self.minzoom() {
            return false;
        }

        let extrema = self.minmax(t.z);
        let validx = extrema.x_min <= t.x && t.x <= extrema.x_max;
        let validy = extrema.y_min <= t.y && t.y <= extrema.y_max;

        return validx && validy;
    }

    /// The neighbors of a tile
    ///
    /// The neighbors function makes no guarantees regarding neighbor tile
    /// ordering.
    ///
    /// The neighbors function returns up to eight neighboring tiles, where
    /// tiles will be omitted when they are not valid.
    ///
    /// # Arguments
    /// * `tile` - instance of Tile
    pub fn neighbors(&self, tile: &Tile) -> Vec<Tile> {
        let t = tile; // parse_tile_arg(tile);
        let extrema = self.minmax(t.z);

        let mut tiles = Vec::new();
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 {
                    continue;
                } else if t.x + i < extrema.x_min || t.y + j < extrema.y_min {
                    continue;
                } else if t.x + i > extrema.x_max || t.y + j > extrema.y_max {
                    continue;
                }

                tiles.push(Tile::new(t.x + i, t.y + j, t.z));
            }
        }

        tiles
    }

    /// Get the parent of a tile
    ///
    /// The parent is the tile of one zoom level lower that contains the
    /// given "child" tile.
    ///
    /// # Arguments
    /// * `tile` - instance of Tile
    /// * `zoom` - Determines the *zoom* level of the returned parent tile.
    ///     This defaults to one lower than the tile (the immediate parent).
    pub fn parent(&self, tile: &Tile, zoom: Option<u8> /*  = None */) -> Vec<Tile> {
        let t = tile; // parse_tile_arg(tile);
        if t.z == self.minzoom() {
            return vec![];
        }

        if let Some(zoom) = zoom {
            if t.z <= zoom {
                panic!("zoom must be less than that of the input t");
            }
        }

        let target_zoom = match zoom {
            Some(zoom) => zoom,
            None => t.z - 1,
        };

        let res = self.resolution(&self.matrix(t.z)) / 10.0;

        let bbox = self.xy_bounds(t);
        let ul_tile = self.tile_(bbox.left + res, bbox.top - res, target_zoom);
        let lr_tile = self.tile_(bbox.right - res, bbox.bottom + res, target_zoom);

        let mut tiles = Vec::new();
        for i in ul_tile.x..lr_tile.x + 1 {
            for j in ul_tile.y..lr_tile.y + 1 {
                tiles.push(Tile::new(i, j, target_zoom));
            }
        }

        tiles
    }

    /// Get the children of a tile
    ///
    /// The children are ordered: top-left, top-right, bottom-right, bottom-left.
    ///
    /// # Arguments
    /// * `tile` - instance of Tile
    /// * `zoom` - Determines the *zoom* level of the returned parent tile.
    ///     This defaults to one lower than the tile (the immediate parent).
    pub fn children(&self, tile: &Tile, zoom: Option<u8>) -> Vec<Tile> {
        let t = tile; // parse_tile_arg(tile);
        let mut tiles = Vec::new();

        if let Some(zoom) = zoom {
            if t.z > zoom {
                panic!("zoom must be greater than that of the input tile");
            }
        }

        let target_zoom = match zoom {
            Some(z) => z,
            None => t.z + 1,
        };

        let bbox = self.xy_bounds(t);
        let res = self.resolution(&self.matrix(t.z)) / 10.0;

        let ul_tile = self.tile_(bbox.left + res, bbox.top - res, target_zoom);
        let lr_tile = self.tile_(bbox.right - res, bbox.bottom + res, target_zoom);

        for i in ul_tile.x..lr_tile.x + 1 {
            for j in ul_tile.y..lr_tile.y + 1 {
                tiles.push(Tile::new(i, j, target_zoom));
            }
        }

        tiles
    }
}

struct MinMax {
    x_min: i64,
    x_max: i64,
    y_min: i64,
    y_max: i64,
}
