use crate::common::Crs;
use crate::quadkey::check_quadkey_support;
use crate::tile::{BoundingBox, Coords, Tile};
use crate::tile_matrix_set::{ordered_axes_inverted, TileMatrix, TileMatrixSet};
use crate::tms_iterator::XyzIterator;
use crate::transform::{merc_tile_ul, Transform, Transformer};
use crate::{BoundingBox2D, CornerOfOrigin, OrderedAxes, TitleDescriptionKeywords};
use std::f64::consts::PI;
use std::num::{NonZeroU16, NonZeroU64};

/// Tile Matrix Set API.
#[derive(Debug)]
pub struct Tms {
    pub tms: TileMatrixSet,
    pub is_quadtree: bool,
    // CRS transformation attributes
    data_crs: Crs,
    geographic_crs: Crs, // default=WGS84_CRS
    to_geographic: Option<Transformer>,
    from_geographic: Option<Transformer>,
}

#[derive(thiserror::Error, Debug)]
pub enum TmsError {
    #[error("Invalid tile zoom identifier: `{0}`")]
    InvalidZoomId(String),
    #[error("Invalid strategy: `{0}`. Should be one of lower|upper|auto")]
    InvalidZoomLevelStrategy(String),
    #[error("Invalid zoom level: `{0}`")]
    InvalidZoom(u8),
    #[error("Point ({0}, {1}) is outside bounds {2:?}")]
    PointOutsideBounds(f64, f64, BoundingBox),
    #[error(transparent)]
    TransformationError(#[from] crate::transform::Error),
    #[error("Zero width or height")]
    NonZeroError,
    // #[error("Raised when math errors occur beyond ~85 degrees N or S")]
    // InvalidLatitudeError,
    // #[error("TileMatrix not found for level: {0} - Unable to construct tileMatrix for TMS with variable scale")]
    // InvalidZoomError(u8),
    // #[error("Raised when errors occur in parsing a function's tile arg(s)")]
    // TileArgParsingError,
    // #[error("Raised when a custom TileMatrixSet doesn't support quadkeys")]
    // NoQuadkeySupport,
    // #[error("Raised when errors occur in computing or parsing quad keys")]
    // QuadKeyError,
}

pub type Result<T> = std::result::Result<T, TmsError>;

impl Clone for Tms {
    // Custom impl because `Clone` is not implemented for `Proj`
    fn clone(&self) -> Tms {
        Tms::init(&self.tms).expect("Repeating initialization")
    }
}

impl Tms {
    /// Prepare transformations and check if TileMatrixSet supports quadkeys.
    pub(crate) fn init(data: &TileMatrixSet) -> Result<Self> {
        let is_quadtree = check_quadkey_support(&data.tile_matrices);
        let data_crs = data.crs.clone();
        let geographic_crs = Crs::default(); // data.get("_geographic_crs", WGS84_CRS)
        let to_geographic = Transformer::from_crs(&data_crs, &geographic_crs, true).ok();
        let from_geographic = Transformer::from_crs(&geographic_crs, &data_crs, true).ok();
        // if to_geographic.is_none()
        //     warnings.warn(
        //         "Could not create coordinate Transformer from input CRS to the given geographic CRS"
        //         "some methods might not be available.",
        //         UserWarning,
        let mut tms = data.clone();
        Self::sort_tile_matrices(&mut tms)?;
        // Check bounding box CRS (TODO: should we store it?)
        if let Some(bounding_box) = &tms.bounding_box {
            if let Some(crs) = &bounding_box.crs {
                if *crs != tms.crs {
                    let _transform = Transformer::from_crs(crs, &tms.crs, true)?;
                }
            }
        }
        Ok(Self {
            tms,
            is_quadtree,
            data_crs,
            geographic_crs,
            to_geographic,
            from_geographic,
        })
    }

    /// Sort matrices by identifier
    fn sort_tile_matrices(tms: &mut TileMatrixSet) -> Result<()> {
        // Check zoom identifier format
        for m in &tms.tile_matrices {
            m.id.parse::<u8>()
                .map_err(|_e| TmsError::InvalidZoomId(m.id.clone()))?;
        }
        tms.tile_matrices.sort_by(|a, b| {
            a.id.parse::<u8>()
                .unwrap()
                .cmp(&b.id.parse::<u8>().unwrap())
        });
        Ok(())
    }

