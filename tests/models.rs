use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use tile_grid::morecantile::*;

#[test]
fn test_tile_matrix_set() {
    let tilesets = Path::new("./data")
        .read_dir()
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().unwrap_or_default() == "json")
        .collect::<Vec<_>>();
    assert!(tilesets.contains(&PathBuf::from("./data/WebMercatorQuad.json")));

    // Load TileMatrixSet in models.
    // Confirm model validation is working
    for tileset in tilesets {
        // let ts = TileMatrixSet::parse_file(tilesets).unwrap();
        let data = read_to_string(tileset).unwrap();
        let tms: TileMatrixSetData = serde_json::from_str(&data).unwrap();
        let ts = TileMatrixSet::init(tms);
        // This would fail if `supportedCRS` isn't supported by PROJ
        assert!(ts.crs().len() > 0);
    }
}

#[test]
fn test_tile_matrix_iter() {
    // Test iterator
    let registry = tms();
    let tms = registry.get("WebMercatorQuad").unwrap();
    assert_eq!(tms.matrices().len(), 25);
}

// #[test]
// fn test_tile_matrix_order() {
//     let tms = morecantile::tms::get("WebMercatorQuad");
//     let mut matrices = tms.tile_matrix.clone();
//     matrices.shuffle(&mut rand::thread_rng());
//     let tms_ordered = morecantile::TileMatrixSet {
//         title: tms.title.clone(),
//         identifier: tms.identifier.clone(),
//         supported_crs: tms.supported_crs.clone(),
//         tile_matrix: matrices,
//     };
//     // Confirm sort
//     assert_eq!(
//         tms.tile_matrix
//             .iter()
//             .map(|matrix| matrix.identifier.clone())
//             .collect::<Vec<_>>(),
//         tms_ordered
//             .tile_matrix
//             .iter()
//             .map(|matrix| matrix.identifier.clone())
//             .collect::<Vec<_>>()
//     );

//     // Confirm sort direction
//     assert!(
//         tms_ordered
//             .tile_matrix
//             .last()
//             .unwrap()
//             .identifier
//             .parse::<i32>()
//             .unwrap()
//             > tms_ordered
//                 .tile_matrix
//                 .first()
//                 .unwrap()
//                 .identifier
//                 .parse::<i32>()
//                 .unwrap()
//     );
// }

// #[test]
// fn test_tile_matrix() {
//     let variable_matrix = morecantile::TileMatrix {
//         identifier: "3".to_string(),
//         scale_denominator: 34942641.5017948,
//         top_left_corner: (-180.0, 90.0),
//         tile_width: 256,
//         tile_height: 256,
//         matrix_width: 16,
//         matrix_height: 8,
//         variable_matrix_width: Some(vec![
//             morecantile::VariableMatrixWidth {
//                 coalesce: 2,
//                 min_tile_row: 0,
//                 max_tile_row: 0,
//             },
//             morecantile::VariableMatrixWidth {
//                 coalesce: 2,
//                 min_tile_row: 3,
//                 max_tile_row: 3,
//             },
//         ]),
//     };
//     assert!(variable_matrix.validate().is_err());
// }

// #[test]
// fn test_invalid_tms() {
//     assert!(morecantile::tms::get("ANotValidName").is_err());
// }

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

#[test]
fn load_from_json() {
    let data = read_to_string("./data/WebMercatorQuad.json").unwrap();
    let tms: TileMatrixSetData = serde_json::from_str(&data).unwrap();
    // println!("{tms:#?}");
    assert_eq!(tms.identifier, "WebMercatorQuad");
    let web_mercator_quad = web_mercator_quad();
    assert_eq!(tms.supported_crs, web_mercator_quad.supported_crs);

    let tms = TileMatrixSet::init(tms);
    assert_eq!(tms.crs(), web_mercator_quad.supported_crs);
    // assert!(tms.is_quadtree);
}

