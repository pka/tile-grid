use thiserror::Error;

#[derive(Error, Debug)]
pub enum MorecantileError {
    #[error("Invalid TileMatrixSet indentifier")]
    InvalidIdentifier,
    #[error("Raised when math errors occur beyond ~85 degrees N or S")]
    InvalidLatitudeError,
    #[error("Raised when errors occur in parsing a function's tile arg(s)")]
    TileArgParsingError,
    #[error("Point is outside TMS bounds")]
    PointOutsideTMSBounds,
    #[error("Raised when a custom TileMatrixSet doesn't support quadkeys")]
    NoQuadkeySupport,
    #[error("Raised when errors occur in computing or parsing quad keys")]
    QuadKeyError,
    #[error("TileMatrix not found for level: {0} - Unable to construct tileMatrix for TMS with variable scale")]
    InvalidZoomError(u16),
}
