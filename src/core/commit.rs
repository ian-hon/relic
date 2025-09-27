use crate::{core::modifications::Change, utils};

const PENDING_TAG: &str = "LOCAL";

#[derive(Debug, Clone)]
pub struct Commit {
    pub id: Option<u32>,
    pub message: String,
    pub description: String,
    pub change: Change,
    pub timestamp: u64,

    pub author: String,
}
impl Commit {
    pub fn header(&self) -> String {
        // "integrated backwards compatibility" (2025-5-26 16:30) (affected : change.rs, content.rs, ...)

        let file_names = self.change.get_affected_blobs();

        format!(
            "({}) \"{}\" (affected : {}{})",
            utils::into_human_readable(self.timestamp),
            self.message,
            file_names
                .iter()
                .take(5)
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            if file_names.len() > 5 { ", ..." } else { "" }
        )
    }

    pub fn serialise(&self) -> String {
        format!(
            "= {} {} {:?} {:?} {}\n{}",
            self.id
                .map_or(PENDING_TAG.to_string(), |i| format!("{:06x}", i).clone()),
            self.timestamp,
            urlencoding::encode(&self.message).to_string(),
            urlencoding::encode(&self.description).to_string(),
            self.author,
            self.change.serialise_changes()
        )
    }

    pub fn deserialise(s: String) -> Option<Commit> {
        // = LOCAL 1747682692319414000 "initial%20commit" "" no_one

        let lines = s.split("\n").collect::<Vec<&str>>();
        if lines.len() < 2 {
            // return None;
        }

        let metadata = lines[0].split(" ").collect::<Vec<&str>>();
        if metadata.len() != 6 {
            // return None;
        }

        let [_, status, time, message, description, author] = *metadata.as_slice() else {
            return None;
        };

        Some(Commit {
            id: status.parse::<u32>().map_or(None, |t| Some(t)),
            message: urlencoding::decode(&message[1..message.len() - 1].to_string())
                .unwrap()
                .to_string(),
            description: urlencoding::decode(&description[1..description.len() - 1].to_string())
                .unwrap()
                .to_string(),
            change: Change::deserialise_changes(lines[1..].join("\n")).unwrap_or(Change::empty()),
            timestamp: time.parse::<u64>().unwrap_or(0),
            author: author.to_string(),
        })
    }
}
