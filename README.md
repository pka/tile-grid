tile-grid
=========

[![Crates.io](https://img.shields.io/crates/v/tile-grid.svg?maxAge=2592000)](https://crates.io/crates/tile-grid)
[![Documentation](https://docs.rs/tile-grid/badge.svg)](https://docs.rs/tile-grid/)

tile-grid is a library for using OGC TileMatrixSets (TMS).

tile-grid follows the OGC Two Dimensional Tile Matrix Set specification found in https://docs.ogc.org/is/17-083r4/17-083r4.html

Note: Variable matrix width tile set are *not supported*.

Usage
-----

### Load tile set matrix and get bounds of a tile

```rust
use tile_grid::{tms, BoundingBox, Tile};

let tms = tms().lookup("WebMercatorQuad").unwrap();

// Get the bounds for tile Z=4, X=10, Y=10 in the input projection
let bounds = tms.xy_bounds(&Tile::new(10, 10, 4));
assert_eq!(
    bounds,
    BoundingBox::new(
        5009377.085697308,
        -7514065.628545959,
        7514065.628545959,
        -5009377.085697308
    )
);

// Get the bounds for tile Z=4, X=10, Y=10 in LatLon (WGS84)
let bounds = tms.bounds(&Tile::new(10, 10, 4)).unwrap();
assert_eq!(
    bounds,
    BoundingBox::new(45.0, -55.77657301866769, 67.5, -40.97989806962013)
);
```

### Find tile for lat/lon

```rust
use tile_grid::{tms, Tile};

let tms = tms().lookup("WebMercatorQuad").unwrap();

let tile = tms.tile(159.31, -42.0, 4).unwrap();
assert_eq!(tile, Tile::new(15, 10, 4));

// Or using coordinates in input CRS
let tile = tms.xy_tile(17734308.1, -5160979.4, 4);
assert_eq!(tile, Tile::new(15, 10, 4));
```

Credits
-------

* [Morecantile](https://github.com/developmentseed/morecantile) by Vincent Sarago et al.
* [ogcapi](https://github.com/georust/ogcapi) by Balthasar Teuscher


License
-------

tile-grid is released under the MIT License.
