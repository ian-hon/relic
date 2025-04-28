use std::{collections::{HashMap, HashSet}, path::Path};

use strum_macros::Display;

use crate::{content::{Content, Directory, File}, state::State};

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

    pub fn get_change(path: String, upstream_file: &File, current_file: &File) -> Vec<Modification> {
        let upstream = upstream_file.content.clone();
        let current = current_file.content.clone();

        if upstream == current { // compare hashes later
            return vec![];
        }

        let upstream_lines = upstream.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
        let current_lines = current.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();

        let upstream_length = upstream_lines.len();
        let current_length = current_lines.len();

        // current algorithm works but sometimes inaccurate when there are lines with equal content
        // note: as of 28.4.24, i have just learnt about Levenstein Distance
        // will update accordingly

        // parallel pointer incrementation
        let mut left = 0; // upstream index
        let mut right = 0; // current index
        let mut result = vec![];

        'outer: while ((left + 1) < upstream_length) && ((right + 1) < current_length) {
            if upstream_lines[left] == current_lines[right] {
                left += 1;
                right += 1;
                continue;
            }

            // println!("change at {left} : {right}");

            // iterate to eof
            let mut left_copy = left.clone();
            while (left_copy + 1) < upstream_length {
                left_copy += 1;

                // println!("lc : {left_copy} : {right}\n\t{}\n\t{}", upstream_lines[left_copy], current_lines[right]);

                if upstream_lines[left_copy] == current_lines[right] {
                    // its a delete event
                    // lines from left_copy - left are all deleted
                    // on the right side, the lines are
                    // right -> (right + (left_copy - left))
                    for r in right..(right + (left_copy - left)) {
                        result.push(Modification::Delete(path.clone(), current_file.name.clone(), r));
                    }

                    left = left_copy;

                    left += 1;
                    right += 1;
                    continue 'outer;
                }
            }

            let mut right_copy = right.clone();
            while (right_copy + 1) < current_length {
                right_copy += 1;

                if current_lines[right_copy] == upstream_lines[left] {
                    // its a create event
                    // lines from right -> right_copy are all created
                    for r in right..right_copy {
                        result.push(Modification::Create(path.clone(), current_file.name.clone(), r, current_lines[r].clone()));
                    }

                    right = right_copy;

                    left += 1;
                    right += 1;
                    continue 'outer;
                }
            }

            // reached eof
            // println!("cant find, must be a modified line");
            // result.push(Modification::Create(right, current_lines[right].clone()));
            // result.push(Modification::Delete(right));
            result.push(Modification::Update(path.clone(), current_file.name.clone(), right, current_lines[right].clone()));

            left += 1;
            right += 1;
        }

        // TODO : investigate whether these two loops can lead to an oob index
        for r in (if right == 0 { 0 } else { right + 1 })..(right + (upstream_length - left)) {
            // all of these are deleted
            // right..(right + (left_copy - left))
            result.push(Modification::Delete(path.clone(), current_file.name.clone(), r));
        }

        for r in (if right == 0 { 0 } else { right + 1 })..current_length {
            // all of these are newly created
            result.push(Modification::Create(path.clone(), current_file.name.clone(), r, current_lines[r].clone()));
        }

        result
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

        // for all created files, log them
        // for all created directories, log them and do the same for all children
        for (dir_name, is_file) in created {
            if is_file {
                let p = path.join(dir_name.clone()).to_string_lossy().to_string();
                container_modifications.push(ContainerModification::CreateFile(p.clone(), dir_name.clone()));
                // Modification::Create here
                modifications.append(&mut Change::get_change(p, &File::new(), match current_map.get(&(dir_name, true)).unwrap() {
                    Content::File(f) => { f },
                    _ => panic!()
                }))
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

        for content in &current.content {
            match content {
                Content::Directory(directory) => {
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
                },
                Content::File(f) => {
                    let upstream_file = match upstream_map.get(&(f.name.clone(), true)) 
                    {
                        Some(c) => match c {
                            Content::File(f) => f,
                            _ => panic!()
                        },
                        None => { continue; }
                    };

                    modifications.append(&mut Change::get_change(path.join(f.name.clone()).to_string_lossy().to_string(), &upstream_file, &f));
                }
            }
        }

        (container_modifications, modifications)
    }
}

#[derive(Debug)]
pub enum Modification {
    // creation/deletion of lines in files
    Create(
        String, // path
        String, // file name
        usize, // line
        String // text
    ),
    Delete(
        String, // path
        String, // file name
        usize // line
    ),
    Update(
        String, // path
        String, // file name
        usize, // line
        String // text
    )
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