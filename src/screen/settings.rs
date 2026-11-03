use iced::{
    widget::{column, row, text, text_input},
    Element, Length, Task,
};

use crate::java::JavaType;
use crate::minecraft;
use crate::theme::Themes;
use crate::widgets::{
    button::{button, ButtonType},
    container::container,
    pick_list::pick_list,
};
use crate::{
    config::Config,
    widgets::{slider::slider, toggler::toggler},
};

pub struct Settings {
    ram_mib: u32,
    java: String,
    instance: String,
    theme: String,
    check_for_updates: bool,
    environment_variables: String,
    wrapper_commands: String,
    instances: Vec<String>,
    new_instance: String,
    instance_status: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    RamChanged(u32),
    JavaSelected(String),
    InstanceSelected(String),
    ThemeSelected(String),
    CheckForUpdatesChanged(bool),
    EnvironmentVariablesChanged(String),
    WrapperCommandsChanged(String),
    NewInstanceChanged(String),
    CreateInstance,
    DeleteInstance,
}

impl Settings {
    pub fn from_config(config: &Config) -> Self {
        let instances = minecraft::get_instances().unwrap_or_else(|error| {
            eprintln!("Failed to load instances: {error:#}");
            vec!["Default".to_string()]
        });
        let instance = if config.current_instance.is_empty() || config.current_instance == "Default"
        {
            String::new()
        } else if instances
            .iter()
            .any(|value| value == &config.current_instance)
        {
            config.current_instance.clone()
        } else {
            String::new()
        };

        Self {
            ram_mib: config.ram_mib.clamp(1, 32 * 1024),
            java: java_label(&config.java).to_string(),
            instance,
            theme: match &config.theme {
                Themes::CatppuccinMocha => "Catppuccin Mocha".to_string(),
                Themes::Nightfly => "Nightfly".to_string(),
                Themes::Dracula => "Dracula".to_string(),
                Themes::GruvboxDark => "Gruvbox Dark".to_string(),
            },
            check_for_updates: config.check_for_updates,
            environment_variables: config
                .environment_variables
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join("\n"),
            wrapper_commands: config.wrapper_commands.join("\n"),
            instances,
            new_instance: String::new(),
            instance_status: None,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let minecraft_section = container(
            column![
                text("Minecraft").size(22),
                column![
                    text(format!("RAM allocated to the game: {} MiB", self.ram_mib)),
                    slider(1..=32 * 1024, self.ram_mib, Message::RamChanged)
                        .step(1u32)
                        .width(Length::Fill),
                ]
                .spacing(8),
                pick_list(
                    "Java version:",
                    [
                        "Automatic".to_string(),
                        "Java 8".to_string(),
                        "Java 17".to_string(),
                        "Java 21".to_string(),
                        "Java 25".to_string(),
                        "Custom".to_string(),
                    ],
                    Some(self.java.clone()),
                    Message::JavaSelected,
                ),
                pick_list(
                    "Game instance:",
                    std::iter::once("Default".to_string())
                        .chain(self.instances.clone())
                        .collect::<Vec<_>>(),
                    Some(if self.instance.is_empty() {
                        "Default".to_string()
                    } else {
                        self.instance.clone()
                    }),
                    Message::InstanceSelected,
                ),
                row![
                    text_input("New instance name", &self.new_instance)
                        .on_input(Message::NewInstanceChanged)
                        .width(Length::Fill),
                    button(text("Create"), None, ButtonType::Primary)
                        .on_press(Message::CreateInstance),
                    button(text("Delete"), None, ButtonType::Secondary).on_press_maybe(
                        (!self.instance.is_empty()).then_some(Message::DeleteInstance)
                    ),
                ]
                .spacing(8),
                self.instance_status
                    .as_deref()
                    .map(text)
                    .unwrap_or_else(|| text("")),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );

        let minelander_section = container(
            column![
                text("Minelander").size(22),
                pick_list(
                    "Theme:",
                    ["Catppuccin Mocha".to_string(), "Nightfly".to_string(), "Dracula".to_string(), "Gruvbox Dark".to_string()],
                    Some(self.theme.clone()),
                    Message::ThemeSelected,
                ),
                toggler(self.check_for_updates)
                    .label("Check for updates automatically")
                    .on_toggle(Message::CheckForUpdatesChanged),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );

        let system_section = container(
            column![
                text("System").size(22),
                column![
                    text("Environment variables"),
                    text_input(
                        "Variables used in the Minecraft command",
                        &self.environment_variables,
                    )
                    .on_input(Message::EnvironmentVariablesChanged)
                    .width(Length::Fill),
                ]
                .spacing(8),
                column![
                    text("Wrapper commands"),
                    text_input(
                        "Commands used to wrap the Minecraft command",
                        &self.wrapper_commands,
                    )
                    .on_input(Message::WrapperCommandsChanged)
                    .width(Length::Fill),
                ]
                .spacing(8),
            ]
            .spacing(14)
            .padding(15)
            .width(Length::Fill),
        );

        column![
            text("Settings").size(50).height(50),
            text("Configure your game and launcher preferences.")
                .size(20)
                .height(50),
            minecraft_section.max_width(700),
            minelander_section.max_width(700),
            system_section.max_width(700),
        ]
        .width(Length::FillPortion(4))
        .spacing(14)
        .padding(6)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RamChanged(value) => self.ram_mib = value,
            Message::JavaSelected(value) => self.java = value,
            Message::InstanceSelected(value) => {
                self.instance = if value == "Default" {
                    String::new()
                } else {
                    value
                };
            }
            Message::ThemeSelected(value) => self.theme = value,
            Message::CheckForUpdatesChanged(value) => self.check_for_updates = value,
            Message::EnvironmentVariablesChanged(value) => self.environment_variables = value,
            Message::WrapperCommandsChanged(value) => self.wrapper_commands = value,
            Message::NewInstanceChanged(value) => self.new_instance = value,
            Message::CreateInstance => match minecraft::create_instance(&self.new_instance) {
                Ok(()) => {
                    self.instances = minecraft::get_instances().unwrap_or_else(|error| {
                        eprintln!("Failed to reload instances: {error:#}");
                        self.instances.clone()
                    });
                    self.instance = self.new_instance.trim().to_string();
                    self.new_instance.clear();
                    self.instance_status = Some("Instance created.".to_string());
                }
                Err(error) => {
                    self.instance_status = Some(format!("Failed to create instance: {error}"));
                }
            },
            Message::DeleteInstance => match minecraft::delete_instance(&self.instance) {
                Ok(()) => {
                    self.instances = minecraft::get_instances().unwrap_or_else(|error| {
                        eprintln!("Failed to reload instances: {error:#}");
                        self.instances.clone()
                    });
                    self.instance.clear();
                    self.instance_status = Some("Instance deleted.".to_string());
                }
                Err(error) => {
                    self.instance_status = Some(format!("Failed to delete instance: {error}"));
                }
            },
        }
        Task::none()
    }

    pub fn apply_to_config(&self, config: &mut Config) {
        config.ram_mib = self.ram_mib;
        config.java = match self.java.as_str() {
            "Automatic" => JavaType::Automatic,
            "Java 8" => JavaType::Java8,
            "Java 17" => JavaType::Java17,
            "Java 21" => JavaType::Java21,
            "Java 25" => JavaType::Java25,
            "Custom" => JavaType::Custom,
            _ => config.java.clone(),
        };
        config.current_instance = self.instance.clone();
        config.theme = match self.theme.as_str() {
            "Catppuccin Mocha" => Themes::CatppuccinMocha,
            "Nightfly" => Themes::Nightfly,
            "Dracula" => Themes::Dracula,
            "Gruvbox Dark" => Themes::GruvboxDark,
            _ => Themes::CatppuccinMocha
        };
        config.check_for_updates = self.check_for_updates;
        config.environment_variables = self
            .environment_variables
            .lines()
            .filter_map(|line| {
                let (key, value) = line.split_once('=')?;
                Some((key.trim().to_string(), value.trim().to_string()))
            })
            .collect();
        config.wrapper_commands = self
            .wrapper_commands
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToString::to_string)
            .collect();
    }
}

fn java_label(java: &JavaType) -> &'static str {
    match java {
        JavaType::Automatic => "Automatic",
        JavaType::Java8 => "Java 8",
        JavaType::Java17 => "Java 17",
        JavaType::Java21 => "Java 21",
        JavaType::Java25 => "Java 25",
        JavaType::Custom => "Custom",
    }
}
