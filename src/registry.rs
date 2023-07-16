use crate::tile_matrix_set::TileMatrixSetOps;
use crate::tms::Tms;
use ogcapi_types::tiles::TileMatrixSet;
use once_cell::sync::OnceCell;
use std::collections::HashMap;

/// Registry of tile matrix sets
#[derive(Clone)]
pub struct TileMatrixSets {
    // Registry containing Tms is not supported because of Proj:
    // trait `Send` is not implemented for `*mut proj_sys::PJ_AREA`
    coll: HashMap<String, TileMatrixSet>,
}

#[derive(thiserror::Error, Debug)]
pub enum RegistryError {
    #[error("Tile Matrix set not found: `{0}`")]
    TmsNotFound(String),
    #[error("`{0}` is already a registered TMS")]
    TmsAlreadyRegistered(String),
    #[error(transparent)]
    TmsError(#[from] crate::tms::TmsError),
}

impl Default for TileMatrixSets {
    fn default() -> Self {
        Self::new()
    }
}

impl TileMatrixSets {
    pub fn new() -> Self {
        Self {
            coll: HashMap::new(),
        }
    }

    pub fn get(&self, id: &str) -> Result<&TileMatrixSet, RegistryError> {
        self.coll
            .get(id)
            .ok_or(RegistryError::TmsNotFound(id.to_string()))
    }

    pub fn lookup(&self, id: &str) -> Result<Tms, RegistryError> {
        self.get(id)?.try_into().map_err(Into::into)
    }

    pub fn list(&self) -> impl Iterator<Item = &String> {
        self.coll.keys()
    }

    pub fn register(
        &mut self,
        custom_tms: Vec<TileMatrixSet>,
        overwrite: bool,
    ) -> Result<(), RegistryError> {
        for tms in custom_tms {
            if self.coll.contains_key(&tms.id) {
                if overwrite {
                    self.coll.insert(tms.id.clone(), tms);
                } else {
                    return Err(RegistryError::TmsAlreadyRegistered(tms.id));
                }
            } else {
                self.coll.insert(tms.id.clone(), tms);
            }
        }
        Ok(())
    }
}

/// Global registry of tile matrix sets
pub fn tms() -> &'static TileMatrixSets {
    static TMS: OnceCell<TileMatrixSets> = OnceCell::new();
    TMS.get_or_init(|| {
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
        .map(|data| TileMatrixSet::from_json(data).unwrap())
        .collect::<Vec<_>>();
        sets.register(tms, false).unwrap();
        // user_tms_dir = os.environ.get("TILEMATRIXSET_DIRECTORY", None)
        // if user_tms_dir:
        //     tms_paths.extend(list(pathlib.Path(user_tms_dir).glob("*.json")))
        sets
    })
}
