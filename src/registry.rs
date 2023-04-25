use crate::tms::{TileMatrixSet, TileMatrixSetData};
use std::collections::HashMap;
use std::fs::read_to_string;

pub struct TileMatrixSets {
    coll: HashMap<String, TileMatrixSet>,
}

impl TileMatrixSets {
    fn new() -> Self {
        Self {
            coll: HashMap::new(),
        }
    }

    pub fn get(&self, identifier: &str) -> Option<&TileMatrixSet> {
        self.coll.get(identifier)
    }

    pub fn list(&self) -> impl Iterator<Item = &String> {
        self.coll.keys().into_iter()
    }

    pub fn register(&mut self, custom_tms: Vec<TileMatrixSetData>, overwrite: bool) {
        for tmsd in custom_tms {
            let tms = TileMatrixSet::init(tmsd);
            if self.coll.contains_key(&tms.tms.identifier) {
                if overwrite {
                    self.coll.insert(tms.tms.identifier.clone(), tms);
                } else {
                    panic!("{} is already a registered TMS.", &tms.tms.identifier)
                }
            } else {
                self.coll.insert(tms.tms.identifier.clone(), tms);
            }
        }
    }
}

// static OnceCell not supported because of Proj:
// trait `Send` is not implemented for `*mut proj_sys::PJ_AREA`

pub fn tms() -> TileMatrixSets {
    let mut sets = TileMatrixSets::new();
    let tms = vec![
        "CanadianNAD83_LCC.json",
        "EuropeanETRS89_LAEAQuad.json",
        "LINZAntarticaMapTilegrid.json",
        "NZTM2000.json",
        "NZTM2000Quad.json",
        "UPSAntarcticWGS84Quad.json",
        "UPSArcticWGS84Quad.json",
        "UTM31WGS84Quad.json",
        "WebMercatorQuad.json",
        "WGS1984Quad.json",
        "WorldCRS84Quad.json",
        "WorldMercatorWGS84Quad.json",
    ]
    .into_iter()
    .map(|f| {
        let data = read_to_string(&format!("./data/{f}")).unwrap();
        let tms: TileMatrixSetData = serde_json::from_str(&data).unwrap();
        tms
    })
    .collect::<Vec<_>>();
    sets.register(tms, true);
    // user_tms_dir = os.environ.get("TILEMATRIXSET_DIRECTORY", None)
    // if user_tms_dir:
    //     tms_paths.extend(list(pathlib.Path(user_tms_dir).glob("*.json")))
    sets
}
