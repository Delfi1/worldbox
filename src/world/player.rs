use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

impl Player {
    pub fn new() -> Self {
        Self {}
    }
}