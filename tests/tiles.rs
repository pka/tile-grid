use std::iter::zip;
use tile_grid::*;

const DEFAULT_GRID_COUNT: usize = 8;

#[test]
fn test_default_grids() {
    // Morecantile.default_grids should return the correct list of grids.
    let registry = tms();
    assert_eq!(registry.list().count(), DEFAULT_GRID_COUNT);

    // assert!(matches!(
    //     registry.get("ANotValidName"),
    //     Err(morecantile::Error::InvalidIdentifier)
    // ));
}

// #[test]
// fn test_register() {
//     // Test register a new grid
//     assert_eq!(morecantile::tms::list().len(), DEFAULT_GRID_COUNT);

//     let crs = CRS::from_epsg(3031);
//     let extent = [-948.75, -543592.47, 5817.41, -3333128.95]; // From https:///epsg.io/3031
//     let tms =
//         morecantile::TileMatrixSet::custom(extent, crs, identifier: &str = "MyCustomGrid3031");

//     morecantile::tms::register(tms);
//     assert_eq!(morecantile::tms::list().len(), DEFAULT_GRID_COUNT);

//     let defaults = morecantile::tms::register(tms);
//     assert_eq!(defaults.list().len(), DEFAULT_GRID_COUNT + 1);
//     assert!(defaults.list().contains("MyCustomGrid3031"));

//     let defaults = morecantile::tms::register([tms]);
//     assert_eq!(defaults.list().len(), DEFAULT_GRID_COUNT + 1);
//     assert!(defaults.list().contains("MyCustomGrid3031"));

//     // Check it will raise an exception if TMS is already registered
//     // with pytest.raises(Exception):
//     //     defaults = defaults.register(tms)

//     // Do not raise is overwrite=True
//     defaults.register(tms, true).unwrap();
//     assert_eq!(defaults.list().len(), DEFAULT_GRID_COUNT + 1);

//     // make sure the default morecantile TMS are not overwriten
//     assert_eq!(
//         morecantile::defaults::default_tms().len(),
//         DEFAULT_GRID_COUNT
//     );

//     // add tms in morecantile defaults (not something to do anyway)
//     let epsg3031 = TileMatrixSet::custom(extent, crs, Some("epsg3031".to_string())).unwrap();
//     morecantile::defaults::default_tms_mut().insert("epsg3031".to_string(), epsg3031);
//     assert_eq!(
//         morecantile::defaults::default_tms().len(),
//         DEFAULT_GRID_COUNT + 1
//     );

//     // make sure updating the default_tms dict has no effect on the default TileMatrixSets
//     assert_eq!(morecantile::tms::list().len(), DEFAULT_GRID_COUNT);

//     // Update internal TMS dict
//     morecantile::tms::tms_mut().insert("MyCustomGrid3031".to_string(), tms);
//     assert_eq!(morecantile::tms::list().len(), DEFAULT_GRID_COUNT + 1);

//     // make sure it doesn't propagate to the default dict
//     assert!(!morecantile::defaults::default_tms().contains_key("MyCustomGrid3031"));
// }

#[test]
fn test_tms_properties() {
    // Test TileSchema().
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    assert_eq!(tms.crs().to_urn(), "urn:ogc:def:crs:EPSG:0:3857");
    assert_eq!(meters_per_unit(tms.crs()), 1.0);
    assert_eq!(tms.minzoom(), 0);
    assert_eq!(tms.maxzoom(), 24);
}

#[test]
fn test_tile_coordinates() {
    // Test coordinates to tile index utils.
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    assert_eq!(tms.tile(-179.0, 85.0, 5, false), Tile::new(0, 0, 5));

    // Check equivalence between mercantile and morecantile
    // let wlon = 20.0;
    // let wlat = 15.0;
    // assert_eq!(tms.tile(20.0, 15.0, 5), mercantile::tile(20.0, 15.0, 5));
    assert_eq!(tms.tile(20.0, 15.0, 5, false), Tile::new(17, 14, 5));
}

