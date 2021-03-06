mod confirm;
mod home;
mod new_game;
mod quit;

pub use self::confirm::*;
pub use self::home::*;
pub use self::new_game::*;
pub use self::quit::*;
use crate::utils;
use amethyst::ecs::prelude::Entity;
use amethyst::ecs::prelude::World;

pub trait UiState {
    fn set_visibility(&self, world: &mut World, is_visibility: bool) {
        if let Some(root) = self.get_root() {
            utils::set_entity_visibility(world, root, is_visibility);
        }

        if is_visibility {
            utils::ui::set_cursor_visibility(world, true);
        }
    }

    fn get_root(&self) -> Option<Entity>;
}
