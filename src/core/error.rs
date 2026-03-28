pub const RELIC_ERROR_CORRUPTED: &str = "Is your relic configuration corrupted?";

#[derive(Debug)]
pub enum RelicError {
    IgnoredFile,
    ConfigurationIncorrect,
    Unimplemented,
    ObjectID(ObjectID),
    BranchError(BranchError),
    IOError(IOError),
    RelicInfo(Box<RelicError>),
    SanctumError(SanctumError),
}

#[derive(Debug)]
pub enum ObjectID {
    InvalidID,
}

#[derive(Debug)]
pub enum IOError {
    InternalError,

    FileNoExist,
    FileCantOpen,
    FileCantCreate,
    FileCantWrite,
    FileCantDelete,
    FileCantCopy,

    DirectoryNoExist,
    DirectoryCantOpen,
    DirectoryCantCreate,
    DirectoryCantDelete,
}

#[derive(Debug)]
pub enum SanctumError {
    SanctumNotFound,
    RecordNoExist,
}

#[derive(Debug)]
pub enum BranchError {
    CantIterateBranches,

    BranchExists,
    BranchDoesntExist,

    DetachedCommitExists,
    DetachedCommitDoesntExist,
}
