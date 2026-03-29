use std::{collections::HashMap, fs, path::PathBuf};

use crate::core::{
    data::commit::Commit,
    error::{self, BranchError, IOError, RelicError},
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
    pub fn get_commit(&self, sanctum_path: &PathBuf) -> Result<Commit, RelicError> {
        if let Some(c) = self.head.construct_strict::<Commit>(sanctum_path) {
            return Ok(c);
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
    // input: name of branch + source
    // output: branch from branches/
    pub fn construct_from_name(name: &str, state: &State, source: &BranchSource) -> Option<Branch> {
        if let Ok(b) = Branch::construct_from_path(&source.path(state).join(name)) {
            return Some(b);
        }
        None
    }

    // input: path of branch
    // output: branch from branches/ (path will be branches/local or branches/upstream)
    pub fn construct_from_path(path: &PathBuf) -> Result<Branch, RelicError> {
        // EXPENSIVE!
        if let Some(f) = path.file_name() {
            let name = f.to_string_lossy().to_string();

            if let Ok(content) = fs::read_to_string(path) {
                if let Some(oid) = ObjectID::from_string(&content) {
                    return Ok(Branch { name, head: oid });
                }
                return Err(RelicError::ObjectID(error::ObjectID::InvalidID));
            }

            return Err(RelicError::IOError(IOError::FileCantOpen));
        }
        Err(RelicError::IOError(IOError::FileNoExist))
    }

    // input: name of branch
    // output: a vec of all sources (+branch) where the branch exists
    pub fn construct_from_name_all(state: &State, name: &str) -> Vec<(Branch, BranchSource)> {
        vec![BranchSource::Local, BranchSource::Upstream]
            .into_iter()
            .map(|s| (Branch::construct_from_name(name, state, &s), s))
            .filter(|i| i.0.is_some())
            // .map(|s| (s.1, s.0.unwrap()))
            .map(|s| (s.0.unwrap(), s.1))
            .collect()
    }

    // returns all branches in both local and upstream
    pub fn get_all_branches(
        state: &State,
    ) -> Result<Vec<(String, HashMap<BranchSource, Branch>)>, RelicError> {
        let Ok(locals) = fs::read_dir(state.get_local_branches_path()) else {
            return Err(RelicError::BranchError(BranchError::CantIterateBranches));
        };

        let Ok(upstreams) = fs::read_dir(state.get_upstream_branches_path()) else {
            return Err(RelicError::BranchError(BranchError::CantIterateBranches));
        };

        let mut result: HashMap<String, HashMap<BranchSource, Branch>> = HashMap::new();

        for file in locals {
            if let Ok(file) = file {
                let file_name = file.file_name();
                let file_type = file.file_type().unwrap();
                if file_type.is_file() {
                    let Some(b) = Branch::construct_from_name(
                        &file_name.to_string_lossy(),
                        state,
                        &BranchSource::Local,
                    ) else {
                        return Err(RelicError::BranchError(BranchError::BranchDoesntExist));
                    };
                    result.insert(b.name.clone(), HashMap::from([(BranchSource::Local, b)]));
                }
            }
        }

        for file in upstreams {
            if let Ok(file) = file {
                let file_name = file.file_name();
                let file_type = file.file_type().unwrap();
                if file_type.is_file() {
                    let Some(b) = Branch::construct_from_name(
                        &file_name.to_string_lossy(),
                        state,
                        &BranchSource::Upstream,
                    ) else {
                        return Err(RelicError::BranchError(BranchError::BranchDoesntExist));
                    };
                    result
                        .entry(b.name.clone())
                        .and_modify(|m| {
                            m.insert(BranchSource::Upstream, b.clone());
                        })
                        .or_insert(HashMap::from([(BranchSource::Upstream, b)]));
                }
            }
        }

        Ok(result.drain().collect())
    }

    pub fn get_head(state: &State, source: &BranchSource) -> Result<HeadType, RelicError> {
        let file_path = state.get_head_path();
        if !file_path.exists() {
            return Err(RelicError::IOError(IOError::FileNoExist));
        }

        if let Ok(s) = fs::read_to_string(file_path) {
            return HeadType::deserialise(&s, state, source);
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

    // update head to be a local branch
    // if branch doesnt exist locally,
    //      clone upstream into local
    // head = branch.name
    pub fn set_head_branch(name: String, state: &State) -> Option<RelicError> {
        // sets the branch as the main active branch

        if let None = Branch::construct_from_name(&name, state, &BranchSource::Local) {
            if let Some(e) = Branch::sync_branches(&name, state, &BranchSource::Upstream) {
                return Some(e);
            }
        }

        if let Some(b) = Branch::construct_from_name(&name, state, &BranchSource::Local) {
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

    // sync branch from upstream into local
    // branches/{opposite}/{name} = branches/{source}/{name}
    // sets opposite's branch to source's commit oid
    pub fn sync_branches(name: &str, state: &State, source: &BranchSource) -> Option<RelicError> {
        let source_path = source.path(state).join(name);
        let opposite_path = source.opposite().path(state).join(name);

        if source_path.exists() {
            if let Ok(_) = fs::copy(source_path, opposite_path) {
                return None;
            }

            return Some(RelicError::IOError(IOError::FileCantCopy));
        }

        Some(RelicError::BranchError(BranchError::BranchDoesntExist))
    }

    // #region CRUD
    // creates new branch inside local
    pub fn instantiate(
        name: String,
        head: Option<Commit>,
        state: &State,
        source: &BranchSource,
    ) -> Result<Option<Branch>, RelicError> {
        // creates new branches/{name}
        let file_path = source.path(state).join(&name);
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

    // outright deletes the entire branch from local branch
    pub fn delete(name: String, state: &State, source: BranchSource) -> Option<RelicError> {
        // TODO: figure out how to deal with upstream deletion
        // deletes branches/{name}
        // head = remains same
        let file_path = source.path(state).join(&name);
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
        name: &String,
        new_commit: &Commit,
        state: &State,
        source: &BranchSource,
    ) -> Result<Branch, RelicError> {
        // branches/{name} = new_commit.oid
        // head = remains same
        let file_path = source.path(state).join(&name);
        if !file_path.exists() {
            return Err(RelicError::BranchError(BranchError::BranchDoesntExist));
        }

        if let Ok(()) = fs::write(file_path, new_commit.oid.to_string()) {
            if let Some(b) = Branch::construct_from_name(&name, state, source) {
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

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum BranchSource {
    Local,
    Upstream,
}
impl BranchSource {
    fn path(&self, state: &State) -> PathBuf {
        match self {
            BranchSource::Local => state.get_local_branches_path(),
            BranchSource::Upstream => state.get_upstream_branches_path(),
        }
    }

    fn opposite(&self) -> BranchSource {
        match self {
            BranchSource::Local => BranchSource::Upstream,
            BranchSource::Upstream => BranchSource::Local,
        }
    }
}

#[derive(Clone)]
pub enum HeadType {
    Empty,
    Branch(Branch),
    Detached(Commit),
}
impl HeadType {
    fn deserialise(s: &str, state: &State, source: &BranchSource) -> Result<HeadType, RelicError> {
        if let Some((delim, name)) = s.split_once("\0") {
            // follows format of
            // branch\0{branch name}
            if delim != DELIMITER {
                return Err(RelicError::ConfigurationIncorrect);
            }

            if let Some(b) = Branch::construct_from_name(name, state, source) {
                return Ok(HeadType::Branch(b));
            }
        } else {
            if s.is_empty() {
                return Ok(HeadType::Empty);
            } else {
                // is detached
                // s is raw oid
                if let Some(Ok(Object::Commit(c))) = ObjectID::from_string(s)
                    .and_then(|o| Some(o.construct(&state.get_sanctum_path())))
                {
                    return Ok(HeadType::Detached(c));
                }
            }
        }
        Err(RelicError::ConfigurationIncorrect)
    }

    pub fn as_human_readable(&self) -> String {
        match self {
            HeadType::Branch(b) => format!("Branch ({})", b.name),
            HeadType::Detached(c) => format!("Detached ({})", c.oid.to_string()),
            HeadType::Empty => format!("Unset"),
        }
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
