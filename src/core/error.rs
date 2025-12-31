#[derive(Debug)]
pub enum RelicError {
    FileCantOpen,
    IgnoredFile,
    ConfigurationIncorrect,
    RelicInfo(Box<RelicError>),
}