#[test]
fn test_bounds() {
    // TileMatrixSet.bounds should return the correct coordinates.
    let expected = [-9.140625, 53.12040528310657, -8.7890625, 53.33087298301705];
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    let bbox = tms.bounds(&Tile::new(486, 332, 10));
    for (a, b) in zip(expected, [bbox.left, bbox.bottom, bbox.right, bbox.top]) {
        assert_eq!(round_to_prec(a - b, 7).abs(), 0.0);
    }
    // assert bbox.left == bbox[0]
    // assert bbox.bottom == bbox[1]
    // assert bbox.right == bbox[2]
    // assert bbox.top == bbox[3]
}

#[test]
fn test_xy_bounds() {
    // TileMatrixSet.xy_bounds should return the correct coordinates.
    let expected = [
        -1017529.7205322663,
        7005300.768279833,
        -978393.962050256,
        7044436.526761846,
    ];
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    let bounds = tms.xy_bounds(&Tile::new(486, 332, 10));
    for (a, b) in zip(
        expected,
        [bounds.left, bounds.bottom, bounds.right, bounds.top],
    ) {
        assert_eq!(round_to_prec(a - b, 7).abs(), 0.0);
    }
}

#[test]
fn test_ul_tile() {
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    let xy = tms.ul(&Tile::new(486, 332, 10));
    let expected = [-9.140625, 53.33087298301705];
    for (a, b) in zip(expected, [xy.x, xy.y]) {
        assert!(a - b < 1e-7);
    }
}

fn round_to_prec(number: f64, precision: u8) -> f64 {
    let factor = 10.0_f64.powi(precision as i32);
    (number * factor).round() / factor
}

#[test]
fn test_projul_tile() {
    // TileMatrixSet._ul should return the correct coordinates in input projection.
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    let xy = tms.ul_(&Tile::new(486, 332, 10));
    let expected = [-1017529.7205322663, 7044436.526761846];
    for (a, b) in zip(expected, [xy.x, xy.y]) {
        assert_eq!(round_to_prec(a - b, 7).abs(), 0.0);
    }
}

#[test]
fn test_projtile() {
    // TileSchema._tile should return the correct tile.
    // tms = morecantile.tms.get("WebMercatorQuad")
    // assert tms._tile(1000, 1000, 1) == morecantile.Tile(1, 0, 1)
}

#[test]
fn test_feature() {
    // TileSchema.feature should create proper geojson feature.
    // tms = morecantile.tms.get("WebMercatorQuad")
    // feat = tms.feature(morecantile.Tile(1, 0, 1))
    // assert feat["bbox"]
    // assert feat["id"]
    // assert feat["geometry"]
    // assert len(feat["properties"].keys()) == 3

    // feat = tms.feature(
    //     morecantile.Tile(1, 0, 1),
    //     buffer=-10,
    //     precision=4,
    //     fid="1",
    //     props={"some": "thing"},
    // )
    // assert feat["bbox"]
    // assert feat["id"] == "1"
    // assert feat["geometry"]
    // assert len(feat["properties"].keys()) == 4

    // with pytest.warns(UserWarning):
    //     feat = tms.feature(
    //         morecantile.Tile(1, 0, 1), projected=True, fid="1", props={"some": "thing"}
    //     )
    // assert feat["bbox"]
    // assert feat["id"] == "1"
    // assert feat["geometry"]
    // assert len(feat["properties"].keys()) == 4
}

#[test]
fn test_ul() {
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    let expected = [-9.140625, 53.33087298301705];
    let lnglat = tms.ul(&Tile::new(486, 332, 10));
    for (a, b) in zip(expected, [lnglat.x, lnglat.y]) {
        assert_eq!(round_to_prec(a - b, 7), 0.0);
    }
    // assert_eq!(lnglat[0], lnglat.x);
    // assert_eq!(lnglat[1], lnglat.y);
}

#[test]
fn test_bbox() {
    // test bbox.
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    let expected = [-9.140625, 53.12040528310657, -8.7890625, 53.33087298301705];
    let bbox = tms.bounds(&Tile::new(486, 332, 10));
    for (a, b) in zip(expected, [bbox.left, bbox.bottom, bbox.right, bbox.top]) {
        assert_eq!(round_to_prec(a - b, 7).abs(), 0.0);
    }
    // assert_eq!(bbox.left, bbox[0]);
    // assert_eq!(bbox.bottom, bbox[1]);
    // assert_eq!(bbox.right, bbox[2]);
    // assert_eq!(bbox.top, bbox[3]);
}

