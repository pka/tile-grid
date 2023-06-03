use crate::{Authority::EPSG, Crs};
use core::num::{NonZeroU16, NonZeroU64};
use std::path::{Path, PathBuf};
use tile_grid::*;

#[test]
fn test_tile_matrix_set() {
    let tilesets = Path::new("./data")
        .read_dir()
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().unwrap_or_default() == "json")
        .filter(|p| {
            !vec!["./data/CDB1GlobalGrid.json", "./data/GNOSISGlobalGrid.json"]
                .contains(&p.as_os_str().to_str().unwrap())
        })
        .collect::<Vec<_>>();
    assert!(tilesets.contains(&PathBuf::from("./data/WebMercatorQuad.json")));

    // Load TileMatrixSet in models.
    // Confirm model validation is working
    #[cfg(feature = "projtransform")] // TODO: load only Mercator TMS
    for tileset in tilesets {
        // let ts = TileMatrixSet::parse_file(tilesets).unwrap();
        let data = std::fs::read_to_string(tileset).unwrap();
        let tms = TileMatrixSet::from_json(&data).unwrap().into_tms().unwrap();
        // This would fail if `supportedCRS` isn't supported by PROJ
        assert!(tms.crs().as_known_crs().len() > 0);
    }
}

#[test]
fn test_tile_matrix_iter() {
    // Test iterator
    let tms = tms().lookup("WebMercatorQuad").unwrap();
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

#[test]
fn test_invalid_tms() {
    assert!(tms().get("ANotValidName").is_err());
    assert!(tms().lookup("ANotValidName").is_err());
}

#[test]
fn morecantile_examples() {
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
    //>>> BoundingBox(left=5009377.085697308, bottom=-7514065.628545959, right=7514065.628545959, top=-5009377.085697308)

    // Get the bounds for tile Z=4, X=10, Y=10 in LatLon (WGS84)
    let bounds = tms.bounds(&Tile::new(10, 10, 4)).unwrap();
    assert_eq!(
        bounds,
        BoundingBox::new(45.0, -55.77657301866769, 67.5, -40.97989806962013)
    );
    // >>> BoundingBox(left=44.999999999999964, bottom=-55.776573018667634, right=67.4999999999999, top=-40.97989806962009)

    // Find tile for lat/lon

    //let tms = tms().lookup("WebMercatorQuad").unwrap();

    let tile = tms.tile(159.31, -42.0, 4).unwrap();
    assert_eq!(tile, Tile::new(15, 10, 4));

    // Or using coordinates in input CRS
    let coord = tms.xy(159.31, -42.0).unwrap();
    if cfg!(projtransform) {
        assert_eq!((coord.x, coord.y), (17734308.078276414, -5160979.444049783));
    } else {
        //assert_eq!((coord.x, coord.y), (17734308.078276414, -5160979.444049781));
    }

    let tile = tms.xy_tile(17734308.1, -5160979.4, 4);
    assert_eq!(tile, Tile::new(15, 10, 4));
}

#[test]
fn mercator_tms() {
    let tms = tms().get("WebMercatorQuad").unwrap();
    assert_eq!(
        serde_json::to_string_pretty(&tms).unwrap(),
        serde_json::to_string_pretty(&web_mercator_quad()).unwrap()
    );
}

fn web_mercator_quad() -> TileMatrixSet {
    TileMatrixSet {
        title_description_keywords: TitleDescriptionKeywords {
            title: Some("Google Maps Compatible for the World".to_string()),
            description: None,
            keywords: None,
        },
        id: "WebMercatorQuad".to_string(),
        uri: Some("http://www.opengis.net/def/tilematrixset/OGC/1.0/WebMercatorQuad".to_string()),
        crs: Crs {
            authority: EPSG,
            version: "0".to_string(),
            code: "3857".to_string(),
        },
        ordered_axes: Some(vec!["X".to_string(), "Y".to_string()]),
        well_known_scale_set: Some(
            "http://www.opengis.net/def/wkss/OGC/1.0/GoogleMapsCompatible".to_string(),
        ),
        bounding_box: None,
        tile_matrices: vec![
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "0".to_string(),
                scale_denominator: 559082264.028717,
                cell_size: 156543.033928041,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(1).unwrap(),
                matrix_height: NonZeroU64::new(1).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "1".to_string(),
                scale_denominator: 279541132.014358,
                cell_size: 78271.5169640204,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(2).unwrap(),
                matrix_height: NonZeroU64::new(2).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "2".to_string(),
                scale_denominator: 139770566.007179,
                cell_size: 39135.7584820102,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(4).unwrap(),
                matrix_height: NonZeroU64::new(4).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "3".to_string(),
                scale_denominator: 69885283.0035897,
                cell_size: 19567.8792410051,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(8).unwrap(),
                matrix_height: NonZeroU64::new(8).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "4".to_string(),
                scale_denominator: 34942641.5017948,
                cell_size: 9783.93962050256,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(16).unwrap(),
                matrix_height: NonZeroU64::new(16).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "5".to_string(),
                scale_denominator: 17471320.7508974,
                cell_size: 4891.96981025128,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(32).unwrap(),
                matrix_height: NonZeroU64::new(32).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "6".to_string(),
                scale_denominator: 8735660.37544871,
                cell_size: 2445.98490512564,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(64).unwrap(),
                matrix_height: NonZeroU64::new(64).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "7".to_string(),
                scale_denominator: 4367830.18772435,
                cell_size: 1222.99245256282,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(128).unwrap(),
                matrix_height: NonZeroU64::new(128).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "8".to_string(),
                scale_denominator: 2183915.09386217,
                cell_size: 611.49622628141,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(256).unwrap(),
                matrix_height: NonZeroU64::new(256).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "9".to_string(),
                scale_denominator: 1091957.54693108,
                cell_size: 305.748113140704,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(512).unwrap(),
                matrix_height: NonZeroU64::new(512).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "10".to_string(),
                scale_denominator: 545978.773465544,
                cell_size: 152.874056570352,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(1024).unwrap(),
                matrix_height: NonZeroU64::new(1024).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "11".to_string(),
                scale_denominator: 272989.386732772,
                cell_size: 76.4370282851762,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(2048).unwrap(),
                matrix_height: NonZeroU64::new(2048).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "12".to_string(),
                scale_denominator: 136494.693366386,
                cell_size: 38.2185141425881,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(4096).unwrap(),
                matrix_height: NonZeroU64::new(4096).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "13".to_string(),
                scale_denominator: 68247.346683193,
                cell_size: 19.109257071294,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(8192).unwrap(),
                matrix_height: NonZeroU64::new(8192).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "14".to_string(),
                scale_denominator: 34123.6733415964,
                cell_size: 9.55462853564703,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(16384).unwrap(),
                matrix_height: NonZeroU64::new(16384).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "15".to_string(),
                scale_denominator: 17061.8366707982,
                cell_size: 4.77731426782351,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(32768).unwrap(),
                matrix_height: NonZeroU64::new(32768).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "16".to_string(),
                scale_denominator: 8530.91833539913,
                cell_size: 2.38865713391175,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(65536).unwrap(),
                matrix_height: NonZeroU64::new(65536).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "17".to_string(),
                scale_denominator: 4265.45916769956,
                cell_size: 1.19432856695587,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(131072).unwrap(),
                matrix_height: NonZeroU64::new(131072).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "18".to_string(),
                scale_denominator: 2132.72958384978,
                cell_size: 0.597164283477939,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(262144).unwrap(),
                matrix_height: NonZeroU64::new(262144).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "19".to_string(),
                scale_denominator: 1066.36479192489,
                cell_size: 0.29858214173897,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(524288).unwrap(),
                matrix_height: NonZeroU64::new(524288).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "20".to_string(),
                scale_denominator: 533.182395962445,
                cell_size: 0.149291070869485,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(1048576).unwrap(),
                matrix_height: NonZeroU64::new(1048576).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "21".to_string(),
                scale_denominator: 266.591197981222,
                cell_size: 0.0746455354347424,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(2097152).unwrap(),
                matrix_height: NonZeroU64::new(2097152).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "22".to_string(),
                scale_denominator: 133.295598990611,
                cell_size: 0.0373227677173712,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(4194304).unwrap(),
                matrix_height: NonZeroU64::new(4194304).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "23".to_string(),
                scale_denominator: 66.6477994953056,
                cell_size: 0.0186613838586856,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(8388608).unwrap(),
                matrix_height: NonZeroU64::new(8388608).unwrap(),
                variable_matrix_widths: None,
            },
            TileMatrix {
                title_description_keywords: TitleDescriptionKeywords {
                    title: None,
                    description: None,
                    keywords: None,
                },
                id: "24".to_string(),
                scale_denominator: 33.3238997476528,
                cell_size: 0.0093306919293428,
                corner_of_origin: None,
                point_of_origin: [-20037508.3427892, 20037508.3427892],
                tile_width: NonZeroU16::new(256).unwrap(),
                tile_height: NonZeroU16::new(256).unwrap(),
                matrix_width: NonZeroU64::new(16777216).unwrap(),
                matrix_height: NonZeroU64::new(16777216).unwrap(),
                variable_matrix_widths: None,
            },
        ],
    }
}

