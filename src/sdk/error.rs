#[derive(Clone, Debug, PartialEq)]
pub enum Err {
    Generic,
    SgxError,
    SgxWriteError,
    AnchorParseError,
    VerifierMissing,
}