    /// Iterate over matrices
    pub fn matrices(&self) -> &Vec<TileMatrix> {
        &self.tms.tile_matrices
    }

    /// Fetch CRS from epsg
    pub fn crs(&self) -> &Crs {
        &self.tms.crs
    }

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
    /// * `id` - Tile Matrix Set identifier (default is 'Custom')
    /// * `ordered_axes`
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
        id: &str,                      // = "Custom",
        ordered_axes: Option<OrderedAxes>,
        geographic_crs: &Crs, // = WGS84_CRS,
    ) -> Result<Self> {
        let matrix_scale = matrix_scale.unwrap_or(vec![1, 1]);
        let bbox = transformed_bbox(&extent, crs, extent_crs)?;
        let width = (bbox.right - bbox.left).abs();
        let height = (bbox.top - bbox.bottom).abs();
        let resolutions: Vec<f64> = (minzoom..=maxzoom)
            .map(|zoom| {
                f64::max(
                    width
                        / (tile_width as f64 * matrix_scale[0] as f64)
                        / 2_u64.pow(zoom as u32) as f64,
                    height
                        / (tile_height as f64 * matrix_scale[1] as f64)
                        / 2_u64.pow(zoom as u32) as f64,
                )
            })
            .collect();
        Self::custom_resolutions(
            extent,
            crs,
            tile_width,
            tile_height,
            extent_crs,
            resolutions,
            title,
            id,
            ordered_axes,
            geographic_crs,
        )
    }

    /// Construct a custom TileMatrixSet with given resolutions
    pub fn custom_resolutions(
        extent: Vec<f64>,
        crs: &Crs,
        tile_width: u16,
        tile_height: u16,
        extent_crs: Option<&Crs>,
        resolutions: Vec<f64>,
        title: &str,
        id: &str,
        ordered_axes: Option<OrderedAxes>,
        geographic_crs: &Crs,
    ) -> Result<Self> {
        let mut tms = TileMatrixSet {
            title_description_keywords: TitleDescriptionKeywords {
                title: Some(title.to_string()),
                description: None,
                keywords: None,
            },
            id: id.to_string(),
            uri: None,
            crs: crs.clone(),
            ordered_axes: ordered_axes.clone(),
            well_known_scale_set: None,
            bounding_box: None,
            tile_matrices: Vec::with_capacity(resolutions.len()),
        };

        let is_inverted = if let Some(ordered_axes) = &ordered_axes {
            ordered_axes_inverted(ordered_axes)
        } else {
            tms.crs_axis_inverted()
        };

        tms.bounding_box = Some(if is_inverted {
            BoundingBox2D {
                crs: Some(extent_crs.unwrap_or(crs).clone()),
                ordered_axes: ordered_axes.clone(),
                lower_left: [extent[1], extent[0]],
                upper_right: [extent[3], extent[2]],
            }
        } else {
            BoundingBox2D {
                crs: Some(extent_crs.unwrap_or(crs).clone()),
                ordered_axes: ordered_axes.clone(),
                lower_left: [extent[0], extent[1]],
                upper_right: [extent[2], extent[3]],
            }
        });

        let bbox = transformed_bbox(&extent, crs, extent_crs)?;

        let x_origin = if !is_inverted { bbox.left } else { bbox.top };
        let y_origin = if !is_inverted { bbox.top } else { bbox.left };
        let corner_of_origin = if !is_inverted {
            None
        } else {
            Some(CornerOfOrigin::BottomLeft)
        };

        let mpu = meters_per_unit(crs);
        for (zoom, res) in resolutions.iter().enumerate() {
            let unitheight = tile_height as f64 * res;
            let unitwidth = tile_width as f64 * res;
            let maxy = ((bbox.top - bbox.bottom - 0.01 * unitheight) / unitheight).ceil() as u64;
            let maxx = ((bbox.right - bbox.left - 0.01 * unitwidth) / unitwidth).ceil() as u64;
            tms.tile_matrices.push(TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: zoom.to_string(),
                scale_denominator: res * mpu as f64 / 0.00028,
                cell_size: *res,
                corner_of_origin: corner_of_origin.clone(),
                point_of_origin: [x_origin, y_origin],
                tile_width: NonZeroU16::new(tile_width).ok_or(TmsError::NonZeroError)?,
                tile_height: NonZeroU16::new(tile_height).ok_or(TmsError::NonZeroError)?,
                matrix_width: NonZeroU64::new(maxx).ok_or(TmsError::NonZeroError)?,
                matrix_height: NonZeroU64::new(maxy).ok_or(TmsError::NonZeroError)?,
                variable_matrix_widths: None,
            });
        }

