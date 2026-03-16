use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct MessageLog {
    pub messages: Vec<String>,
}

impl MessageLog {
    pub fn add(&mut self, msg: impl Into<String>) {
        self.messages.push(msg.into());
    }
}
