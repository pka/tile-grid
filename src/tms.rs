use crate::common::{Crs, Links};
use crate::quadkey::{check_quadkey_support, meters_per_unit, point_in_bbox, DEFAULT_BBOX_PREC};
use crate::tile::{BoundingBox, Coords, Tile};
use crate::transform::{Transform, Transformer};
use crate::{BoundingBox2D, Point2D, TitleDescriptionKeywords};
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use std::num::NonZeroU64;

const LL_EPSILON: f64 = 1e-11;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileMatrixSets {
    pub tile_matrix_sets: Vec<TileMatrixSetItem>,
}

/// A minimal tile matrix set element for use within a list of tile matrix
/// sets linking to a full definition.
#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TileMatrixSetItem {
    /// Optional local tile matrix set identifier, e.g. for use as unspecified
    /// `{tileMatrixSetId}` parameter. Implementation of 'identifier'
    pub id: Option<String>,
    /// Title of this tile matrix set, normally used for display to a human
    pub title: Option<String>,
    /// Reference to an official source for this tileMatrixSet
    pub uri: Option<String>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub crs: Option<Crs>,
    /// Links to related resources. A 'self' link to the tile matrix set definition is required.
    pub links: Links,
}

/// A definition of a tile matrix set following the Tile Matrix Set standard.
/// For tileset metadata, such a description (in `tileMatrixSet` property) is
/// only required for offline use, as an alternative to a link with a
/// `http://www.opengis.net/def/rel/ogc/1.0/tiling-scheme` relation type.
#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TileMatrixSet {
    #[serde(flatten)]
    pub title_description_keywords: TitleDescriptionKeywords,
    /// Tile matrix set identifier. Implementation of 'identifier'
    pub id: String,
    /// Reference to an official source for this TileMatrixSet
    pub uri: Option<String>,
    /// Coordinate Reference System (CRS)
    #[serde_as(as = "DisplayFromStr")]
    pub crs: Crs,
    pub ordered_axes: Option<Vec<String>>,
    /// Reference to a well-known scale set
    pub well_known_scale_set: Option<String>,
    /// Minimum bounding rectangle surrounding the tile matrix set, in the
    /// supported CRS
    pub bounding_box: Option<BoundingBox2D>,
    /// Describes scale levels and its tile matrices
    pub tile_matrices: Vec<TileMatrix>,
}

#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TileMatrix {
    #[serde(flatten)]
    pub title_description_keywords: TitleDescriptionKeywords,
    /// Identifier selecting one of the scales defined in the [TileMatrixSet]
    /// and representing the scaleDenominator the tile. Implementation of 'identifier'
    pub id: String,
    /// Scale denominator of this tile matrix
    pub scale_denominator: f64,
    /// Cell size of this tile matrix
    pub cell_size: f64,
    /// description": "The corner of the tile matrix (_topLeft_ or
    /// _bottomLeft_) used as the origin for numbering tile rows and columns.
    /// This corner is also a corner of the (0, 0) tile.
    #[serde(default)]
    pub corner_of_origin: CornerOfOrigin,
    /// Precise position in CRS coordinates of the corner of origin (e.g. the
    /// top-left corner) for this tile matrix. This position is also a corner
    /// of the (0, 0) tile. In previous version, this was 'topLeftCorner' and
    /// 'cornerOfOrigin' did not exist.
    pub point_of_origin: Point2D,
    /// Width of each tile of this tile matrix in pixels
    pub tile_width: NonZeroU64,
    /// Height of each tile of this tile matrix in pixels
    pub tile_height: NonZeroU64,
    /// Width of the matrix (number of tiles in width)
    pub matrix_width: NonZeroU64,
    /// Height of the matrix (number of tiles in height)
    pub matrix_height: NonZeroU64,
    /// Describes the rows that has variable matrix width
    pub variable_matrix_widths: Option<Vec<VariableMatrixWidth>>,
}

