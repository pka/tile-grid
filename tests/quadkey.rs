use tile_grid::*;

#[test]
fn test_quadkey_support() {
    let tests = vec![
        // ("LINZAntarticaMapTilegrid", false),
        //"CDB1GlobalGrid", // Error("missing field `coalesc`", line: 19, column: 67)
        #[cfg(feature = "projtransform")]
        ("EuropeanETRS89_LAEAQuad", true),
        //"GNOSISGlobalGrid", // Error("missing field `coalesc`", line: 31, column: 66)
        #[cfg(feature = "projtransform")]
        ("CanadianNAD83_LCC", false),
        #[cfg(feature = "projtransform")]
        ("UPSArcticWGS84Quad", true),
        //("NZTM2000", false),
        //("NZTM2000Quad", true),
        #[cfg(feature = "projtransform")]
        ("UTM31WGS84Quad", false),
        #[cfg(feature = "projtransform")]
        ("UPSAntarcticWGS84Quad", true),
        ("WorldMercatorWGS84Quad", true),
        //?("WGS1984Quad", false),
        ("WorldCRS84Quad", false),
        ("WebMercatorQuad", true),
    ];
    let registry = tms();
    for (name, result) in tests.into_iter() {
        dbg!(&name);
        let tms: Tms = registry.get(name).unwrap().into();
        assert_eq!(tms.is_quadtree, result);
    }
}

#[test]
fn test_quadkey() {
    let tms: Tms = tms().get("WebMercatorQuad").unwrap().into();
    let expected = "0313102310".to_string();
    assert_eq!(tms.quadkey(Tile::new(486, 332, 10)), expected);
}

#[test]
fn test_quadkey_to_tile() {
    let tms: Tms = tms().get("WebMercatorQuad").unwrap().into();
    let qk = "0313102310".to_string();
    let expected = Tile::new(486, 332, 10);
    assert_eq!(tms.quadkey_to_tile(&qk), expected);
}

#[test]
fn test_empty_quadkey_to_tile() {
    // Empty qk should give tile 0,0,0.
    let tms: Tms = tms().get("WebMercatorQuad").unwrap().into();
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
