//! A library for using OGC TileMatrixSets (TMS).

//! ## Load tile set matrix and get bounds of a tile
//!
//! ```rust
//! use tile_grid::{tms, BoundingBox, Tile};
//!
//! let tms= tms().lookup("WebMercatorQuad").unwrap();
//!
//! // Get the bounds for tile Z=4, X=10, Y=10 in the input projection
//! let bounds = tms.xy_bounds(&Tile::new(10, 10, 4));
//! assert_eq!(
//!     bounds,
//!     BoundingBox::new(
//!         5009377.085697308,
//!         -7514065.628545959,
//!         7514065.628545959,
//!         -5009377.085697308
//!     )
//! );
//!
//! // Get the bounds for tile Z=4, X=10, Y=10 in LatLon (WGS84)
//! let bounds = tms.bounds(&Tile::new(10, 10, 4)).unwrap();
//! assert_eq!(
//!     bounds,
//!     BoundingBox::new(45.0, -55.77657301866769, 67.5, -40.97989806962013)
//! );
//! ```
//!
//! ## Find tile for lat/lon
//!
//! ```rust
//! use tile_grid::{tms, Tile};
//!
//! let tms= tms().lookup("WebMercatorQuad").unwrap();
//!
//! let tile = tms.tile(159.31, -42.0, 4).unwrap();
//! assert_eq!(tile, Tile::new(15, 10, 4));
//!
//! // Or using coordinates in input CRS
//! let tile = tms.xytile(17734308.1, -5160979.4, 4);
//! assert_eq!(tile, Tile::new(15, 10, 4));
//! ```

mod common;
mod quadkey;
mod registry;
mod tile;
mod tile_matrix_set;
mod tileset;
mod tms;
mod tms_iterator;
mod transform;
mod wmts;

pub use common::*;
pub use registry::*;
pub use tile::*;
pub use tile_matrix_set::*;
pub use tileset::*;
pub use tms::*;
pub use tms_iterator::*;
pub use wmts::*;

use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;

/// A 2DPoint in the CRS indicated elsewere
type Point2D = [f64; 2];

/// Ordered list of names of the dimensions defined in the CRS
type OrderedAxes = [String; 2];

#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TitleDescriptionKeywords {
    /// Title of this resource entity, normally used for display to a human
    pub title: Option<String>,
    /// Brief narrative description of this resoource entity, normally available
    /// for display to a human
    pub description: Option<String>,
    /// Unordered list of one or more commonly used or formalized word(s) or
    /// phrase(s) used to describe this resource entity
    pub keywords: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct Query {
    pub collections: Option<String>,
}

/// Minimum bounding rectangle surrounding a 2D resource in the CRS indicated elsewere
#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoundingBox2D {
    pub lower_left: Point2D,
    pub upper_right: Point2D,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub crs: Option<Crs>,
    pub orderd_axes: Option<OrderedAxes>,
}
