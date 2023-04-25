use crate::tms::{TileMatrixSet, TileMatrixSetInst};
use std::collections::HashMap;
use std::fs::read_to_string;

pub struct TileMatrixSets {
    coll: HashMap<String, TileMatrixSetInst>,
}

impl TileMatrixSets {
    fn new() -> Self {
        Self {
            coll: HashMap::new(),
        }
    }

    pub fn get(&self, id: &str) -> Option<&TileMatrixSetInst> {
        self.coll.get(id)
    }

    pub fn list(&self) -> impl Iterator<Item = &String> {
        self.coll.keys().into_iter()
    }

    pub fn register(&mut self, custom_tms: Vec<TileMatrixSet>, overwrite: bool) {
        for tmsd in custom_tms {
            let tms = TileMatrixSetInst::init(tmsd);
            if self.coll.contains_key(&tms.tms.id) {
                if overwrite {
                    self.coll.insert(tms.tms.id.clone(), tms);
                } else {
                    panic!("{} is already a registered TMS.", &tms.tms.id)
                }
            } else {
                self.coll.insert(tms.tms.id.clone(), tms);
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
        //"CDB1GlobalGrid.json", // Error("missing field `coalesc`", line: 19, column: 67)
        "EuropeanETRS89_LAEAQuad.json",
        //"GNOSISGlobalGrid.json", // Error("missing field `coalesc`", line: 31, column: 66)
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
        let tms: TileMatrixSet = serde_json::from_str(&data).unwrap();
        tms
    })
    .collect::<Vec<_>>();
    sets.register(tms, true);
    // user_tms_dir = os.environ.get("TILEMATRIXSET_DIRECTORY", None)
    // if user_tms_dir:
    //     tms_paths.extend(list(pathlib.Path(user_tms_dir).glob("*.json")))
    sets
}
