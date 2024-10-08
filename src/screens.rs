use iced::{
    alignment,
    widget::{
        button, column, container, pick_list, row, scrollable, slider, svg, text, text_input,
        toggler, Column,
    },
    Alignment, Length,
};

use crate::{downloader, theme, widget::Renderer, LauncherState, Message, Screen};

pub fn get_screen_content(
    minelander: &super::Minelander,
) -> Column<'static, Message, super::theme::Theme, Renderer> {
    match minelander.screen {
        Screen::Main => {
            let (launch_text, launch_message) = match minelander.launcher.state {
                LauncherState::Idle => ("Launch", Option::Some(Message::Launch)),
                LauncherState::Launching(_) => ("Launching", Option::None),
                LauncherState::GettingLogs => ("Running", Option::None),
                LauncherState::Waiting => ("...", Option::None),
            };
            let launch_button = button(
                text(launch_text)
                    .size(35)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            .width(285)
            .height(60)
            .on_press_maybe(launch_message);

            let close_button = match minelander.launcher.state {
                LauncherState::GettingLogs => Some(
                    button(
                        text("Close game")
                            .size(15)
                            .horizontal_alignment(alignment::Horizontal::Center)
                            .vertical_alignment(alignment::Vertical::Center),
                    )
                    .width(189)
                    .height(35)
                    .on_press(Message::CloseGame)
                    .style(theme::Button::Red),
                ),
                _ => None,
            };

            let mut account_name_list = vec![];

            for i in &minelander.accounts {
                account_name_list.push(i.username.clone())
            }

            column![
                //mainscreen
                //title
                column![
                    text("Minelander").size(50),
                    text(format!("Hello {}!", minelander.current_account.username))
                        .style(theme::Text::Peach)
                        .size(18)
                ]
                .spacing(5),
                //username and version input
                row![
                    container(
                        column![
                            text("Account"),
                            pick_list(
                                account_name_list,
                                Some(minelander.current_account.username.clone()),
                                Message::CurrentAccountChanged
                            )
                            .placeholder("Select an Account")
                            .width(285)
                            .text_size(15),
                            text("Version:"),
                            pick_list(
                                minelander.all_versions.clone(),
                                Some(minelander.current_version.clone()),
                                Message::VersionChanged,
                            )
                            .placeholder("Select a version")
                            .width(285)
                            .text_size(15)
                        ]
                        .spacing(10)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(10),
                    container(
                        column![
                            button(
                                text("Open game folder")
                                    .horizontal_alignment(alignment::Horizontal::Center)
                            )
                            .width(200)
                            .height(32)
                            .on_press(Message::OpenGameFolder),
                            button(
                                text("Open current instance folder")
                                    .horizontal_alignment(alignment::Horizontal::Center)
                            )
                            .width(200)
                            .height(32)
                            .on_press(Message::OpenGameInstanceFolder),
                            button(
                                text("Logs").horizontal_alignment(alignment::Horizontal::Center)
                            )
                            .width(80)
                            .height(32)
                            .on_press(Message::ChangeScreen(Screen::Logs)),
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(20)
                ]
                .spacing(15),
                //launchbutton
                row![
                    column![launch_button,]
                        .push_maybe(close_button)
                        .spacing(15)
                        .align_items(Alignment::Center),
                    column![text(minelander.game_state_text.to_string())
                        .style(theme::Text::Green)
                        .size(15)
                        .height(40), text(minelander.game_state_text_2.to_string())
                        .style(theme::Text::Green)
                        .size(15)
                        .height(40)].spacing(5)
                ]
                .spacing(10),
            ]
            .spacing(25)
            .max_width(800)
        }

        Screen::Settings => column![
            // Settings screen
            //title
            text("Settings").size(50),
            //jvm and profile management
            row![
                container(
                    column![
                        column![
                            text("JVM:"),
                            pick_list(
                                minelander.java_name_list.clone(),
                                Some(minelander.current_java_name.clone()),
                                Message::JavaChanged
                            )
                            .width(250)
                            .text_size(25),
                            button(
                                text("Manage JVMs")
                                    .width(250)
                                    .horizontal_alignment(alignment::Horizontal::Center)
                            )
                            .height(32)
                            .on_press(Message::ChangeScreen(Screen::Java))
                        ]
                        .spacing(10)
                        .max_width(800),
                        column![
                            text("Game instance:"),
                            pick_list(
                                minelander.game_instance_list.clone(),
                                Some(minelander.current_game_instance.clone()),
                                Message::GameInstanceChanged
                            )
                            .width(250)
                            .text_size(25),
                            button(
                                text("Manage game instances")
                                    .width(250)
                                    .horizontal_alignment(alignment::Horizontal::Center)
                            )
                            .height(32)
                            .on_press(Message::ChangeScreen(Screen::GameInstance))
                        ]
                        .spacing(10)
                        .max_width(800)
                    ]
                    .spacing(10)
                )
                .style(theme::Container::BlackContainer)
                .padding(10),
                //memory, gamemode and showallversions option
                container(
                    column![
                        column![
                            text(format!("Allocated memory: {}GiB", minelander.game_ram))
                                .size(25)
                                .horizontal_alignment(alignment::Horizontal::Center),
                            slider(0.5..=16.0, minelander.game_ram, Message::GameRamChanged)
                                .width(250)
                                .step(0.5)
                        ],
                        button("Add wrapper commands")
                            .on_press(Message::ChangeScreen(Screen::ModifyCommand))
                    ]
                    .spacing(50)
                )
                .style(theme::Container::BlackContainer)
                .padding(10)
            ]
            .spacing(15),
        ]
        .spacing(15)
        .max_width(800),

        Screen::Installation => {
            let vanilla_pick_list = pick_list(
                minelander.vanilla_versions_download_list.clone(),
                Some(minelander.vanilla_version_to_download.clone()),
                Message::VanillaVersionToDownloadChanged,
            )
            .placeholder("Select a version")
            .width(250)
            .text_size(15);

            let fabric_pick_list = pick_list(
                minelander.fabric_versions_download_list.clone(),
                Some(minelander.fabric_version_to_download.clone()),
                Message::FabricVersionToDownloadChanged,
            )
            .placeholder("Select a version")
            .width(250)
            .text_size(15);

            let vanilla_button_message = match minelander.vanilla_version_to_download.is_empty() {
                true => None,
                false => Some(Message::InstallVersion(downloader::VersionType::Vanilla)),
            };

            let fabric_button_message = match minelander.fabric_version_to_download.is_empty() {
                true => None,
                false => Some(Message::InstallVersion(downloader::VersionType::Fabric)),
            };

            let vanilla_install_button = button(
                text("Install")
                    .size(20)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .width(250)
            .height(40)
            .on_press_maybe(vanilla_button_message)
            .style(theme::Button::Secondary);

            let fabric_install_button = button(
                text("Install")
                    .size(20)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .width(250)
            .height(40)
            .on_press_maybe(fabric_button_message)
            .style(theme::Button::Secondary);

            column![
                //installerscreen
                //title
                text("Version installer").size(50),
                row![
                    //vanilla
                    container(
                        column![
                            text("Vanilla"),
                            vanilla_pick_list,
                            //installbutton
                            vanilla_install_button
                        ]
                        .spacing(15)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(10),
                    //fabric
                    container(
                        column![
                            text("Fabric"),
                            fabric_pick_list,
                            //installbutton
                            fabric_install_button
                        ]
                        .spacing(15)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(10)
                ]
                .spacing(15),
                row![
                    toggler(
                        String::new(),
                        minelander.show_all_versions_in_download_list,
                        Message::ShowAllVersionsInDownloadListChanged
                    )
                    .width(Length::Shrink),
                    text("Show non-release versions")
                        .horizontal_alignment(alignment::Horizontal::Center)
                ]
                .spacing(10),
                text(&minelander.download_text).size(15)
            ]
            .spacing(15)
            .max_width(800)
        }

        Screen::Java => column![
            text("Manage JVMs")
                .size(50)
                .horizontal_alignment(alignment::Horizontal::Center),
            container(
                column![
                    text("New JVM"),
                    text("JVM name:"),
                    text_input("", &minelander.jvm_to_add_name)
                        .on_input(Message::JvmNameToAddChanged)
                        .size(25)
                        .width(250),
                    text("JVM path:"),
                    text_input("", &minelander.jvm_to_add_path)
                        .on_input(Message::JvmPathToAddChanged)
                        .size(25)
                        .width(250),
                    text("JVM flags:"),
                    text_input("", &minelander.jvm_to_add_flags)
                        .on_input(Message::JvmFlagsToAddChanged)
                        .size(25)
                        .width(250),
                    button(
                        text("Add")
                            .size(15)
                            .horizontal_alignment(alignment::Horizontal::Center)
                    )
                    .width(135)
                    .height(35)
                    .on_press(Message::JvmAdded)
                ]
                .spacing(5)
            )
            .style(theme::Container::BlackContainer)
            .padding(15)
        ]
        .spacing(15)
        .max_width(800),
        Screen::GameInstance => column![
            text("Manage game instances")
                .size(50)
                .horizontal_alignment(alignment::Horizontal::Center),
            container(
                column![
                    text("New game instance"),
                    text("Game instance name:"),
                    text_input("", &minelander.game_instance_to_add)
                        .on_input(Message::GameInstanceToAddChanged)
                        .size(25)
                        .width(250),
                    button(
                        text("Add")
                            .size(15)
                            .horizontal_alignment(alignment::Horizontal::Center)
                    )
                    .width(135)
                    .height(35)
                    .on_press(Message::GameInstanceAdded)
                ]
                .spacing(15)
            )
            .style(theme::Container::BlackContainer)
            .padding(15)
        ]
        .spacing(15)
        .max_width(800),

        Screen::Logs => column![
            text("Game logs").size(25),
            container(
                scrollable(text(minelander.logs.join("\n")).size(10))
                    .width(700.0)
                    .height(345.)
            )
            .style(theme::Container::BlackContainer)
            .padding(5)
        ]
        .spacing(10),
        Screen::ModifyCommand => column![
            text("Modify game command").size(50),
            text("Wraper commands").size(25),
            text_input(
                "Example: command1 command2",
                &minelander.game_wrapper_commands
            )
            .on_input(Message::GameWrapperCommandsChanged)
            .size(12),
            text("Enviroment variables").size(25),
            text_input(
                "Example: KEY1=value1 KEY2=value2",
                &minelander.game_enviroment_variables
            )
            .on_input(Message::GameEnviromentVariablesChanged)
            .size(12)
        ]
        .spacing(25),
        Screen::InfoAndUpdates => {
            let credits = format!("Minelander {} by jafkc2.", env!("CARGO_PKG_VERSION"));

            let update_text = if minelander.update_available {
                format!(
                    "Update available: {} -> {}",
                    env!("CARGO_PKG_VERSION"),
                    minelander.last_version
                )
            } else {
                minelander.last_version.clone()
            };

            let update_button_message = match minelander.update_available {
                true => Some(Message::Update),
                false => None,
            };

            column![
                text("Info and updates").size(50),
                row![
                    container(
                        column![
                            text("Updates").size(15),
                            text(update_text),
                            button("Update")
                                .on_press_maybe(update_button_message)
                                .style(theme::Button::Secondary)
                                .padding(5),
                            text(minelander.update_text.clone())
                        ]
                        .spacing(30)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(20),
                    container(
                        column![
                            text("Info").size(15),
                            text(credits),
                            row![button(text("Github repository").size(12))
                                .on_press(Message::OpenURL(
                                    "https://github.com/jafkc2/minelander".to_string()
                                ))
                                .padding(5)]
                            .spacing(10)
                        ]
                        .spacing(30)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(20)
                ]
                .spacing(15),
            ]
            .spacing(25)
        }
        Screen::Accounts => {
            let mut accounts_column = column![];
            for i in &minelander.accounts {
                let account_type = if i.microsoft { "Microsoft" } else { "Local" };

                let text_content = format!("{} ({})", i.username, account_type);

                let delete_button = button(svg(svg::Handle::from_memory(
                    include_bytes!("icons/trash.svg").as_slice(),
                )))
                .width(30)
                .height(30)
                .style(theme::Button::Red)
                .on_press(Message::RemoveAccount(i.username.clone()));

                accounts_column =
                    accounts_column.push(row![text(text_content), delete_button].spacing(10));
            }

            column![
                text("Accounts").size(50),
                row![
                    container(
                        column![text("Account list").size(30), accounts_column]
                            .spacing(15)
                            .width(250)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(15),
                    container(
                        column![
                            button("Add Microsoft account")
                                .on_press(Message::ChangeScreen(Screen::MicrosoftAccount)),
                            button("Add local account")
                                .on_press(Message::ChangeScreen(Screen::LocalAccount)),
                        ]
                        .spacing(15)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(10)
                ]
                .spacing(15)
            ]
            .spacing(25)
        }

        Screen::MicrosoftAccount => {
            column![
                text("Microsoft Account").size(50),
                container(column![
                    text("Open the page below in the browser and enter the code to authenticate."),

                    row![
                        text(format!("Page: {}", minelander.auth_code.link)),
                        button("Open in browser")
                            .on_press(Message::OpenURL(minelander.auth_code.link.clone()))
                    ]
                    .spacing(10),
                    row![
                        text(format!("Code: {}", minelander.auth_code.code)),
                        button("Copy to clipboard")
                            .on_press(Message::CopyToClipboard(minelander.auth_code.code.clone()))
                    ]
                    .spacing(10),
                    text(minelander.auth_status.clone())
                ].spacing(15))
                .style(theme::Container::BlackContainer)
                .padding(15)
            ].spacing(25)
        }
        Screen::LocalAccount => column![
            text("Local Account").size(50),
            container(
                column![
                    text("Account name"),
                    text_input("Account Name", &minelander.local_account_to_add_name)
                        .on_input(Message::LocalAccountNameChanged)
                        .width(285),
                    button("Add local account").on_press(Message::AddedLocalAccount),
                    text("Account name requires 3 to 16 characters.").size(12)
                ]
                .spacing(15)
            )
            .style(theme::Container::BlackContainer)
            .padding(15)
        ].spacing(25),
        Screen::GettingStarted => column![
            text("Getting started").size(50),
            container(
                column![
                    text("Hi, this is Minelander, an open source Minecraft Launcher! To start, let's add an account."),
                    row![button("Add Microsoft account")
                    .on_press(Message::ChangeScreen(Screen::MicrosoftAccount)),
                button("Add local account")
                    .on_press(Message::ChangeScreen(Screen::LocalAccount)),].spacing(10),
                ]
                .spacing(15)
            )
            .style(theme::Container::BlackContainer)
            .padding(15)
        ].spacing(25),
        Screen::GettingStarted2 => column![
            text("Getting started").size(50),
            container(
                column![
                    text(format!("Great, nice to meet you, {}. You don't have any Minecraft version installed, you can install a Minecraft version in the installation menu.", minelander.current_account.username)),
                    row![button("Installation menu")
                    .on_press(Message::ChangeScreen(Screen::Installation)),
                button("Skip")
                    .on_press(Message::ChangeScreen(Screen::Main)),].spacing(10),
                ]
                .spacing(15)
            )
            .style(theme::Container::BlackContainer)
            .padding(15)
        ].spacing(25),
    }
}
