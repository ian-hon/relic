use std::{collections::{HashMap, HashSet}, path::Path};

use strum_macros::Display;

use crate::{content::{Content, Directory}, state::State};

#[derive(Debug)]
pub struct Change {
    pub container_modifications: Vec<ContainerModification>,
    pub modifications: Vec<Modification>
}
impl Change {
    pub fn serialise_changes() -> String {
        "".to_string()
    }
    
    pub fn deserialise_changes() -> String {
        "".to_string()
    }

    pub fn get_change(upstream: String, current: String) -> Vec<Modification> {
        vec![]
    }

    pub fn get_change_container(upstream: &Directory, current: &Directory, path: &Path) -> (Vec<ContainerModification>, Vec<Modification>) {
        // assume that both current and previous have the same directory names
        // has to be bfs

        // initialise current state set
        let mut current_set = HashSet::new();
        let mut current_map = HashMap::new();
        for c in &current.content {
            match c {
                Content::Directory(d) => {
                    current_set.insert((d.name.clone(), false));
                    current_map.insert((d.name.clone(), false), c);
                },
                Content::File(f) => {
                    current_set.insert((f.name.clone(), true));
                    current_map.insert((f.name.clone(), true), c);
                }
            }
        }
        //

        // initialise upstream state set
        let mut upstream_set = HashSet::new();
        let mut upstream_map = HashMap::new();
        for c in &upstream.content {
            match c {
                Content::Directory(d) => {
                    upstream_set.insert((d.name.clone(), false));
                    upstream_map.insert((d.name.clone(), false), c);
                },
                Content::File(f) => {
                    upstream_set.insert((f.name.clone(), true));
                    upstream_map.insert((f.name.clone(), true), c);
                }
            }
        }
        //

        // use set differences to determine file and directory creation or deletion
        let deleted = upstream_set.difference(&current_set).map(|(n, t)| (n.to_string(), *t)).collect::<Vec<(String, bool)>>();
        let created = current_set.difference(&upstream_set).map(|(n, t)| (n.to_string(), *t)).collect::<Vec<(String, bool)>>();
        //

        // for all deleted files, log them
        // for all deleted directories, log them and do the same for all children
        let mut container_modifications = vec![];
        let mut modifications = vec![];
        for (dir_name, is_file) in deleted {
            if is_file {
                container_modifications.push(ContainerModification::DeleteFile(path.join(dir_name.clone()).to_string_lossy().to_string()));
            } else {
                let p = path.join(dir_name.clone());
                container_modifications.push(ContainerModification::DeleteDirectory(p.to_string_lossy().to_string()));
                // traverse all children, add them to result as well
                let (mut c_m, mut m) = Change::get_change_container(
                    match upstream_map.get(&(dir_name, false)).unwrap() {
                        Content::Directory(deleted_d) => { deleted_d },
                        _ => panic!()
                    },
                    &Directory::new(),
                    &p
                );
                container_modifications.append(&mut c_m);
                modifications.append(&mut m);
            }
        }
        //

        // (
        //     [
        //         DeleteDirectory("./lorem/ipsum"),
        //         DeleteFile("./lorem/ipsum/saturn"),
        //         DeleteFile("./lorem/ipsum/jupiter"),
        //         CreateFile("./lorem/pluto", "pluto")
        //     ],
        //     []
        // )

        // for all created files, log them
        // for all created directories, log them and do the same for all children
        for (dir_name, is_file) in created {
            if is_file {
                container_modifications.push(ContainerModification::CreateFile(path.join(dir_name.clone()).to_string_lossy().to_string(), dir_name));
                // Modification::Create here
            } else {
                let p = path.join(dir_name.clone());
                container_modifications.push(ContainerModification::CreateDirectory(p.to_string_lossy().to_string(), dir_name.clone()));

                let (mut c_m, mut m) = Change::get_change_container(
                    &Directory::new(),
                    match current_map.get(&(dir_name, false)).unwrap() {
                        Content::Directory(d) => d,
                        _ => panic!()
                    },
                    &p
                );
                container_modifications.append(&mut c_m);
                modifications.append(&mut m);
            }
        }

        for directory in current.content
            .iter()
            .filter_map(
                |c| match c {
                    Content::Directory(d) => Some(d),
                    _ => None
                }
            )
        {
            // get the matching upstream directory
            // if it doesnt exist, that means the content is new and can be ignored
            // we ignore it because we have already logged it in the section above
            let p = path.join(directory.name.clone());
            let upstream_directory = match upstream_map.get(&(directory.name.clone(), false)) {
                Some(u) => {
                    match u {
                        Content::Directory(u_d) => { u_d },
                        _ => panic!()
                    }
                },
                _ => { continue; }
            };
            //

            let (mut c_m, mut m) = Change::get_change_container(
                upstream_directory,
                directory,
                &p
            );
            container_modifications.append(&mut c_m);
            modifications.append(&mut m);
        }

        // for directory in current.content
        //     .iter()
        //     .filter_map(
        //         |c| match c {
        //             Content::Directory(d) => Some(d),
        //             _ => None
        //         }
        //     )
        // {
        //     let p = path.join(directory.name.clone()).to_string_lossy().to_string();
        //     let upstream_directory = match upstream_map.get(&(directory.name.clone(), false)) {
        //         Some(u) => {
        //             match u {
        //                 Content::Directory(u_d) => { u_d },
        //                 _ => panic!()
        //             }
        //         },
        //         None => {
        //             container_modifications.push(ContainerModification::CreateDirectory(p.clone(), directory.name.clone()));
        //             &Directory::new()
        //         }
        //     };

        //     match current_map.get(&(directory.name.clone(), false)).unwrap() {
        //         Content::Directory(dir) => {
        //             let (mut c_m, mut m) = Change::get_change_container(
        //                 upstream_directory,
        //                 dir,
        //                 Path::new(&p)
        //             );
        //             container_modifications.append(&mut c_m);
        //             modifications.append(&mut m);
        //         },
        //         _ => {}
        //     }
        // }

        (container_modifications, modifications)
    }
}

#[derive(Debug)]
pub enum Modification {
    // creation/deletion of lines in files
    Create(u128, Vec<u8>),
    Delete(u128)
}

#[derive(Debug)]
pub enum ContainerModification {
    // creation/deletion of files & folders
    // TODO : change so only path needed
    CreateDirectory(
        String, // path
        String // name
    ),
    DeleteDirectory(
        String // path
    ),

    CreateFile(
        String, // path
        String // name
    ),
    DeleteFile(
        String, // path
    )
}