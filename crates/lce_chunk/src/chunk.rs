use bevy::prelude::*;

#[derive(Component, Debug, Default)]
#[require(Children, Transform)]
pub struct Chunk;
