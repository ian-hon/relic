#[derive(Debug)]
pub enum RelicError {
    FileCantOpen,
    IgnoredFile,
    ConfigurationIncorrect,
    RelicInfo(Box<RelicError>),
    SanctumError(SanctumError),
}

#[derive(Debug)]
pub enum SanctumError {
    SanctumNotFound,
    RecordNoExist,
}