/// Variable Matrix Width data structure
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariableMatrixWidth {
    /// Number of tiles in width that coalesce in a single tile for these rows
    pub coalesc: NonZeroU64,
    /// First tile row where the coalescence factor applies for this tilematrix
    pub min_tile_row: u64,
    /// Last tile row where the coalescence factor applies for this tilematrix
    pub smax_tile_row: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum CornerOfOrigin {
    TopLeft,
    BottomLeft,
}

impl Default for CornerOfOrigin {
    fn default() -> Self {
        CornerOfOrigin::TopLeft
    }
}

#[cfg(test)]
mod test {
    use super::TileMatrixSet;

    #[test]
    fn parse_tms_example() {
        let content = std::fs::read_to_string("./data/WebMercatorQuad.json").unwrap();
        let tms: TileMatrixSet = serde_json::from_str(&content).unwrap();
        dbg!(&tms);
        println!("{}", serde_json::to_string_pretty(&tms).unwrap());
    }
}

impl TileMatrixSet {
    /// Check if CRS has inverted AXIS (lat,lon) instead of (lon,lat).
    fn crs_axis_inverted(&self) -> bool {
        if let Some(axes) = &self.ordered_axes {
            axes[0] == "Y" || axes[0] == "LAT" || axes[0] == "N"
        } else {
            false // TODO: Check CRS axis ordering
        }
    }
}

#[derive(Debug)]
pub struct TileMatrixSetInst<'a> {
    pub tms: &'a TileMatrixSet,
    pub is_quadtree: bool,
    // CRS transformation attributes
    geographic_crs: Crs, // default=WGS84_CRS
    to_geographic: Option<Transformer>,
    from_geographic: Option<Transformer>,
}

