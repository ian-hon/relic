use std::{fs, path::PathBuf, str::FromStr};

use crate::core::{
    data::commit::Commit,
    error::{BranchError, IOError, RelicError},
    object::Object,
    oid::ObjectID,
    state::State,
};

pub const DELIMITER: &str = "branch";
pub const DEFAULT_BRANCH: &str = "main";

/*
two formats:
- just OID
    - means detached HEAD state, not attributed to any branch per se
- branch\0{branch name}
    - connected to branch
*/
#[derive(Clone, Debug)]
pub struct Branch {
    pub name: String,
    pub head: ObjectID,
}
impl Branch {
    // instance methods
    pub fn get_commit(self, sanctum_path: &PathBuf) -> Result<Commit, RelicError> {
        if let Ok(result) = self.head.construct(sanctum_path) {
            return match result {
                Object::Commit(c) => Ok(c),
                _ => Err(RelicError::ConfigurationIncorrect),
            };
        }

        Err(RelicError::ConfigurationIncorrect)
    }

    // pub fn update_head(self, commit: Commit, state: &State) -> Option<RelicError> {
    //     if let Ok(()) = fs::write(file_path, commit.oid.to_string()) {
    //         if let Some(b) = Branch::construct_from_name(&name, state) {
    //             return Ok(b);
    //         }
    //         return Err(RelicError::BranchError(BranchError::BranchDoesntExist));
    //     }
    // }

    // static methods
    // input: name of branch
    // output: branch from branches/
    pub fn construct_from_name(name: &str, state: &State) -> Option<Branch> {
        if let Ok(b) = Branch::construct_from_path(&state.branches_path.join(name)) {
            return Some(b);
        }
        None
    }

    // input: path of branch
    // output: branch from branches/
    pub fn construct_from_path(path: &PathBuf) -> Result<Branch, RelicError> {
        // EXPENSIVE!
        if let Some(f) = path.file_name() {
            let name = f.to_string_lossy().to_string();

            if let Ok(content) = fs::read_to_string(path) {
                let oid = ObjectID::from_string(&content);

                return Ok(Branch { name, head: oid });
            }

            return Err(RelicError::IOError(IOError::FileCantOpen));
        }
        Err(RelicError::IOError(IOError::FileNoExist))
    }

    pub fn get_head(state: &State) -> Result<HeadType, RelicError> {
        let file_path = state.get_head_path();
        if !file_path.exists() {
            return Err(RelicError::IOError(IOError::FileNoExist));
        }

        if let Ok(s) = fs::read_to_string(file_path) {
            return HeadType::deserialise(&s, state);
        }
        Err(RelicError::IOError(IOError::FileCantOpen))
    }

    // update head to be a detached commit
    pub fn set_head_detached(commit: Commit, state: &State) -> Option<RelicError> {
        let file_path = state.get_head_path();
        if !file_path.exists() {
            return Some(RelicError::IOError(IOError::FileNoExist));
        }

        if let Ok(()) = fs::write(file_path, HeadType::Detached(commit).to_string()) {
            return None;
        }

        Some(RelicError::BranchError(
            BranchError::DetachedCommitDoesntExist,
        ))
    }

    // update head to be a branch
    pub fn set_head_branch(name: String, state: &State) -> Option<RelicError> {
        // sets the branch as the main active branch

        if let Some(b) = Branch::construct_from_name(&name, state) {
            let file_path = state.get_head_path();
            if !file_path.exists() {
                return Some(RelicError::IOError(IOError::FileNoExist));
            }

            if let Ok(()) = fs::write(file_path, HeadType::Branch(b).to_string()) {
                return None;
            }
            return Some(RelicError::IOError(IOError::FileCantWrite));
        }

        Some(RelicError::BranchError(BranchError::BranchDoesntExist))
    }

    // #region CRUD
    // creates new branch inside branches
    pub fn instantiate(
        name: String,
        head: Option<Commit>,
        state: &State,
    ) -> Result<Option<Branch>, RelicError> {
        // creates new branches/{name}
        let file_path = state.branches_path.join(&name);
        if file_path.exists() {
            return Err(RelicError::BranchError(BranchError::BranchExists));
        }

        match head {
            Some(head) => {
                let b = Branch {
                    name,
                    head: head.oid,
                };
                if let Ok(_) = fs::write(file_path, head.oid.to_string()) {
                    // if let Ok(_) = fs::write(file_path, HeadType::Branch(b.clone()).to_string()) {
                    return Ok(Some(b));
                }
            }
            None => {
                if let Ok(_) = fs::write(file_path, "") {
                    return Ok(None);
                }
            }
        }

        Err(RelicError::Unimplemented)
    }

    // outright deletes the entire branch from branches
    pub fn delete(name: String, state: &State) -> Option<RelicError> {
        // deletes branches/{name}
        // head = remains same
        let file_path = state.branches_path.join(&name);
        if !file_path.exists() {
            return Some(RelicError::BranchError(BranchError::BranchDoesntExist));
        }

        if let Ok(()) = fs::remove_file(file_path) {
            return None;
        }

        Some(RelicError::IOError(IOError::FileCantDelete))
    }

    // updates the branch's commit to this new commit
    pub fn update_branch(
        name: String,
        new_commit: Commit,
        state: &State,
    ) -> Result<Branch, RelicError> {
        // branches/{name} = new_commit.oid
        // head = remains same
        let file_path = state.branches_path.join(&name);
        if !file_path.exists() {
            return Err(RelicError::BranchError(BranchError::BranchDoesntExist));
        }

        if let Ok(()) = fs::write(file_path, new_commit.oid.to_string()) {
            if let Some(b) = Branch::construct_from_name(&name, state) {
                return Ok(b);
            }
            return Err(RelicError::BranchError(BranchError::BranchDoesntExist));
        }

        Err(RelicError::IOError(IOError::FileCantOpen))
    }
    // #endregion

    fn format_branch_name(name: &str) -> String {
        format!("{DELIMITER}\0{}", name)
    }
}

#[derive(Clone)]
pub enum HeadType {
    Empty,
    Branch(Branch),
    Detached(Commit),
}
impl HeadType {
    fn deserialise(s: &str, state: &State) -> Result<HeadType, RelicError> {
        if let Some((delim, name)) = s.split_once("\0") {
            // follows format of
            // branch\0{branch name}
            if delim != DELIMITER {
                return Err(RelicError::ConfigurationIncorrect);
            }

            if let Some(b) = Branch::construct_from_name(name, state) {
                return Ok(HeadType::Branch(b));
            }
        } else {
            if s.is_empty() {
                return Ok(HeadType::Empty);
            } else {
                // is detached
                // s is raw oid
                if let Ok(Object::Commit(c)) =
                    ObjectID::from_string(s).construct(&state.get_sanctum_path())
                {
                    return Ok(HeadType::Detached(c));
                }
            }
        }
        Err(RelicError::ConfigurationIncorrect)
    }

    pub fn get_commit(self, sanctum_path: &PathBuf) -> Result<Option<Commit>, RelicError> {
        match self {
            HeadType::Branch(b) => b.get_commit(sanctum_path).and_then(|c| Ok(Some(c))),
            HeadType::Detached(c) => Ok(Some(c)),
            HeadType::Empty => Ok(None),
        }
    }
}
impl ToString for HeadType {
    fn to_string(&self) -> String {
        match self {
            HeadType::Branch(b) => Branch::format_branch_name(&b.name),
            HeadType::Detached(c) => c.oid.to_string(),
            HeadType::Empty => String::new(),
        }
    }
}