#[test]
fn test_xy_tile() {
    // x, y for the 486-332-10 tile is correctly calculated.
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    let ul = tms.ul(&Tile::new(486, 332, 10));
    let xy = tms.xy(ul.x, ul.y, false);
    let expected = [-1017529.7205322663, 7044436.526761846];
    for (a, b) in zip(expected, [xy.x, xy.y]) {
        assert!((a - b).abs() < 0.0000001);
    }
}

#[test]
fn test_xy_null_island() {
    // x, y for (0, 0) is correctly calculated
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    let xy = tms.xy(0.0, 0.0, false);
    let expected = [0.0, 0.0];
    for (a, b) in zip(expected, [xy.x, xy.y]) {
        assert!((a - b).abs() < 1e-7);
    }
}

#[test]
fn test_xy_south_pole() {
    // Return -inf for y at South P
}

#[test]
fn test_xy_north_pole() {
    // Return inf for y at North Po
}

#[test]
fn test_xy_truncate() {
    // Input is truncated
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    assert_eq!(
        tms.xy(-181.0, 0.0, true),
        tms.xy(tms.bbox().left, 0.0, false)
    );
}

#[test]
fn test_lnglat() {
    // test lnglat.
}

#[test]
fn test_lnglat_gdal3() {
    // test lnglat.
}

#[test]
fn test_lnglat_xy_roundtrip() {
    // Test roundtrip.
}

#[test]
fn test_xy_bounds_mercantile() {
    // test xy_bounds.
}

#[test]
fn test_tile_not_truncated() {
    // test tile.
}

#[test]
fn test_tile_truncate() {
    // Input is truncated
}

#[test]
fn test_tiles() {
    // Test tiles from bbox.
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();

    let bounds = (-105.0, 39.99, -104.99, 40.0);
    let tiles = tms.tiles(bounds.0, bounds.1, bounds.2, bounds.3, &vec![14], false);
    let expect = vec![Tile::new(3413, 6202, 14), Tile::new(3413, 6203, 14)];
    assert_eq!(tiles.collect::<Vec<Tile>>(), expect);

    // Single zoom
    let bounds = (-105.0, 39.99, -104.99, 40.0);
    let tiles = tms.tiles(bounds.0, bounds.1, bounds.2, bounds.3, &vec![14], false);
    let expect = vec![Tile::new(3413, 6202, 14), Tile::new(3413, 6203, 14)];
    assert_eq!(tiles.collect::<Vec<Tile>>(), expect);

    // Input is truncated
    assert_eq!(
        tms.tiles(-181.0, 0.0, -170.0, 10.0, &vec![2], true)
            .collect::<Vec<Tile>>(),
        tms.tiles(-180.0, 0.0, -170.0, 10.0, &vec![2], false)
            .collect::<Vec<Tile>>()
    );

    assert_eq!(
        tms.tiles(-180.0, -90.0, 180.0, 90.0, &vec![0], false)
            .collect::<Vec<Tile>>(),
        vec![Tile::new(0, 0, 0)]
    );
    assert_eq!(
        tms.tiles(-180.0, -90.0, 180.0, 90.0, &vec![0], false)
            .collect::<Vec<Tile>>(),
        vec![Tile::new(0, 0, 0)]
    );

    // Antimeridian-crossing bounding boxes are handled
    let bounds = (175.0, 5.0, -175.0, 10.0);
    assert_eq!(
        tms.tiles(bounds.0, bounds.1, bounds.2, bounds.3, &vec![2], false)
            .count(),
        2
    );
}

#[test]
fn test_global_tiles_clamped() {
    // Y is clamped to (0, 2 ** zoom - 1).
}

#[test]
fn test_tiles_roundtrip_children() {
    // tiles(bounds(tile)) gives the tile's children
}

#[test]
fn test_tiles_roundtrip() {
    // Tiles(bounds(tile)) gives the tile.
}