impl<'a> TileMatrixSetInst<'a> {
    /// Create PyProj transforms and check if TileMatrixSet supports quadkeys.
    pub fn init(data: &'a TileMatrixSet) -> Self {
        let is_quadtree = check_quadkey_support(&data.tile_matrices);
        let geographic_crs = Crs::default(); // data.get("_geographic_crs", WGS84_CRS)
        let to_geographic = Some(Transformer::from_crs(&data.crs, &geographic_crs, true));
        let from_geographic = Some(Transformer::from_crs(&geographic_crs, &data.crs, true));
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
            a.id.parse::<u8>()
                .unwrap()
                .cmp(&b.id.parse::<u8>().unwrap())
        });
        v
    }

    /// Iterate over matrices
    pub fn matrices(&self) -> &Vec<TileMatrix> {
        &self.tms.tile_matrices
    }

    // def __repr__(self):
    //     """Simplify default pydantic model repr."""
    //     return f"<TileMatrixSet title='{self.title}' identifier='{self.identifier}'>"

    /// Fetch CRS from epsg
    pub fn crs(&self) -> &Crs {
        &self.tms.crs
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
        self.tms.tile_matrices[0].id.parse::<u8>().unwrap()
    }
    /// TileMatrixSet maximum TileMatrix identifier
    pub fn maxzoom(&self) -> u8 {
        self.tms.tile_matrices[self.tms.tile_matrices.len() - 1]
            .id
            .parse::<u8>()
            .unwrap()
    }
    /// Check if CRS has inverted AXIS (lat,lon) instead of (lon,lat).
    fn invert_axis(&self) -> bool {
        self.tms.crs_axis_inverted()
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
        crs: &Crs,
        tile_width: u16,               // = 256,
        tile_height: u16,              // = 256,
        matrix_scale: Option<Vec<u8>>, // = None,
        extent_crs: Option<&Crs>,      // = None,
        minzoom: u8,                   // = 0,
        maxzoom: u8,                   // = 24,
        title: &str,                   // = "Custom TileMatrixSet",
        identifier: &str,              // = "Custom",
        geographic_crs: &Crs,          // = WGS84_CRS,
    ) -> Self {
        todo!()
        /*
        let matrix_scale = matrix_scale.unwrap_or(vec![1, 1]);

        let mut tms: TileMatrixSet = TileMatrixSet {
            title: title.to_string(),
            abstract_: None,
            keywords: None,
            identifier: identifier.to_string(),
            crs: crs.to_string(),
            well_known_scale_set: None,
            bounding_box: None,
            tile_matrices: Vec::new(),
        };

        let is_inverted = tms.crs_axis_inverted();

        tms.bounding_box = Some(if is_inverted {
            BoundingBox2D {
                crs: extent_crs.unwrap_or(crs).to_string(),
                lower_left: [extent[1], extent[0]],
                upper_right: [extent[3], extent[2]],
            }
        } else {
            BoundingBox2D {
                type_: "BoundingBoxType".to_string(),
                crs: extent_crs.unwrap_or(crs).to_string(),
                lower_left: [extent[0], extent[1]],
                upper_right: [extent[2], extent[3]],
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
                point_of_origin: [x_origin, y_origin],
                tile_width,
                tile_height,
                matrix_width: matrix_scale[0] as u64 * 2_u64.pow(zoom as u32),
                matrix_height: matrix_scale[1] as u64 * 2_u64.pow(zoom as u32),
            });
        }

        let mut tms = TileMatrixSet::init(tms);
        tms.geographic_crs = geographic_crs.to_string();
        tms
        */
    }

    /// Return the TileMatrix for a specific zoom.
    pub fn matrix(&self, zoom: u8) -> TileMatrix {
        for m in &self.tms.tile_matrices {
            if m.id == zoom.to_string() {
                return m.clone();
            }
        }

        let matrix_scale = (1..self.tms.tile_matrices.len())
            .map(|idx| {
                (self.tms.tile_matrices[idx].scale_denominator
                    / self.tms.tile_matrices[idx - 1].scale_denominator)
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

        let mut tile_matrix = self.tms.tile_matrices.last().unwrap().clone();
        let factor = 1.0 / matrix_scale[0];
        while tile_matrix.id != zoom.to_string() {
            tile_matrix = TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: (tile_matrix.id.parse::<i32>().unwrap() + 1).to_string(),
                scale_denominator: tile_matrix.scale_denominator / factor,
                cell_size: tile_matrix.cell_size, // FIXME
                corner_of_origin: tile_matrix.corner_of_origin,
                point_of_origin: tile_matrix.point_of_origin,
                tile_width: tile_matrix.tile_width,
                tile_height: tile_matrix.tile_height,
                matrix_width: NonZeroU64::new(
                    (u64::from(tile_matrix.matrix_width) as f64 * factor).round() as u64,
                )
                .unwrap(),
                matrix_height: NonZeroU64::new(
                    (u64::from(tile_matrix.matrix_height) as f64 * factor).round() as u64,
                )
                .unwrap(),
                variable_matrix_widths: None,
            }
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
            matrix.point_of_origin[1]
        } else {
            matrix.point_of_origin[0]
        };
        let origin_y = if self.invert_axis() {
            matrix.point_of_origin[0]
        } else {
            matrix.point_of_origin[1]
        };

        let xtile = if !xcoord.is_infinite() {
            ((xcoord - origin_x) / (res * u64::from(matrix.tile_width) as f64)).floor()
        } else {
            0.0
        };
        let ytile = if !ycoord.is_infinite() {
            ((origin_y - ycoord) / (res * u64::from(matrix.tile_height) as f64)).floor()
        } else {
            0.0
        };

        // avoid out-of-range tiles
        let xtile = if xtile < 0.0 { 0 } else { xtile as u64 };

        let ytile = if ytile < 0.0 { 0 } else { ytile as u64 };

        let xtile = if xtile > matrix.matrix_width.into() {
            matrix.matrix_width.into()
        } else {
            xtile
        };

        let ytile = if ytile > matrix.matrix_height.into() {
            matrix.matrix_height.into()
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
            matrix.point_of_origin[1]
        } else {
            matrix.point_of_origin[0]
        };
        let origin_y = if self.invert_axis() {
            matrix.point_of_origin[0]
        } else {
            matrix.point_of_origin[1]
        };

        let xcoord = origin_x + t.x as f64 * res * u64::from(matrix.tile_width) as f64;
        let ycoord = origin_y - t.y as f64 * res * u64::from(matrix.tile_height) as f64;
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
                (&bounding_box.lower_left[1], &bounding_box.lower_left[0])
            } else {
                (&bounding_box.lower_left[0], &bounding_box.lower_left[1])
            };
            let (right, top) = if self.invert_axis() {
                (&bounding_box.upper_right[1], &bounding_box.upper_right[0])
            } else {
                (&bounding_box.upper_right[0], &bounding_box.upper_right[1])
            };
            if let Some(crs) = &bounding_box.crs {
                if crs != self.crs() {
                    let transform = Transformer::from_crs(crs, &self.crs(), true);
                    let (left, bottom, right, top) = transform
                        .transform_bounds(*left, *bottom, *right, *top /* , Some(21) */);
                    (left, bottom, right, top)
                } else {
                    (*left, *bottom, *right, *top)
                }
            } else {
                (*left, *bottom, *right, *top)
            }
        } else {
            let zoom = self.minzoom();
            let matrix = self.matrix(zoom);
            let top_left = self.ul_(&Tile::new(0, 0, zoom));
            let bottom_right = self.ul_(&Tile::new(
                u64::from(matrix.matrix_width) as i64,
                u64::from(matrix.matrix_height) as i64,
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

    /// Return TileMatrix Extrema.
    ///
    /// # Arguments
    /// * `zoom` - The zoom level.
    fn minmax(&self, zoom: u8) -> MinMax {
        let m = self.matrix(zoom);
        MinMax {
            x_min: 0,
            x_max: (u64::from(m.matrix_width) - 1) as i64,
            y_min: 0,
            y_max: (u64::from(m.matrix_height) - 1) as i64,
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

impl<'a> From<&'a TileMatrixSet> for TileMatrixSetInst<'a> {
    fn from(tms: &'a TileMatrixSet) -> Self {
        TileMatrixSetInst::init(tms)
    }
}