fn web_mercator_quad() -> TileMatrixSetData {
    TileMatrixSetData {
        type_: "TileMatrixSetType".to_string(),
        title: "Google Maps Compatible for the World".to_string(),
        abstract_: None,
        keywords: None,
        identifier: "WebMercatorQuad".to_string(),
        supported_crs: "urn:ogc:def:crs:EPSG::3857".to_string(),
        well_known_scale_set: Some(
            "http://www.opengis.net/def/wkss/OGC/1.0/GoogleMapsCompatible".to_string(),
        ),
        bounding_box: Some(TMSBoundingBox {
            type_: "BoundingBoxType".to_string(),
            crs: "urn:ogc:def:crs:EPSG::3857".to_string(),
            lower_corner: [-20037508.342789244, -20037508.342789244],
            upper_corner: [20037508.342789244, 20037508.342789244],
        }),
        tile_matrix: vec![
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "0".to_string(),
                scale_denominator: 559082264.0287178,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 1,
                matrix_height: 1,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "1".to_string(),
                scale_denominator: 279541132.0143589,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 2,
                matrix_height: 2,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "2".to_string(),
                scale_denominator: 139770566.00717944,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 4,
                matrix_height: 4,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "3".to_string(),
                scale_denominator: 69885283.00358972,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 8,
                matrix_height: 8,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "4".to_string(),
                scale_denominator: 34942641.50179486,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 16,
                matrix_height: 16,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "5".to_string(),
                scale_denominator: 17471320.75089743,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 32,
                matrix_height: 32,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "6".to_string(),
                scale_denominator: 8735660.375448715,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 64,
                matrix_height: 64,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "7".to_string(),
                scale_denominator: 4367830.1877243575,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 128,
                matrix_height: 128,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "8".to_string(),
                scale_denominator: 2183915.0938621787,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 256,
                matrix_height: 256,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "9".to_string(),
                scale_denominator: 1091957.5469310894,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 512,
                matrix_height: 512,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "10".to_string(),
                scale_denominator: 545978.7734655447,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 1024,
                matrix_height: 1024,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "11".to_string(),
                scale_denominator: 272989.38673277234,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 2048,
                matrix_height: 2048,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "12".to_string(),
                scale_denominator: 136494.69336638617,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 4096,
                matrix_height: 4096,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "13".to_string(),
                scale_denominator: 68247.34668319309,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 8192,
                matrix_height: 8192,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "14".to_string(),
                scale_denominator: 34123.67334159654,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 16384,
                matrix_height: 16384,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "15".to_string(),
                scale_denominator: 17061.83667079827,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 32768,
                matrix_height: 32768,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "16".to_string(),
                scale_denominator: 8530.918335399136,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 65536,
                matrix_height: 65536,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "17".to_string(),
                scale_denominator: 4265.459167699568,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 131072,
                matrix_height: 131072,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "18".to_string(),
                scale_denominator: 2132.729583849784,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 262144,
                matrix_height: 262144,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "19".to_string(),
                scale_denominator: 1066.364791924892,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 524288,
                matrix_height: 524288,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "20".to_string(),
                scale_denominator: 533.182395962446,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 1048576,
                matrix_height: 1048576,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "21".to_string(),
                scale_denominator: 266.591197981223,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 2097152,
                matrix_height: 2097152,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "22".to_string(),
                scale_denominator: 133.2955989906115,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 4194304,
                matrix_height: 4194304,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "23".to_string(),
                scale_denominator: 66.64779949530575,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 8388608,
                matrix_height: 8388608,
            },
            TileMatrix {
                type_: "TileMatrixType".to_string(),
                title: None,
                abstract_: None,
                keywords: None,
                identifier: "24".to_string(),
                scale_denominator: 33.323899747652874,
                top_left_corner: [-20037508.342789244, 20037508.342789244],
                tile_width: 256,
                tile_height: 256,
                matrix_width: 16777216,
                matrix_height: 16777216,
            },
        ],
    }
}