        let mut tms = Tms::init(&tms)?;
        tms.geographic_crs = geographic_crs.clone();
        Ok(tms)
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
                id: (tile_matrix.id.parse::<u8>().unwrap() + 1).to_string(),
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
    ) -> Result<u8> {
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
                return Err(TmsError::InvalidZoomLevelStrategy(
                    zoom_level_strategy.to_string(),
                ));
            }
        }
        Ok(zoom_level)
    }

    /// Transform point(x,y) to geographic longitude and latitude.
    fn lnglat(&self, x: f64, y: f64, truncate: bool /* =False */) -> Result<Coords> {
        point_in_bbox(Coords::new(x, y), self.xy_bbox(), DEFAULT_BBOX_PREC)?;
        let (mut lng, mut lat) = self.to_geographic.transform(x, y)?;

        if truncate {
            (lng, lat) = self.truncate_lnglat(lng, lat)?;
        }

        Ok(Coords::new(lng, lat))
    }

    /// Transform geographic longitude and latitude coordinates to TMS CRS
    pub fn xy(&self, lng: f64, lat: f64) -> Result<Coords> {
        point_in_bbox(Coords::new(lng, lat), self.xy_bbox(), DEFAULT_BBOX_PREC)?;

        let (x, y) = self.from_geographic.transform(lng, lat)?;

        Ok(Coords::new(x, y))
    }

    /// Transform geographic longitude and latitude coordinates to TMS CRS. Truncate geographic coordinates to TMS geographic bbox.
    pub fn xy_truncated(&self, lng: f64, lat: f64) -> Result<Coords> {
        let (lng, lat) = self.truncate_lnglat(lng, lat)?;
        self.xy(lng, lat)
    }

    /// Truncate geographic coordinates to TMS geographic bbox.
    //
    // Adapted from <https://github.com/mapbox/mercantile/blob/master/mercantile/__init__.py>
    pub fn truncate_lnglat(&self, lng: f64, lat: f64) -> Result<(f64, f64)> {
        let mut lng = lng;
        let mut lat = lat;
        let bbox = self.bbox()?;
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

        Ok((lng, lat))
    }

    /// Get the tile containing a Point (in TMS CRS).
    ///
    /// # Arguments
    /// * `xcoord`, ycoord - A `X` and `Y` pair in TMS coordinate reference system.
    /// * `zoom` - The zoom level.
    pub fn xy_tile(&self, xcoord: f64, ycoord: f64, zoom: u8) -> Tile {
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
            ((xcoord - origin_x) / (res * u16::from(matrix.tile_width) as f64)).floor()
        } else {
            0.0
        };
        let ytile = if !ycoord.is_infinite() {
            ((origin_y - ycoord) / (res * u16::from(matrix.tile_height) as f64)).floor()
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

        Tile::new(xtile, ytile, zoom)
    }

    /// Get the tile for a given geographic longitude and latitude pair.
    ///
    /// # Arguments
    /// * `lng`, `lat` : A longitude and latitude pair in geographic coordinate reference system.
    /// * `zoom` : The zoom level.
    pub fn tile(&self, lng: f64, lat: f64, zoom: u8) -> Result<Tile> {
        let xy = self.xy(lng, lat)?;
        Ok(self.xy_tile(xy.x, xy.y, zoom))
    }

    /// Get the tile for a given geographic longitude and latitude pair. Truncate inputs to limits of TMS geographic bounds.
    ///
    /// # Arguments
    /// * `lng`, `lat` : A longitude and latitude pair in geographic coordinate reference system.
    /// * `zoom` : The zoom level.
    pub fn tile_truncated(&self, lng: f64, lat: f64, zoom: u8) -> Result<Tile> {
        let xy = self.xy_truncated(lng, lat)?;
        Ok(self.xy_tile(xy.x, xy.y, zoom))
    }

    /// Return the upper left coordinate of the tile in TMS coordinate reference system.
    ///
    /// # Arguments
    /// * `tile`: (x, y, z) tile coordinates or a Tile object we want the upper left coordinates of.
    pub fn xy_ul(&self, tile: &Tile) -> Coords {
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

        let xcoord = origin_x + t.x as f64 * res * u16::from(matrix.tile_width) as f64;
        let ycoord = origin_y - t.y as f64 * res * u16::from(matrix.tile_height) as f64;
        Coords::new(xcoord, ycoord)
    }

    /// Return the bounding box of the tile in TMS coordinate reference system.
    ///
    /// # Arguments
    /// * `tile`: Tile object we want the bounding box of.
    pub fn xy_bounds(&self, tile: &Tile) -> BoundingBox {
        let t = tile; // parse_tile_arg(tile);

        let top_left = self.xy_ul(&t);
        let bottom_right = self.xy_ul(&Tile::new(t.x + 1, t.y + 1, t.z));
        BoundingBox::new(top_left.x, bottom_right.y, bottom_right.x, top_left.y)
    }

    /// Return the upper left coordinates of the tile in geographic coordinate reference system.
    ///
    /// # Arguments
    /// * `tile` - (x, y, z) tile coordinates or a Tile object we want the upper left geographic coordinates of.
    pub fn ul(&self, tile: &Tile) -> Result<Coords> {
        let coords = if self.data_crs.as_srid() == 3857 && self.geographic_crs.as_srid() == 4326 {
            let (lon, lat) = merc_tile_ul(tile.x as u32, tile.y as u32, tile.z);
            Coords::new(lon, lat)
        } else {
            let xy = self.xy_ul(tile);
            self.lnglat(xy.x, xy.y, false)?
        };
        Ok(coords)
    }

    /// Return the bounding box of the tile in geographic coordinate reference system.
    ///
    /// # Arguments
    /// * `tile` - Tile object we want the bounding box of.
    pub fn bounds(&self, tile: &Tile) -> Result<BoundingBox> {
        let t = tile; // parse_tile_arg(tile);

        let top_left = self.ul(t)?;
        let bottom_right = self.ul(&Tile::new(t.x + 1, t.y + 1, t.z))?;
        Ok(BoundingBox::new(
            top_left.x,
            bottom_right.y,
            bottom_right.x,
            top_left.y,
        ))
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
                    // Verified in init function
                    let transform =
                        Transformer::from_crs(crs, &self.crs(), true).expect("Transformer");
                    let (left, bottom, right, top) = transform
                        .transform_bounds(*left, *bottom, *right, *top /* , Some(21) */)
                        .expect("Transformer");
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
            let top_left = self.xy_ul(&Tile::new(0, 0, zoom));
            let bottom_right = self.xy_ul(&Tile::new(
                u64::from(matrix.matrix_width),
                u64::from(matrix.matrix_height),
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
    pub fn bbox(&self) -> Result<BoundingBox> {
        let xy_bbox = self.xy_bbox();
        let bbox = self.to_geographic.transform_bounds(
            xy_bbox.left,
            xy_bbox.bottom,
            xy_bbox.right,
            xy_bbox.top,
        )?;
        Ok(BoundingBox::new(bbox.0, bbox.1, bbox.2, bbox.3))
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
    ) -> Result<impl Iterator<Item = Tile>> {
        let mut tiles: Vec<Tile> = Vec::new();
        let bbox = self.bbox()?;
        let bboxes = if west > east {
            vec![
                (bbox.left, south, east, north),
                (west, south, bbox.right, north),
            ]
        } else {
            vec![(west, south, east, north)]
        };
        let get_tile = if truncate {
            Tms::tile_truncated
        } else {
            Tms::tile
        };
        for bb in bboxes {
            let w = bb.0.max(bbox.left);
            let s = bb.1.max(bbox.bottom);
            let e = bb.2.min(bbox.right);
            let n = bb.3.min(bbox.top);
            for z in zooms {
                let ul_tile = get_tile(self, w + LL_EPSILON, n - LL_EPSILON, *z)?;
                let lr_tile = get_tile(self, e - LL_EPSILON, s + LL_EPSILON, *z)?;
                for i in ul_tile.x..=lr_tile.x {
                    for j in ul_tile.y..=lr_tile.y {
                        tiles.push(Tile::new(i, j, *z));
                    }
                }
            }
        }
        Ok(tiles.into_iter())
    }

    /// Get the tile limits overlapped by a geographic bounding box
    fn extent_limits(
        &self,
        extend: &BoundingBox,
        minzoom: u8,
        maxzoom: u8,
        truncate: bool, /* = False */
    ) -> Result<Vec<MinMax>> {
        if extend.left > extend.right || minzoom > maxzoom {
            return Ok(Vec::new()); // TODO: Handle extend over date line
        }
        let bbox = self.bbox()?;
        let get_tile = if truncate {
            Tms::tile_truncated
        } else {
            Tms::tile
        };
        let w = extend.left.max(bbox.left);
        let s = extend.bottom.max(bbox.bottom);
        let e = extend.right.min(bbox.right);
        let n = extend.top.min(bbox.top);
        let limits = (minzoom..=maxzoom)
            .map(|z| {
                let ul_tile = get_tile(self, w + LL_EPSILON, n - LL_EPSILON, z)?;
                let lr_tile = get_tile(self, e - LL_EPSILON, s + LL_EPSILON, z)?;
                Ok(MinMax {
                    x_min: ul_tile.x,
                    x_max: lr_tile.x,
                    y_min: ul_tile.y,
                    y_max: lr_tile.y,
                })
            })
            .collect::<Result<Vec<MinMax>>>()?;
        Ok(limits)
    }

    /// Get the tile limits overlapped by a bounding box in TMS CRS
    fn extent_limits_xy(&self, extend: &BoundingBox, minzoom: u8, maxzoom: u8) -> Vec<MinMax> {
        if extend.left > extend.right || minzoom > maxzoom {
            return Vec::new(); // TODO: Handle extend over date line
        }
        let bbox = self.xy_bbox();
        let w = extend.left.max(bbox.left);
        let s = extend.bottom.max(bbox.bottom);
        let e = extend.right.min(bbox.right);
        let n = extend.top.min(bbox.top);
        (minzoom..=maxzoom)
            .map(|z| {
                let res = self.resolution(&self.matrix(z)) / 10.0;
                let ul_tile = self.xy_tile(w + res, n - res, z);
                let lr_tile = self.xy_tile(e - res, s + res, z);
                MinMax {
                    x_min: ul_tile.x,
                    x_max: lr_tile.x,
                    y_min: ul_tile.y,
                    y_max: lr_tile.y,
                }
            })
            .collect()
    }

    /// Get iterator over all tiles overlapped by a geographic bounding box
    pub fn xyz_iterator_geographic(
        &self,
        extend: &BoundingBox,
        minzoom: u8,
        maxzoom: u8,
    ) -> Result<XyzIterator> {
        let limits = self.extent_limits(extend, minzoom, maxzoom, false)?;
        Ok(XyzIterator::new(minzoom, maxzoom, limits))
    }

    /// Get iterator over all tiles overlapped by a bounding box in TMS CRS
    pub fn xyz_iterator(&self, extend: &BoundingBox, minzoom: u8, maxzoom: u8) -> XyzIterator {
        let limits = self.extent_limits_xy(extend, minzoom, maxzoom);
        XyzIterator::new(minzoom, maxzoom, limits)
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
            x_max: u64::from(m.matrix_width).saturating_sub(1),
            y_min: 0,
            y_max: u64::from(m.matrix_height).saturating_sub(1),
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

        validx && validy
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
        for x in t.x.saturating_sub(1)..=t.x.saturating_add(1) {
            for y in t.y.saturating_sub(1)..=t.y.saturating_add(1) {
                if x == t.x && y == t.y {
                    continue;
                } else if x < extrema.x_min || y < extrema.y_min {
                    continue;
                } else if x > extrema.x_max || y > extrema.y_max {
                    continue;
                }

                tiles.push(Tile::new(x, y, t.z));
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
    pub fn parent(&self, tile: &Tile, zoom: Option<u8> /*  = None */) -> Result<Vec<Tile>> {
        if tile.z == self.minzoom() {
            return Ok(vec![]);
        }

        if let Some(zoom) = zoom {
            if tile.z <= zoom {
                // zoom must be less than that of the input tile
                return Err(TmsError::InvalidZoom(zoom));
            }
        } else if tile.z == 0 {
            return Err(TmsError::InvalidZoom(0));
        }

        let target_zoom = match zoom {
            Some(zoom) => zoom,
            None => tile.z - 1,
        };

        let res = self.resolution(&self.matrix(tile.z)) / 10.0;

        let bbox = self.xy_bounds(tile);
        let ul_tile = self.xy_tile(bbox.left + res, bbox.top - res, target_zoom);
        let lr_tile = self.xy_tile(bbox.right - res, bbox.bottom + res, target_zoom);

        let mut tiles = Vec::new();
        for i in ul_tile.x..=lr_tile.x {
            for j in ul_tile.y..=lr_tile.y {
                tiles.push(Tile::new(i, j, target_zoom));
            }
        }

        Ok(tiles)
    }

    /// Get the children of a tile
    ///
    /// The children are ordered: top-left, top-right, bottom-right, bottom-left.
    ///
    /// # Arguments
    /// * `tile` - instance of Tile
    /// * `zoom` - Determines the *zoom* level of the returned parent tile.
    ///     This defaults to one lower than the tile (the immediate parent).
    pub fn children(&self, tile: &Tile, zoom: Option<u8>) -> Result<Vec<Tile>> {
        let mut tiles = Vec::new();

        if let Some(zoom) = zoom {
            if tile.z > zoom {
                // zoom must be greater than that of the input tile
                return Err(TmsError::InvalidZoom(zoom));
            }
        }

        let target_zoom = match zoom {
            Some(z) => z,
            None => tile.z + 1,
        };

        let bbox = self.xy_bounds(tile);
        let res = self.resolution(&self.matrix(tile.z)) / 10.0;

        let ul_tile = self.xy_tile(bbox.left + res, bbox.top - res, target_zoom);
        let lr_tile = self.xy_tile(bbox.right - res, bbox.bottom + res, target_zoom);

        for i in ul_tile.x..=lr_tile.x {
            for j in ul_tile.y..=lr_tile.y {
                tiles.push(Tile::new(i, j, target_zoom));
            }
        }

        Ok(tiles)
    }
}

#[derive(Debug)]
pub(crate) struct MinMax {
    pub x_min: u64,
    pub x_max: u64,
    pub y_min: u64,
    pub y_max: u64,
}

impl TileMatrixSet {
    pub fn into_tms(&self) -> Result<Tms> {
        Tms::init(&self)
    }
}

fn transformed_bbox(extent: &Vec<f64>, crs: &Crs, extent_crs: Option<&Crs>) -> Result<BoundingBox> {
    let (mut left, mut bottom, mut right, mut top) = (extent[0], extent[1], extent[2], extent[3]);
    if let Some(extent_crs) = extent_crs {
        if extent_crs != crs {
            let transform = Transformer::from_crs(extent_crs, crs, true)?;
            (left, bottom, right, top) =
                transform.transform_bounds(left, bottom, right, top /* Some(21) */)?;
        }
    }
    Ok(BoundingBox::new(left, bottom, right, top))
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
        _ => panic!(
            "CRS {crs:?} with Unit Name `{}` is not supported",
            unit_name
        ),
    }
}

const LL_EPSILON: f64 = 1e-11;

pub const DEFAULT_BBOX_PREC: u8 = 5;

/// Check if a point is in a bounding box.
pub fn point_in_bbox(point: Coords, bbox: BoundingBox, precision: u8 /* = 5 */) -> Result<()> {
    fn round_to_prec(number: f64, precision: u8) -> f64 {
        let factor = 10.0_f64.powi(precision as i32);
        (number * factor).round() / factor
    }
    let inside = round_to_prec(point.x, precision) >= round_to_prec(bbox.left, precision)
        && round_to_prec(point.x, precision) <= round_to_prec(bbox.right, precision)
        && round_to_prec(point.y, precision) >= round_to_prec(bbox.bottom, precision)
        && round_to_prec(point.y, precision) <= round_to_prec(bbox.top, precision);
    if inside {
        Ok(())
    } else {
        Err(TmsError::PointOutsideBounds(point.x, point.y, bbox))
    }
}
