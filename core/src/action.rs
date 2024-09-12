use std::fmt::Display;

pub enum Action {
    Command { text: String },
}

impl Action {
    pub fn push_char(&mut self, c: char) {
        match self {
            Action::Command { text } => text.push(c),
        }
    }

    pub fn remove_char(&mut self) {
        match self {
            Action::Command { text } => {
                text.pop();
            }
        }
    }

    pub fn execute(&self) {
        todo!("Execute action");
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Command { text } => f.write_str(&format!(":{text}")),
        }
    }
}
