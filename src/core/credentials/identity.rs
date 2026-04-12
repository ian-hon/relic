#[derive(Debug, Clone)]
pub struct Identity {
    username: String,
    email: String,
}

impl Identity {
    pub fn new<S>(name: S, email: S) -> Self
    where
        S: ToString,
    {
        Self {
            username: name.to_string(),
            email: email.to_string(),
        }
    }

    pub fn deserialise(s: &String) -> Option<Self> {
        // john_doe <johndoe@example.com>
        let sections = s.rsplit_once(" <")?;
        Some(Self::new(sections.0, &sections.1[..(sections.1.len() - 1)]))
    }

    pub fn serialise(&self) -> String {
        format!("{} <{}>", self.username, self.email)
    }
}
