use ogcapi_types::tiles::{OrderedAxes, TileMatrixSet};
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum TileMatrixSetError {
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error("{0}: {1}")]
    FileError(PathBuf, #[source] std::io::Error),
}

pub trait TileMatrixSetOps: Sized {
    fn from_json_file(json_path: &str) -> Result<Self, TileMatrixSetError>;
    fn from_json(json: &str) -> Result<Self, TileMatrixSetError>;
    /// Check if CRS has inverted AXIS (lat,lon) instead of (lon,lat).
    fn crs_axis_inverted(&self) -> bool;
}

impl TileMatrixSetOps for TileMatrixSet {
    fn from_json_file(json_path: &str) -> Result<Self, TileMatrixSetError> {
        let content = std::fs::read_to_string(json_path)
            .map_err(|e| TileMatrixSetError::FileError(json_path.into(), e))?;
        TileMatrixSet::from_json(&content)
    }
    fn from_json(json: &str) -> Result<Self, TileMatrixSetError> {
        serde_json::from_str(&json).map_err(Into::into)
    }
    /// Check if CRS has inverted AXIS (lat,lon) instead of (lon,lat).
    fn crs_axis_inverted(&self) -> bool {
        if let Some(axes) = &self.ordered_axes {
            ordered_axes_inverted(axes)
        } else {
            false // TODO: Check CRS axis ordering
        }
    }
}

pub(crate) fn ordered_axes_inverted(axes: &OrderedAxes) -> bool {
    first_axes_inverted(&axes[0].to_uppercase())
}

fn first_axes_inverted(first: &str) -> bool {
    first == "Y" || first == "LAT" || first == "N"
}

#[cfg(test)]
mod test {
    use super::TileMatrixSetOps;
    use ogcapi_types::tiles::TileMatrixSet;

    #[test]
    fn parse_tms_example() {
        let tms = TileMatrixSet::from_json_file("./data/WebMercatorQuad.json").unwrap();
        println!("{}", serde_json::to_string_pretty(&tms).unwrap());
    }
}
