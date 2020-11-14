use amethyst::ecs::Component;
use amethyst::ecs::DenseVecStorage;

pub struct Actor {
    pub actions: ActorActions,
    pub turning_to: f32,
}

bitflags! {
    pub struct ActorActions: u8 {
        const MOVEMENT_FORWARD   = 0b0000_0001;
        const MOVEMENT_BACKWARDS = 0b0000_0010;
        const MOVEMENT_LEFT      = 0b0000_0100;
        const MOVEMENT_RIGHT     = 0b0000_1000;
    }
}

impl Actor {
    pub const MOVEMENT_VELOCITY: f32 = 2.0;

    pub fn new() -> Self {
        return Actor {
            actions: ActorActions::empty(),
            turning_to: 0.0,
        };
    }
}

impl Component for Actor {
    type Storage = DenseVecStorage<Self>;
}
