//! A library for map tile grid calculations
//!
//! ## Predefined grids
//!
//! ```rust
//! use tile_grid::{Extent, Grid};
//!
//! let grid = Grid::wgs84();
//! assert_eq!(
//!     grid.tile_extent(0, 0, 0),
//!     Extent {
//!         minx: -180.0,
//!         miny: -90.0,
//!         maxx: 0.0,
//!         maxy: 90.0,
//!     }
//! );
//! ```
//!
//! ## Grid iterators
//!
//! ```rust
//! use tile_grid::{Grid, GridIterator};
//!
//! let grid = Grid::web_mercator();
//! let tile_limits = grid.tile_limits(grid.extent.clone(), 0);
//! let griditer = GridIterator::new(0, 2, tile_limits);
//! for (z, x, y) in griditer {
//!     println!("Tile {}/{}/{}", z, x, y);
//! }
//! ```
//!
//!
//! ## Custom grids
//!
//! ```rust
//! use tile_grid::{Extent, Grid, Unit, Origin};
//!
//! let grid = Grid::new(
//!     256,
//!     256,
//!     Extent {
//!         minx: 2420000.0,
//!         miny: 1030000.0,
//!         maxx: 2900000.0,
//!         maxy: 1350000.0,
//!     },
//!     2056,
//!     Unit::Meters,
//!     vec![
//!         4000.0, 3750.0, 3500.0, 3250.0, 3000.0, 2750.0, 2500.0, 2250.0, 2000.0, 1750.0, 1500.0,
//!         1250.0, 1000.0, 750.0, 650.0, 500.0, 250.0, 100.0, 50.0, 20.0, 10.0, 5.0, 2.5, 2.0,
//!         1.5, 1.0, 0.5,
//!     ],
//!     Origin::TopLeft,
//! );
//! assert_eq!(
//!     grid.tile_extent(0, 0, 15),
//!     Extent {
//!         minx: 2420000.0,
//!         miny: 1222000.0,
//!         maxx: 2548000.0,
//!         maxy: 1350000.0,
//!     }
//! );
//! ```

mod grid;
mod grid_iterator;
#[cfg(test)]
mod grid_test;

pub use grid::{extent_wgs84_to_merc, Extent, ExtentInt, Grid, Origin, Unit};
pub use grid_iterator::GridIterator;

// OGC TMS
mod common;
mod errors;
mod quadkey;
mod registry;
mod tile;
mod tms;
mod transform;

pub use common::*;
pub use quadkey::*;
pub use registry::*;
pub use tile::*;
pub use tms::*;

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
