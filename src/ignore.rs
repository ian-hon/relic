use std::collections::HashSet;

pub struct IgnoreSet {
    pub dir_ignore: HashSet<String>,
    pub file_ignore: HashSet<String>
}
impl IgnoreSet {
    pub fn create(content: String) -> IgnoreSet {
        let mut result = IgnoreSet {
            dir_ignore: HashSet::new(),
            file_ignore: HashSet::new()
        };

        // always ignore the .bones directory
        result.dir_ignore.insert(".bones".to_string());

        for line in content.split("\n") {
            if line.is_empty() {
                continue;
            }

            // doesnt take into account cases like 
            // some_directory// <- double slashes
            if line.ends_with("/") {
                let i = line[0..line.len() - 1].to_string();
                if i.is_empty() {
                    continue;
                }

                result.dir_ignore.insert(i);
            } else {
                result.dir_ignore.insert(line.to_string());
            }
        }

        result
    }
}
