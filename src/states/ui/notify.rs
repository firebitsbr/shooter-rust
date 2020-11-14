use crate::resources::UiTask;
use crate::resources::UiTaskResource;
use crate::states::ui::UiState;
use amethyst::ecs::prelude::Entity;
use amethyst::input::is_key_down;
use amethyst::prelude::*;
use amethyst::ui::UiEvent;
use amethyst::ui::UiEventType;
use amethyst::ui::UiFinder;
use amethyst::winit::VirtualKeyCode;

const ROOT_ID: &str = "notify";
const LABEL_TITLE_ID: &str = "notify.title";
const LABEL_DETAILS_ID: &str = "notify.details";
const BUTTON_1_ID: &str = "notify.button_1";
const BUTTON_2_ID: &str = "notify.button_2";
const BUTTON_1_TEXT_ID: &str = "notify.button_1_btn_txt";
const BUTTON_2_TEXT_ID: &str = "notify.button_2_btn_txt";

pub struct NotifyState {
    root: Option<Entity>,
    title: &'static str,
    details: String,
    button_1: Option<Entity>,
    button_2: Option<Entity>,
    button_1_text: String,
    button_2_text: String,
    on_button_1: Option<fn() -> SimpleTrans>,
    on_button_2: Option<fn() -> SimpleTrans>,
}

impl NotifyState {
    pub fn new(
        title: &'static str,
        details: Option<String>,
        button_1_text: Option<String>,
        button_2_text: Option<String>,
        on_button_1: Option<fn() -> SimpleTrans>,
        on_button_2: Option<fn() -> SimpleTrans>,
    ) -> Self {
        return Self {
            root: None,
            title,
            details: details.unwrap_or_else(String::new),
            button_1: None,
            button_2: None,
            button_1_text: button_1_text.unwrap_or_else(String::new),
            button_2_text: button_2_text.unwrap_or_else(String::new),
            on_button_1,
            on_button_2,
        };
    }

    pub fn new_as_confirm(
        title: &'static str,
        conformation_text: String,
        on_confirm: fn() -> SimpleTrans
    ) -> Self {
        return Self::new(
            title,
            None,
            Some(conformation_text),
            Some("Cancel".to_string()),
            Some(on_confirm),
            Some(|| Trans::Pop),
        );
    }

    pub fn new_as_error(title: &'static str, details: String) -> Self {
        return Self::new(
            title,
            Some(details),
            None,
            Some("OK".to_string()),
            None,
            Some(|| Trans::Pop),
        );
    }
}

impl SimpleState for NotifyState {
    fn on_start(&mut self, mut data: StateData<GameData>) {
        data.world.exec(|finder: UiFinder| {
            self.root = finder.find(ROOT_ID);
            self.button_1 = finder.find(BUTTON_1_ID);
            self.button_2 = finder.find(BUTTON_2_ID);
        });

        {
            let mut tasks = data.world.write_resource::<UiTaskResource>();
            tasks.insert(LABEL_TITLE_ID.to_string(), UiTask::SetText(self.title.to_string()));
            tasks.insert(LABEL_DETAILS_ID.to_string(), UiTask::SetText(self.details.to_string()));
            tasks.insert(BUTTON_1_TEXT_ID.to_string(), UiTask::SetText(self.button_1_text.clone()));
            tasks.insert(BUTTON_2_TEXT_ID.to_string(), UiTask::SetText(self.button_2_text.clone()));
        }

        self.set_visibility(&mut data.world, true);
    }

    fn on_pause(&mut self, mut data: StateData<GameData>) {
        self.set_visibility(&mut data.world, false);
    }

    fn on_resume(&mut self, mut data: StateData<GameData>) {
        self.set_visibility(&mut data.world, true);
    }

    fn on_stop(&mut self, mut data: StateData<GameData>) {
        self.button_1 = None;
        self.button_2 = None;
        self.set_visibility(&mut data.world, false);
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
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
                if Some(target) == self.button_1 {
                    if let Some(action) = self.on_button_1 {
                        return action();
                    }
                }

                if Some(target) == self.button_2 {
                    if let Some(action) = self.on_button_2 {
                        return action();
                    }
                }
            }
            _ => {}
        }

        return Trans::None;
    }
}

impl UiState for NotifyState {
    fn get_root(&self) -> Option<Entity> {
        return self.root;
    }
}