#[test]
fn test_extend_zoom() {
    // TileMatrixSet.ul should return the correct coordinates.
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();

    let merc = tms.xy_bounds(&Tile::new(1000, 1000, 25));
    let more = tms.xy_bounds(&Tile::new(1000, 1000, 25));
    for (a, b) in zip(
        [more.left, more.bottom, more.right, more.top],
        [merc.left, merc.bottom, merc.right, merc.top],
    ) {
        assert_eq!(round_to_prec(a - b, 7), 0.0);
    }

    let merc = tms.xy_bounds(&Tile::new(2000, 2000, 26));
    let more = tms.xy_bounds(&Tile::new(2000, 2000, 26));
    for (a, b) in zip(
        [more.left, more.bottom, more.right, more.top],
        [merc.left, merc.bottom, merc.right, merc.top],
    ) {
        assert_eq!(round_to_prec(a - b, 7), 0.0);
    }

    let merc = tms.xy_bounds(&Tile::new(2000, 2000, 27));
    let more = tms.xy_bounds(&Tile::new(2000, 2000, 27));
    for (a, b) in zip(
        [more.left, more.bottom, more.right, more.top],
        [merc.left, merc.bottom, merc.right, merc.top],
    ) {
        assert_eq!(round_to_prec(a - b, 7), 0.0);
    }

    let merc = tms.xy_bounds(&Tile::new(2000, 2000, 30));
    let more = tms.xy_bounds(&Tile::new(2000, 2000, 30));
    for (a, b) in zip(
        [more.left, more.bottom, more.right, more.top],
        [merc.left, merc.bottom, merc.right, merc.top],
    ) {
        assert_eq!(round_to_prec(a - b, 7), 0.0);
    }
}

#[test]
fn test_is_power_of_two() {
    // is power ot 2?
}

#[test]
fn test_is_valid_tile() {
    // test if tile are valid.
}

#[test]
fn test_neighbors() {
    // test neighbors.
}

#[test]
fn test_neighbors_invalid() {
    // test neighbors.
}

#[test]
fn test_root_neighbors_invalid() {
    // test neighbors.
}

#[test]
fn test_parent() {
    // test parent
}

#[test]
fn test_parent_multi() {
    // test parent
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();
    let parent = tms.parent(&Tile::new(486, 332, 10), Some(8));
    assert_eq!(parent[0], Tile::new(121, 83, 8));
}

#[test]
fn test_children() {
    // test children.
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();

    let x = 243;
    let y = 166;
    let z = 9;
    let children = tms.children(&Tile::new(x, y, z), None);
    assert_eq!(children.len(), 4);
    assert!(children.contains(&Tile::new(2 * x, 2 * y, z + 1)));
    assert!(children.contains(&Tile::new(2 * x + 1, 2 * y, z + 1)));
    assert!(children.contains(&Tile::new(2 * x + 1, 2 * y + 1, z + 1)));
    assert!(children.contains(&Tile::new(2 * x, 2 * y + 1, z + 1)));
}

#[test]
fn test_children_multi() {
    // test children multizoom.
    let tms: TileMatrixSetInst = tms().get("WebMercatorQuad").unwrap().into();

    let children = tms.children(&Tile::new(243, 166, 9), Some(11));
    assert_eq!(children.len(), 16);
    let targets = [
        Tile::new(972, 664, 11),
        Tile::new(973, 664, 11),
        Tile::new(973, 665, 11),
        Tile::new(972, 665, 11),
        Tile::new(974, 664, 11),
        Tile::new(975, 664, 11),
        Tile::new(975, 665, 11),
        Tile::new(974, 665, 11),
        Tile::new(974, 666, 11),
        Tile::new(975, 666, 11),
        Tile::new(975, 667, 11),
        Tile::new(974, 667, 11),
        Tile::new(972, 666, 11),
        Tile::new(973, 666, 11),
        Tile::new(973, 667, 11),
        Tile::new(972, 667, 11),
    ];
    for target in targets {
        assert!(children.contains(&target));
    }
}

#[test]
fn test_children_invalid_zoom() {
    // invalid zoom.
}