#[test]
#[cfg(feature = "projtransform")]
fn custom_lv95() {
    let custom_tms = Tms::custom_resolutions(
        vec![2420000.0, 1030000.0, 2900000.0, 1350000.0],
        &Crs::from_epsg(2056),
        256,
        256,
        Some(&Crs::from_epsg(2056)),
        vec![
            4000.0, 3750.0, 3500.0, 3250.0, 3000.0, 2750.0, 2500.0, 2250.0, 2000.0, 1750.0, 1500.0,
            1250.0, 1000.0, 750.0, 650.0, 500.0, 250.0, 100.0, 50.0, 20.0, 10.0, 5.0, 2.5, 2.0,
            1.5, 1.0, 0.5,
        ],
        "LV95/CH1903+",
        "LV95",
        Some(vec!["E".to_string(), "N".to_string()]),
        &Crs::default(),
    )
    .unwrap();

    let lv95_json = r##"{
  "title": "LV95/CH1903+",
  "id": "LV95",
  "crs": "http://www.opengis.net/def/crs/EPSG/0/2056",
  "orderedAxes": [
    "E",
    "N"
  ],
  "boundingBox": {
    "lowerLeft": [
      2420000.0,
      1030000.0
    ],
    "upperRight": [
      2900000.0,
      1350000.0
    ],
    "crs": "http://www.opengis.net/def/crs/EPSG/0/2056",
    "orderedAxes": [
      "E",
      "N"
    ]
  },
  "tileMatrices": [
    {
      "id": "0",
      "scaleDenominator": 14285714.285714287,
      "cellSize": 4000.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1,
      "matrixHeight": 1
    },
    {
      "id": "1",
      "scaleDenominator": 13392857.142857144,
      "cellSize": 3750.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1,
      "matrixHeight": 1
    },
    {
      "id": "2",
      "scaleDenominator": 12500000.000000002,
      "cellSize": 3500.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1,
      "matrixHeight": 1
    },
    {
      "id": "3",
      "scaleDenominator": 11607142.857142858,
      "cellSize": 3250.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1,
      "matrixHeight": 1
    },
    {
      "id": "4",
      "scaleDenominator": 10714285.714285715,
      "cellSize": 3000.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1,
      "matrixHeight": 1
    },
    {
      "id": "5",
      "scaleDenominator": 9821428.571428573,
      "cellSize": 2750.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1,
      "matrixHeight": 1
    },
    {
      "id": "6",
      "scaleDenominator": 8928571.42857143,
      "cellSize": 2500.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1,
      "matrixHeight": 1
    },
    {
      "id": "7",
      "scaleDenominator": 8035714.285714286,
      "cellSize": 2250.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1,
      "matrixHeight": 1
    },
    {
      "id": "8",
      "scaleDenominator": 7142857.142857144,
      "cellSize": 2000.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1,
      "matrixHeight": 1
    },
    {
      "id": "9",
      "scaleDenominator": 6250000.000000001,
      "cellSize": 1750.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 2,
      "matrixHeight": 1
    },
    {
      "id": "10",
      "scaleDenominator": 5357142.857142857,
      "cellSize": 1500.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 2,
      "matrixHeight": 1
    },
    {
      "id": "11",
      "scaleDenominator": 4464285.714285715,
      "cellSize": 1250.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 2,
      "matrixHeight": 1
    },
    {
      "id": "12",
      "scaleDenominator": 3571428.571428572,
      "cellSize": 1000.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 2,
      "matrixHeight": 2
    },
    {
      "id": "13",
      "scaleDenominator": 2678571.4285714286,
      "cellSize": 750.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 3,
      "matrixHeight": 2
    },
    {
      "id": "14",
      "scaleDenominator": 2321428.571428572,
      "cellSize": 650.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 3,
      "matrixHeight": 2
    },
    {
      "id": "15",
      "scaleDenominator": 1785714.285714286,
      "cellSize": 500.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 4,
      "matrixHeight": 3
    },
    {
      "id": "16",
      "scaleDenominator": 892857.142857143,
      "cellSize": 250.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 8,
      "matrixHeight": 5
    },
    {
      "id": "17",
      "scaleDenominator": 357142.85714285716,
      "cellSize": 100.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 19,
      "matrixHeight": 13
    },
    {
      "id": "18",
      "scaleDenominator": 178571.42857142858,
      "cellSize": 50.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 38,
      "matrixHeight": 25
    },
    {
      "id": "19",
      "scaleDenominator": 71428.57142857143,
      "cellSize": 20.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 94,
      "matrixHeight": 63
    },
    {
      "id": "20",
      "scaleDenominator": 35714.28571428572,
      "cellSize": 10.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 188,
      "matrixHeight": 125
    },
    {
      "id": "21",
      "scaleDenominator": 17857.14285714286,
      "cellSize": 5.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 375,
      "matrixHeight": 250
    },
    {
      "id": "22",
      "scaleDenominator": 8928.57142857143,
      "cellSize": 2.5,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 750,
      "matrixHeight": 500
    },
    {
      "id": "23",
      "scaleDenominator": 7142.857142857143,
      "cellSize": 2.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 938,
      "matrixHeight": 625
    },
    {
      "id": "24",
      "scaleDenominator": 5357.142857142858,
      "cellSize": 1.5,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1250,
      "matrixHeight": 834
    },
    {
      "id": "25",
      "scaleDenominator": 3571.4285714285716,
      "cellSize": 1.0,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 1875,
      "matrixHeight": 1250
    },
    {
      "id": "26",
      "scaleDenominator": 1785.7142857142858,
      "cellSize": 0.5,
      "pointOfOrigin": [
        2420000.0,
        1350000.0
      ],
      "tileWidth": 256,
      "tileHeight": 256,
      "matrixWidth": 3750,
      "matrixHeight": 2500
    }
  ]
}"##;
    // println!("{}", serde_json::to_string_pretty(&custom_tms.tms).unwrap());
    assert_eq!(
        serde_json::to_string_pretty(&custom_tms.tms).unwrap(),
        lv95_json
    );
}
