use crate::states::game::GameType;
use crate::states::ui::UiState;
use crate::states::GameState;
use crate::utils;
use amethyst::ecs::prelude::Entity;
use amethyst::input::is_key_down;
use amethyst::prelude::*;
use amethyst::ui::UiEvent;
use amethyst::ui::UiEventType;
use amethyst::ui::UiFinder;
use amethyst::winit::VirtualKeyCode;
use std::net::SocketAddr;

const ROOT_ID: &str = "new_game";
const BUTTON_PLAY_SINGLE_ID: &str = "new_game.single";
const BUTTON_JOIN_ID: &str = "new_game.join";
const BUTTON_HOST_ID: &str = "new_game.host";
const BUTTON_BACK_ID: &str = "new_game.back";
const INPUT_IP_ID: &str = "new_game.ip";
const INPUT_PORT_ID: &str = "new_game.port";

pub struct NewGameState {
    root: Option<Entity>,
    button_play_single: Option<Entity>,
    button_join: Option<Entity>,
    button_host: Option<Entity>,
    button_back: Option<Entity>,
}

impl NewGameState {
    pub fn new() -> Self {
        return Self {
            root: None,
            button_play_single: None,
            button_join: None,
            button_host: None,
            button_back: None,
        };
    }

    fn parse_input_address(world: &World) -> Result<SocketAddr, String> {
        let ip;
        let port;

        if let Some(ip_temp) = utils::ui::fetch_text(world, INPUT_IP_ID) {
            ip = ip_temp;
        } else {
            return Err("No IP specified".to_string());
        }

        if let Some(port_temp) = utils::ui::fetch_text(world, INPUT_PORT_ID) {
            port = port_temp;
        } else {
            return Err("No port specified".to_string());
        }

        return format!("{}:{}", ip, port)
            .parse()
            .map_err(|_| "Wrong address".to_string());
    }

    fn parse_input_port(world: &World) -> Result<u16, String> {
        if let Some(port) = utils::ui::fetch_text(world, INPUT_PORT_ID) {
            return port.parse().map_err(|_| "Wrong port".to_string());
        } else {
            return Err("No prot specified".to_string());
        }
    }
}

impl SimpleState for NewGameState {
    fn on_start(&mut self, mut data: StateData<GameData>) {
        data.world.exec(|finder: UiFinder| {
            self.root = finder.find(ROOT_ID);
            self.button_play_single = finder.find(BUTTON_PLAY_SINGLE_ID);
            self.button_join = finder.find(BUTTON_JOIN_ID);
            self.button_host = finder.find(BUTTON_HOST_ID);
            self.button_back = finder.find(BUTTON_BACK_ID);
        });

        self.set_visibility(&mut data.world, true);
    }

    fn on_pause(&mut self, mut data: StateData<GameData>) {
        self.set_visibility(&mut data.world, false);
    }

    fn on_resume(&mut self, mut data: StateData<GameData>) {
        self.set_visibility(&mut data.world, true);
    }

    fn on_stop(&mut self, mut data: StateData<GameData>) {
        self.button_play_single = None;
        self.button_join = None;
        self.button_host = None;
        self.button_back = None;
        self.set_visibility(&mut data.world, false);
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Window(event) => {
                if is_key_down(&event, VirtualKeyCode::Escape) {
                    return Trans::Pop;
                }
            }
            StateEvent::Ui(UiEvent {
                event_type: UiEventType::Click,
                target,
            }) => {
                if Some(target) == self.button_play_single {
                    return Trans::Replace(Box::new(GameState::new(GameType::Single)));
                }

                if Some(target) == self.button_join {
                    match Self::parse_input_address(&data.world) {
                        Ok(address) => {
                            let game_type = GameType::Join(address);
                            return Trans::Replace(Box::new(GameState::new(game_type)));
                        }
                        Err(error) => {
                            log::error!("{}", error); // TODO: Show error page
                        }
                    }
                }

                if Some(target) == self.button_host {
                    match Self::parse_input_port(&data.world) {
                        Ok(port) => {
                            let game_type = GameType::Host(port);
                            return Trans::Replace(Box::new(GameState::new(game_type)));
                        }
                        Err(error) => {
                            log::error!("{}", error); // TODO: Show error page
                        }
                    }
                }

                if Some(target) == self.button_back {
                    return Trans::Pop;
                }
            }
            _ => {}
        }

        return Trans::None;
    }
}

impl UiState for NewGameState {
    fn get_root(&self) -> Option<Entity> {
        return self.root;
    }
}
