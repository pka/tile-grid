//! A library for using OGC TileMatrixSets (TMS).

//! ## Load tile set matrix and get bounds of a tile
//!
//! ```rust
//! use tile_grid::{tms, BoundingBox, Xyz};
//!
//! let tms = tms().lookup("WebMercatorQuad").unwrap();
//!
//! // Get the bounds for tile Z=4, X=10, Y=10 in the input projection
//! let bounds = tms.xy_bounds(&Xyz::new(10, 10, 4));
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
//! let bounds = tms.bounds(&Xyz::new(10, 10, 4)).unwrap();
//! assert_eq!(
//!     bounds,
//!     BoundingBox::new(45.0, -55.77657301866769, 67.5, -40.97989806962013)
//! );
//! ```
//!
//! ## Find tile for lat/lon
//!
//! ```rust
//! # use tile_grid::{tms, Xyz};
//! # let tms = tms().lookup("WebMercatorQuad").unwrap();
//! let tile = tms.tile(159.31, -42.0, 4).unwrap();
//! assert_eq!(tile, Xyz::new(15, 10, 4));
//!
//! // Or using coordinates in input CRS
//! let tile = tms.xy_tile(17734308.1, -5160979.4, 4);
//! assert_eq!(tile, Xyz::new(15, 10, 4));
//! ```

mod hilbert;
mod quadkey;
mod registry;
mod tile;
mod tile_matrix_set;
mod tms;
mod tms_iterator;
mod transform;
mod wmts;

pub use registry::{RegistryError as Error, *};
pub use tile::*;
pub use tile_matrix_set::*;
pub use tms::*;
pub use tms_iterator::*;
pub use wmts::*;
