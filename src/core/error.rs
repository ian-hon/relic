#[derive(Debug)]
pub enum RelicError {
    IgnoredFile,
    ConfigurationIncorrect,
    IOError(IOError),
    RelicInfo(Box<RelicError>),
    SanctumError(SanctumError),
}

#[derive(Debug)]
pub enum IOError {
    FileNoExist,
    FileCantOpen,
    FileCantCreate,
    DirectoryNoExist,
    DirectoryCantOpen,
    DirectoryCantCreate,
}

#[derive(Debug)]
pub enum SanctumError {
    SanctumNotFound,
    RecordNoExist,
}
