use crate::tile_matrix_set::TileMatrixSet;
use once_cell::sync::OnceCell;
use std::collections::HashMap;

pub struct TileMatrixSets {
    // Registry containing TileMatrixSetsImpl not supported because of Proj:
    // trait `Send` is not implemented for `*mut proj_sys::PJ_AREA`
    coll: HashMap<String, TileMatrixSet>,
}

impl TileMatrixSets {
    fn new() -> Self {
        Self {
            coll: HashMap::new(),
        }
    }

    pub fn get(&self, id: &str) -> Option<&TileMatrixSet> {
        self.coll.get(id)
    }

    pub fn list(&self) -> impl Iterator<Item = &String> {
        self.coll.keys().into_iter()
    }

    pub fn register(&mut self, custom_tms: Vec<TileMatrixSet>, overwrite: bool) {
        for tms in custom_tms {
            if self.coll.contains_key(&tms.id) {
                if overwrite {
                    self.coll.insert(tms.id.clone(), tms);
                } else {
                    panic!("{} is already a registered TMS.", &tms.id)
                }
            } else {
                self.coll.insert(tms.id.clone(), tms);
            }
        }
    }
}

/// Global registry of tile matrix sets
pub fn tms() -> &'static TileMatrixSets {
    static TMS: OnceCell<TileMatrixSets> = OnceCell::new();
    &TMS.get_or_init(|| {
        const WEB_MERCARTOR_QUAD: &[u8; 7744] = include_bytes!("../data/WebMercatorQuad.json");
        let mut sets = TileMatrixSets::new();
        let tms = vec![
            #[cfg(feature = "projtransform")]
            include_str!("../data/CanadianNAD83_LCC.json"),
            //include_str!("../data/CDB1GlobalGrid.json"), // Error("missing field `coalesc`", line: 19, column: 67)
            #[cfg(feature = "projtransform")]
            include_str!("../data/EuropeanETRS89_LAEAQuad.json"),
            //include_str!("../data/GNOSISGlobalGrid.json"), // Error("missing field `coalesc`", line: 31, column: 66)
            #[cfg(feature = "projtransform")]
            include_str!("../data/UPSAntarcticWGS84Quad.json"),
            #[cfg(feature = "projtransform")]
            include_str!("../data/UPSArcticWGS84Quad.json"),
            #[cfg(feature = "projtransform")]
            include_str!("../data/UTM31WGS84Quad.json"),
            include_str!("../data/WebMercatorQuad.json"),
            include_str!("../data/WGS1984Quad.json"),
            //include_str!("../data/WorldCRS84Quad.json"), // conflicts with WGS1984Quad
            include_str!("../data/WorldMercatorWGS84Quad.json"),
        ]
        .into_iter()
        .map(|data| {
            let tms: TileMatrixSet = serde_json::from_str(data).unwrap();
            tms
        })
        .collect::<Vec<_>>();
        sets.register(tms, false);
        // user_tms_dir = os.environ.get("TILEMATRIXSET_DIRECTORY", None)
        // if user_tms_dir:
        //     tms_paths.extend(list(pathlib.Path(user_tms_dir).glob("*.json")))
        sets
    })
}
