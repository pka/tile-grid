use tile_grid::*;

#[test]
fn test_quadkey_support() {
    let tests = vec![
        ("LINZAntarticaMapTilegrid", false),
        ("EuropeanETRS89_LAEAQuad", true),
        ("CanadianNAD83_LCC", false),
        ("UPSArcticWGS84Quad", true),
        ("NZTM2000", false),
        ("NZTM2000Quad", true),
        ("UTM31WGS84Quad", false),
        ("UPSAntarcticWGS84Quad", true),
        ("WorldMercatorWGS84Quad", true),
        ("WGS1984Quad", false),
        ("WorldCRS84Quad", false),
        ("WebMercatorQuad", true),
    ];

    let registry = tms();
    for (name, result) in tests.into_iter() {
        let tms = registry.get(name).unwrap();
        assert_eq!(tms.is_quadtree, result);
    }
}

#[test]
fn test_quadkey() {
    let registry = tms();
    let tms = registry.get("WebMercatorQuad").unwrap();
    let expected = "0313102310".to_string();
    assert_eq!(tms.quadkey(Tile::new(486, 332, 10)), expected);
}

#[test]
fn test_quadkey_to_tile() {
    let registry = tms();
    let tms = registry.get("WebMercatorQuad").unwrap();
    let qk = "0313102310".to_string();
    let expected = Tile::new(486, 332, 10);
    assert_eq!(tms.quadkey_to_tile(&qk), expected);
}

#[test]
fn test_empty_quadkey_to_tile() {
    // Empty qk should give tile 0,0,0.
    let registry = tms();
    let tms = registry.get("WebMercatorQuad").unwrap();
    let qk = "";
    let expected = Tile::new(0, 0, 0);
    assert_eq!(tms.quadkey_to_tile(qk), expected);
}

// #[test]
// fn test_quadkey_failure() -> Result<(), Error> {
//     let registry = tms();
//     let tms = registry.get("WebMercatorQuad").unwrap();
//     assert!(tms.quadkey_to_tile("lolwut").is_err());
//     Ok(())
// }
