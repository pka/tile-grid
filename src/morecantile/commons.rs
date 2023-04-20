/// A xmin,ymin,xmax,ymax coordinates tuple.
// Examples:
// >>> BoundingBox(-180.0, -90.0, 180.0, 90.0)
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    /// min horizontal coordinate.
    pub left: f64,
    /// min vertical coordinate.
    pub bottom: f64,
    /// max horizontal coordinate.
    pub right: f64,
    /// max vertical coordinate.
    pub top: f64,
}

impl BoundingBox {
    /// Create a new BoundingBox.
    pub fn new(left: f64, bottom: f64, right: f64, top: f64) -> Self {
        Self {
            left,
            bottom,
            right,
            top,
        }
    }
}

/// A x,y Coordinates pair.
// Examples:
// >>> Coords(-90.3, 10.5)
#[derive(Debug, Clone, PartialEq)]
pub struct Coords {
    /// horizontal coordinate input projection unit.
    pub x: f64,
    /// vertical coordinate input projection unit.
    pub y: f64,
}

impl Coords {
    /// Create a new Coords.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// TileMatrixSet X,Y,Z tile indices.
// Examples:
// >>> Tile(0, 0, 0)
#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    /// horizontal index.
    pub x: i64,
    /// verctical index.
    pub y: i64,
    /// zoom level.
    pub z: u8,
}

impl Tile {
    /// Create a new Tile.
    pub fn new(x: i64, y: i64, z: u8) -> Self {
        Self { x, y, z }
    }
}
