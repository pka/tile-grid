## 0.6.0

* Use structs from ogcapi-types
* Make tile resolution function public
* Fix iterator panic when z_min >= z_max - z_min

## 0.5.2

* Deserialize Crs in TileSet structs with DisplayFromStr

## 0.5.1

* Make tileset structs and enums public

## 0.5.0

* Rename `Tile` to `Xyz`
* Only return errors for unsupported transformations when
  methods requiring transformations are called
* `matrix` method returns `AsRef<TileMatrix>` instead of cloned matrix
* Fix `zoom_for_res` and pass enum for strategy
* Construct TMS from custom resolutions
* Impl Clone for `Tms` and `TileMatrixSets`
* Change `tile_width`/`tile_height` from u64 to u16

## 0.4.0

* New implementation based on OGC TileMatrixSets 2.0

## 0.3.0

* Rename `extent_to_merc` to `extent_wgs84_to_merc`

## 0.2.2

* Export function `lonlat_to_merc` to project WGS84 coordinates into Mercator

## 0.2.1

* impl `Clone` for `Grid`, `Unit` and `Origin`

## 0.2.0

* Export tile_grid structs in top-level scope

## 0.1.1

* Add module documentation

## 0.1.0

* Extract crate from t-rex
