#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use self::widget::Element;
use iced::{
    alignment, executor,
    widget::{
        button, column, container, pick_list, row, scrollable, slider, svg, text, text_input,
        toggler, tooltip, Button,
    },
    window, Alignment, Application, Command, Length, Settings, Subscription,
};
use launcher::get_minecraft_dir;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::fs::File;
use std::io::Read;
use std::{collections::HashMap, env::set_current_dir};
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
use widget::Renderer;

mod downloader;
mod launcher;
mod theme;

fn main() -> iced::Result {
    if !Path::new(&get_minecraft_dir()).exists(){
        match fs::create_dir_all(&get_minecraft_dir()){
            Ok(_) => println!("Minecraft directory was created."),
            Err(e) => println!("Failed to create Minecraft directory: {e}"),
        };
    }


    let icon = include_bytes!("icons/siglauncher.png");

    Siglauncher::run(Settings {
        window: window::Settings {
            size: (800, 450),
            resizable: false,
            icon: Some(window::icon::from_file_data(icon, None).unwrap()),

            ..window::Settings::default()
        },

        ..Settings::default()
    })
}

#[derive(Default)]
struct Siglauncher {
    screen: Screen,
    launcher: Launcher,
    downloaders: Vec<Downloader>,
    logs: Vec<String>,

    username: String,
    current_version: String,
    game_state_text: String,

    game_ram: f64,
    current_java_name: String,
    current_java: Java,
    current_game_profile: String,
    game_wrapper_commands: String,
    game_enviroment_variables: String,
    show_all_versions_in_download_list: bool,

    all_versions: Vec<String>,
    java_name_list: Vec<String>,
    game_profile_list: Vec<String>,
    vanilla_versions_download_list: Vec<String>,
    fabric_versions_download_list: Vec<String>,
    vanilla_version_to_download: String,
    fabric_version_to_download: String,
    download_text: String,
    files_download_number: i32,

    needs_to_update_download_list: bool,

    jvm_to_add_name: String,
    jvm_to_add_path: String,
    jvm_to_add_flags: String,

    game_profile_to_add: String,

    restrict_launch: bool,
    java_download_size: u8,
}

#[derive(PartialEq, Debug, Clone, Default)]
enum Screen {
    #[default]
    Main,
    Options,
    Installation,
    Java,
    GameProfile,
    Logs,
    ModifyCommand,
}
#[derive(Debug, Clone)]
enum Message {
    LoadVersionList(Vec<String>),

    Launch,
    ManageGameInfo((usize, launcher::Progress)),

    UsernameChanged(String),
    VersionChanged(String),

    JavaChanged(String),
    GameProfileChanged(String),
    GameRamChanged(f64),
    GameWrapperCommandsChanged(String),
    GameEnviromentVariablesChanged(String),
    ShowAllVersionsInDownloadListChanged(bool),

    GotDownloadList(Result<Vec<Vec<String>>, String>),
    VanillaVersionToDownloadChanged(String),
    FabricVersionToDownloadChanged(String),
    InstallVersion(downloader::VersionType),
    ManageDownload((usize, downloader::Progress)),
    VanillaJson(Value),

    OpenGameFolder,
    OpenGameProfileFolder,

    ChangeScreen(Screen),

    JvmNameToAddChanged(String),
    JvmPathToAddChanged(String),
    JvmFlagsToAddChanged(String),
    JvmAdded,

    GameProfileToAddChanged(String),
    GameProfileAdded,

    GithubButtonPressed,
}

impl Siglauncher {
    pub fn launch(&mut self) {
        if updateusersettingsfile(self.username.clone(), self.current_version.clone()).is_err() {
            println!("Failed to save user settings!")
        };

        let wrapper_commands_vec: Vec<String> = if !self.game_wrapper_commands.is_empty() {
            self.game_wrapper_commands
                .split(' ')
                .map(|s| s.to_owned())
                .collect()
        } else {
            Vec::new()
        };

        let enviroment_variables_hash_map = if !self.game_enviroment_variables.is_empty() {
            let mut hashmap = HashMap::new();
            let splitted_env_vars = self.game_enviroment_variables.split(' ');

            for i in splitted_env_vars {
                if i.contains('=') {
                    let splitted_i: Vec<String> = i.split('=').map(|i| i.to_owned()).collect();
                    hashmap.insert(splitted_i[0].clone(), splitted_i[1].clone());
                }
            }

            hashmap
        } else {
            HashMap::new()
        };

        let game_settings = launcher::GameSettings {
            username: self.username.clone(),
            game_version: self.current_version.clone(),
            jvm: self.current_java.path.clone(),
            jvmargs: self
                .current_java
                .flags
                .split(' ')
                .map(|s| s.to_owned())
                .collect(),
            ram: self.game_ram,
            game_wrapper_commands: wrapper_commands_vec,
            game_directory: self.current_game_profile.clone(),
            autojava: self.current_java_name == "Automatic",
            enviroment_variables: enviroment_variables_hash_map,
        };
        self.launcher.start(game_settings);
        self.logs.clear();
    }
}

impl Application for Siglauncher {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = theme::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        // Configuration file
        checksettingsfile();

        let mut file = File::open(get_config_file_path()).unwrap();
        let mut fcontent = String::new();
        file.read_to_string(&mut fcontent).unwrap();
        let content = serde_json::from_str(&fcontent);
        let p: Value = content.unwrap();
        // Configuration file

        // Get Java info
        let mut currentjava = Java {
            name: String::new(),
            path: String::new(),
            flags: String::new(),
        };
        let mut jvmnames: Vec<String> = Vec::new();
        if let Some(jvms) = p["JVMs"].as_array() {
            for jvm in jvms {
                jvmnames.push(jvm["name"].as_str().unwrap().to_owned());
                if jvm["name"] == p["current_java_name"] {
                    currentjava.name = jvm["name"].as_str().unwrap().to_owned();
                    currentjava.path = jvm["path"].as_str().unwrap().to_owned();
                    currentjava.flags = jvm["flags"].as_str().unwrap().to_owned();
                }
            }
        }
        // Get Java info

        // Game profile folder creation if it doesn't exist
        let mc_dir = launcher::get_minecraft_dir();
        let game_profile_folder_path = format!("{}/siglauncher_profiles", mc_dir);
        if !Path::new(&game_profile_folder_path).exists() {
            match fs::create_dir_all(&game_profile_folder_path) {
                Ok(_) => println!("Created game profiles folder"),
                Err(e) => println!("Failed to create game profiles folder: {}", e),
            }
        }
        // Game profile folder creation if it doesn't exist

        // Some modified versions need this file
        if !Path::new(&format!("{}/launcher_profiles.json", mc_dir)).exists() {
            match File::create(format!("{}/launcher_profiles.json", mc_dir)) {
                Ok(mut file) => {
                    println!("Created launcher_profiles.json");
                    match file.write_all("{\"profiles\":{}}".as_bytes()) {
                        Ok(_) => println!("Wrote data to launcher_profiles.json"),
                        Err(e) => println!("Failed to write data to launcher_profiles.json: {}", e),
                    }
                }
                Err(d) => println!("Failed to create launcher_profiles.json: {}.", d),
            }
        }
        // Some modified versions need this file

        // Get game profiles
        let entries = fs::read_dir(game_profile_folder_path).unwrap();
        let mut new_game_profile_list = entries
            .filter_map(|entry| {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    Some(path.file_name().unwrap().to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        new_game_profile_list.push("Default".to_string());

        (
            Siglauncher {
                screen: Screen::Main,
                username: p["username"].as_str().unwrap().to_owned(),
                current_version: p["current_version"].as_str().unwrap().to_owned(),
                game_ram: p["game_ram"].as_f64().unwrap(),
                current_java_name: currentjava.name.clone(),
                current_java: currentjava,
                current_game_profile: p["current_game_profile"].as_str().unwrap().to_owned(),
                game_wrapper_commands: p["game_wrapper_commands"].as_str().unwrap().to_owned(),
                game_enviroment_variables: p["game_enviroment_variables"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
                show_all_versions_in_download_list: p["show_all_versions"].as_bool().unwrap(),
                java_name_list: jvmnames,
                game_profile_list: new_game_profile_list,
                needs_to_update_download_list: true,
                ..Default::default()
            },
            Command::perform(launcher::getinstalledversions(), Message::LoadVersionList),
        )
    }

    fn title(&self) -> String {
        String::from("Siglauncher 0.5.3")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Launch => {
                if !self.restrict_launch
                    && !self.current_version.is_empty()
                    && !self.username.is_empty()
                {
                    self.launch();
                }
                Command::none()
            }
            Message::ManageGameInfo((_id, progress)) => {
                match progress {
                    launcher::Progress::Checked(missing) => {
                        if let Some(missing) = missing {
                            match missing {
                                launcher::Missing::Java8 => {
                                    self.launcher.state = LauncherState::Waiting;
                                    self.downloaders.push(Downloader {
                                        state: DownloaderState::Idle,
                                        id: self.downloaders.len(),
                                    });
                                    let index = self.downloaders.len() - 1;
                                    self.downloaders[index].start_java(downloader::Java::J8)
                                }
                                launcher::Missing::Java17 => {
                                    self.launcher.state = LauncherState::Waiting;
                                    self.downloaders.push(Downloader {
                                        state: DownloaderState::Idle,
                                        id: self.downloaders.len(),
                                    });
                                    let index = self.downloaders.len() - 1;
                                    self.downloaders[index].start_java(downloader::Java::J17)
                                }
                                launcher::Missing::VersionFiles(vec) => {
                                    self.game_state_text =
                                        String::from("Found missing files. Starting download.");
                                    self.launcher.state = LauncherState::Waiting;
                                    self.downloaders.push(Downloader {
                                        state: DownloaderState::Idle,
                                        id: self.downloaders.len(),
                                    });
                                    let index = self.downloaders.len() - 1;
                                    self.downloaders[index].start_missing_files(vec)
                                }
                                launcher::Missing::VanillaJson(ver, folder) => {
                                    self.launcher.state = LauncherState::Waiting;
                                    self.game_state_text =
                                        String::from("Downloading required json");
                                    return Command::perform(
                                        async move {
                                            match downloader::downloadversionjson(
                                                &downloader::VersionType::Vanilla,
                                                &ver,
                                                &folder,
                                                &reqwest::Client::new(),
                                            )
                                            .await
                                            {
                                                Ok(ok) => ok,
                                                Err(_) => Value::Null,
                                            }
                                        },
                                        Message::VanillaJson,
                                    );
                                }
                            }
                        }
                    }
                    launcher::Progress::Started => {
                        self.launcher.state = LauncherState::GettingLogs;
                        self.game_state_text = String::new()
                    }
                    launcher::Progress::GotLog(log) => {
                        self.logs.push(log);
                    }
                    launcher::Progress::Finished => {
                        self.game_state_text = String::new();
                        self.launcher.state = LauncherState::Idle;
                    }
                    launcher::Progress::Errored(e) => {
                        self.game_state_text = e;
                        self.launcher.state = LauncherState::Idle;
                    }
                }

                Command::none()
            }
            Message::UsernameChanged(new_username) => {
                if new_username.len() < 16 {
                    self.username = new_username
                }

                Command::none()
            }
            Message::VersionChanged(new_version) => {
                self.current_version = new_version;
                Command::none()
            }
            Message::ChangeScreen(new_screen) => {
                if self.screen == Screen::Options {
                    updatesettingsfile(
                        self.game_ram,
                        self.current_java_name.clone(),
                        self.current_game_profile.clone(),
                        self.game_wrapper_commands.clone(),
                        self.show_all_versions_in_download_list,
                    )
                    .unwrap();
                }

                self.screen = new_screen.clone();

                if new_screen == Screen::Main {
                    return Command::perform(
                        launcher::getinstalledversions(),
                        Message::LoadVersionList,
                    );
                } else if new_screen == Screen::Installation
                    && (!self.vanilla_versions_download_list.is_empty()
                        || !self.fabric_versions_download_list.is_empty()
                        || self.needs_to_update_download_list)
                {
                    let show_all_versions = self.show_all_versions_in_download_list;
                    return Command::perform(
                        async move {
                            downloader::get_downloadable_version_list(show_all_versions).await
                        },
                        Message::GotDownloadList,
                    );
                }

                Command::none()
            }
            Message::OpenGameFolder => {
                open::that(launcher::get_minecraft_dir()).unwrap();
                Command::none()
            }
            Message::OpenGameProfileFolder => {
                if self.current_game_profile == "Default" {
                    open::that(launcher::get_minecraft_dir()).unwrap();
                } else {
                    open::that(format!(
                        "{}/siglauncher_profiles/{}",
                        launcher::get_minecraft_dir(),
                        self.current_game_profile
                    ))
                    .unwrap();
                }
                Command::none()
            }
            Message::JavaChanged(selected_jvm_name) => {
                set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

                let mut file = File::open(get_config_file_path()).unwrap();
                let mut fcontent = String::new();
                file.read_to_string(&mut fcontent).unwrap();
                let content = serde_json::from_str(&fcontent);
                let p: Value = content.unwrap();

                let mut newjvm: Vec<String> = Vec::new();

                let mut newjvmname: String = String::new();

                if let Some(jvms) = p["JVMs"].as_array() {
                    for jvm in jvms {
                        if jvm["name"] == selected_jvm_name {
                            newjvm.push(jvm["name"].as_str().unwrap().to_owned());
                            newjvm.push(jvm["path"].as_str().unwrap().to_owned());
                            newjvm.push(jvm["flags"].as_str().unwrap().to_owned());

                            newjvmname = jvm["name"].as_str().unwrap().to_owned();
                        }
                    }
                }

                self.current_java_name = newjvmname;
                self.current_java = Java {
                    name: newjvm[0].clone(),
                    path: newjvm[1].clone(),
                    flags: newjvm[2].clone(),
                };
                Command::none()
            }
            Message::GameProfileChanged(new_game_profile) => {
                self.current_game_profile = new_game_profile;
                Command::none()
            }
            Message::GameRamChanged(new_ram) => {
                self.game_ram = new_ram;
                Command::none()
            }
            Message::GameWrapperCommandsChanged(s) => {
                self.game_wrapper_commands = s;
                Command::none()
            }
            Message::ShowAllVersionsInDownloadListChanged(bool) => {
                self.needs_to_update_download_list = true;
                self.show_all_versions_in_download_list = bool;
                Command::none()
            }
            Message::GotDownloadList(result) => {
                match result {
                    Ok(list) => {
                        self.needs_to_update_download_list = false;
                        if !list.is_empty() {
                            self.vanilla_versions_download_list.clear();
                            self.fabric_versions_download_list.clear();
                            for i in &list[0] {
                                let ii = i;
                                self.vanilla_versions_download_list.push(ii.to_string());
                            }
                            for i in &list[1] {
                                let ii = i;
                                self.fabric_versions_download_list.push(ii.to_string());
                            }
                        }
                    }
                    Err(err) => self.download_text = err,
                }

                Command::none()
            }
            Message::VanillaVersionToDownloadChanged(new_version) => {
                self.vanilla_version_to_download = new_version;
                Command::none()
            }
            Message::FabricVersionToDownloadChanged(new_version) => {
                self.fabric_version_to_download = new_version;
                Command::none()
            }
            Message::InstallVersion(ver_type) => {
                let version = match ver_type {
                    downloader::VersionType::Vanilla => self.vanilla_version_to_download.clone(),
                    downloader::VersionType::Fabric => self.fabric_version_to_download.clone(),
                };
                self.downloaders
                    .push(Downloader::new(self.downloaders.len()));

                let index = self.downloaders.len() - 1;
                self.downloaders[index].start(version, ver_type);
                Command::none()
            }
            Message::JvmNameToAddChanged(name) => {
                self.jvm_to_add_name = name;
                Command::none()
            }
            Message::JvmPathToAddChanged(path) => {
                self.jvm_to_add_path = path;
                Command::none()
            }
            Message::JvmFlagsToAddChanged(flags) => {
                self.jvm_to_add_flags = flags;
                Command::none()
            }
            Message::JvmAdded => {
                if !self.jvm_to_add_name.is_empty() && !self.jvm_to_add_path.is_empty() {
                    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

                    let mut data = getjson(get_config_file_path());

                    let new_jvm = Java {
                        name: self.jvm_to_add_name.clone(),
                        path: self.jvm_to_add_path.clone(),
                        flags: self.jvm_to_add_flags.clone(),
                    };
                    if let Value::Array(arr) = &mut data["JVMs"] {
                        arr.push(serde_json::json!(new_jvm));
                        data["JVMs"] = serde_json::json!(arr)
                    }

                    let mut updatedjvmlist = Vec::new();

                    if let Some(jvms) = data["JVMs"].as_array() {
                        for jvm in jvms {
                            updatedjvmlist.push(jvm["name"].as_str().unwrap().to_owned());
                        }
                    }
                    self.java_name_list = updatedjvmlist;
                    let serialized = serde_json::to_string_pretty(&data).unwrap();

                    let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(get_config_file_path())
                        .unwrap();
                    file.write_all(serialized.as_bytes()).unwrap();
                    self.screen = Screen::Options;
                }
                Command::none()
            }
            Message::GameProfileToAddChanged(game_prof) => {
                self.game_profile_to_add = game_prof;
                Command::none()
            }
            Message::GameProfileAdded => {
                if !self.game_profile_to_add.is_empty() {
                    fs::create_dir_all(format!(
                        "{}/siglauncher_profiles/{}",
                        launcher::get_minecraft_dir(),
                        self.game_profile_to_add
                    ))
                    .expect("Failed to create directory!");

                    let entries = fs::read_dir(format!(
                        "{}/siglauncher_profiles",
                        launcher::get_minecraft_dir()
                    ))
                    .unwrap();

                    let mut new_game_profile_list = entries
                        .filter_map(|entry| {
                            let path = entry.unwrap().path();
                            if path.is_dir() {
                                Some(path.file_name().unwrap().to_string_lossy().to_string())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    new_game_profile_list.push("Default".to_string());

                    self.game_profile_list = new_game_profile_list;

                    self.screen = Screen::Options;
                }
                Command::none()
            }
            Message::GithubButtonPressed => {
                open::that("https://github.com/JafKc/siglauncher").unwrap();
                Command::none()
            }

            Message::ManageDownload((id, progress)) => {
                match progress {
                    downloader::Progress::GotDownloadList(file_number) => {
                        self.download_text =
                            format!("Downloaded 0 from {} files. (0%)", file_number);
                        self.files_download_number = file_number;
                    }
                    downloader::Progress::Downloaded(remaining_files_number) => {
                        let downloaded_files = self.files_download_number - remaining_files_number;

                        let percentage = (downloaded_files as f32
                            / self.files_download_number as f32
                            * 100.0) as i32;

                        self.download_text = format!(
                            "Downloaded {} from {} files. ({}%)",
                            downloaded_files, self.files_download_number, percentage
                        );
                    }
                    downloader::Progress::Finished => {
                        self.download_text = String::from("Version installed successfully.");
                        for (index, downloader) in self.downloaders.iter().enumerate() {
                            if downloader.id == id {
                                self.downloaders.remove(index);
                                break;
                            }
                        }
                    }
                    downloader::Progress::Errored(error) => {
                        self.download_text = format!("Failed to install: {error}");
                        for (index, downloader) in self.downloaders.iter().enumerate() {
                            if downloader.id == id {
                                self.downloaders.remove(index);
                                break;
                            }
                        }
                    }
                    downloader::Progress::StartedJavaDownload(size) => {
                        self.restrict_launch = true;
                        self.game_state_text = format!("Downloading java. 0 / {size} MiB (0%)");
                        self.java_download_size = size;
                    }
                    downloader::Progress::JavaDownloadProgressed(downloaded, percentage) => {
                        self.game_state_text = format!(
                            "Downloading Java. {downloaded} / {} MiB ({percentage}%)",
                            self.java_download_size
                        )
                    }
                    downloader::Progress::JavaDownloadFinished => {
                        self.game_state_text = String::from("Extracting Java")
                    }
                    downloader::Progress::JavaExtracted => {
                        self.game_state_text = String::from("Java was installed successfully.");
                        self.restrict_launch = false;
                        for (index, downloader) in self.downloaders.iter().enumerate() {
                            if downloader.id == id {
                                self.downloaders.remove(index);
                                break;
                            }
                        }

                        self.launch();
                    }
                    downloader::Progress::MissingFilesDownloadProgressed(missing_files) => {
                        self.restrict_launch = true;
                        self.game_state_text =
                            format!("Downloading missing files. {} left", missing_files);
                    }
                    downloader::Progress::MissingFilesDownloadFinished => {
                        self.restrict_launch = false;
                        for (index, downloader) in self.downloaders.iter().enumerate() {
                            if downloader.id == id {
                                self.downloaders.remove(index);
                                break;
                            }
                        }

                        self.launch();
                    }
                }
                Command::none()
            }
            Message::VanillaJson(result) => {
                if result.is_null() {
                    self.game_state_text =
                        String::from("Json download failed. Check your internet connection.");
                } else {
                    self.game_state_text = String::from("Json downloaded successfully.");
                }

                self.launch();
                Command::none()
            }
            Message::LoadVersionList(ver_list) => {
                self.all_versions = ver_list;
                Command::none()
            }
            Message::GameEnviromentVariablesChanged(s) => {
                self.game_enviroment_variables = s;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let sidebar = container(
            column![
                //main
                action(
                    button(svg(svg::Handle::from_memory(
                        include_bytes!("icons/home.svg").as_slice()
                    )))
                    .on_press(Message::ChangeScreen(Screen::Main))
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(40.))
                    .height(Length::Fixed(40.)),
                    "Main"
                ),
                //options
                action(
                    button(svg(svg::Handle::from_memory(
                        include_bytes!("icons/options.svg").as_slice()
                    )))
                    .on_press(Message::ChangeScreen(Screen::Options))
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(40.))
                    .height(Length::Fixed(40.)),
                    "Options"
                ),
                //download screen
                action(
                    button(svg(svg::Handle::from_memory(
                        include_bytes!("icons/download.svg").as_slice()
                    )))
                    .on_press(Message::ChangeScreen(Screen::Installation))
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(40.))
                    .height(Length::Fixed(40.)),
                    "Install a version"
                ),
                //account
                action(
                    button(svg(svg::Handle::from_memory(
                        include_bytes!("icons/account.svg").as_slice()
                    )))
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(40.))
                    .height(Length::Fixed(40.)),
                    "WIP"
                ),
                //github
                action(
                    button(svg(svg::Handle::from_memory(
                        include_bytes!("icons/github.svg").as_slice()
                    )))
                    .on_press(Message::GithubButtonPressed)
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(40.))
                    .height(Length::Fixed(40.)),
                    "Redirect to github repository"
                )
            ]
            .spacing(25)
            .align_items(Alignment::Center),
        )
        .style(theme::Container::BlackContainer)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .width(50)
        .height(Length::Fill);

        let content = match self.screen {
            Screen::Main => {
                let (launch_text, launch_message) = match self.launcher.state {
                    LauncherState::Idle => ("Launch", Option::Some(Message::Launch)),
                    LauncherState::Launching(_) => ("Launching", Option::None),
                    LauncherState::GettingLogs => ("Running", Option::None),
                    LauncherState::Waiting => ("...", Option::None),
                };
                let launch_button = button(
                    text(launch_text)
                        .size(40)
                        .horizontal_alignment(alignment::Horizontal::Center),
                )
                .width(285)
                .height(60)
                .on_press_maybe(launch_message);

                column![
                    //mainscreen
                    //title
                    column![
                        text("Siglauncher").size(50),
                        text(format!("Hello {}!", self.username))
                            .style(theme::Text::Peach)
                            .size(18)
                    ]
                    .spacing(5),
                    //username and version input
                    row![
                        container(
                            column![
                                text("Username:"),
                                text_input("Username", &self.username)
                                    .on_input(Message::UsernameChanged)
                                    .size(25)
                                    .width(285),
                                text("Version:"),
                                pick_list(
                                    &self.all_versions,
                                    Some(self.current_version.clone()),
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
                                    text("Open game profile folder")
                                        .horizontal_alignment(alignment::Horizontal::Center)
                                )
                                .width(200)
                                .height(32)
                                .on_press(Message::OpenGameProfileFolder),
                                button(
                                    text("Logs")
                                        .horizontal_alignment(alignment::Horizontal::Center)
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
                        launch_button,
                        text(&self.game_state_text)
                            .style(theme::Text::Green)
                            .size(18)
                    ]
                    .spacing(10),
                ]
                .spacing(25)
                .max_width(800)
            }

            Screen::Options => column![
                //optionsscreen
                //title
                text("Options").size(50),
                //jvm and profile management
                row![
                    container(
                        column![
                            column![
                                text("JVM:").horizontal_alignment(alignment::Horizontal::Center),
                                pick_list(
                                    &self.java_name_list,
                                    Some(self.current_java_name.clone()),
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
                            .max_width(800)
                            .align_items(Alignment::Center),
                            column![
                                text("Game profile:")
                                    .horizontal_alignment(alignment::Horizontal::Center),
                                pick_list(
                                    &self.game_profile_list,
                                    Some(self.current_game_profile.clone()),
                                    Message::GameProfileChanged
                                )
                                .width(250)
                                .text_size(25),
                                button(
                                    text("Manage game profiles")
                                        .width(250)
                                        .horizontal_alignment(alignment::Horizontal::Center)
                                )
                                .height(32)
                                .on_press(Message::ChangeScreen(Screen::GameProfile))
                            ]
                            .spacing(10)
                            .max_width(800)
                            .align_items(Alignment::Center)
                        ]
                        .spacing(10)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(10),
                    //memory, gamemode and showallversions option
                    container(
                        column![
                            column![
                                text(format!("Allocated memory: {}GiB", self.game_ram))
                                    .size(25)
                                    .horizontal_alignment(alignment::Horizontal::Center),
                                slider(0.5..=16.0, self.game_ram, Message::GameRamChanged)
                                    .width(250)
                                    .step(0.5)
                            ],
                            row![
                                toggler(
                                    String::new(),
                                    self.show_all_versions_in_download_list,
                                    Message::ShowAllVersionsInDownloadListChanged
                                )
                                .width(Length::Shrink),
                                text("Show all versions in installer")
                                    .horizontal_alignment(alignment::Horizontal::Center)
                            ]
                            .spacing(10),
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
                column![
                //installerscreen
                //title
                text("Version installer").size(50),

                row![
                //vanilla
                container(
                column![
                    text("Vanilla"),
                pick_list(
                    self.vanilla_versions_download_list.clone(),
                    Some(self.vanilla_version_to_download.clone()),
                    Message::VanillaVersionToDownloadChanged,
                )
                .placeholder("Select a version")
                .width(250)
                .text_size(15),
                //installbutton
                button(
                    text("Install")
                        .size(25)
                        .horizontal_alignment(alignment::Horizontal::Center)
                )
                .width(250)
                .height(40)
                .on_press_maybe(Some(Message::InstallVersion(downloader::VersionType::Vanilla)))
                .style(theme::Button::Secondary)].spacing(15)).style(theme::Container::BlackContainer).padding(10),

                //fabric
                container(
                    column![
                        text("Fabric"),
                    pick_list(
                        self.fabric_versions_download_list.clone(),
                        Some(self.fabric_version_to_download.clone()),
                        Message::FabricVersionToDownloadChanged,
                    )
                    .placeholder("Select a version")
                    .width(250)
                    .text_size(15),
                    //installbutton
                    button(
                        text("Install")
                            .size(25)
                            .horizontal_alignment(alignment::Horizontal::Center)
                    )
                    .width(250)
                    .height(40)
                    .on_press_maybe(Some(Message::InstallVersion(downloader::VersionType::Fabric)))
                    .style(theme::Button::Secondary)].spacing(15)).style(theme::Container::BlackContainer).padding(10)].spacing(15),

                if !self.show_all_versions_in_download_list{
                    text("Enable the \"Show all versions in installer\" setting to download snapshots and other versions.").style(theme::Text::Green)
                } else{
                    text("")
                },
                text(&self.download_text).size(15)]
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
                        text_input("", &self.jvm_to_add_name)
                            .on_input(Message::JvmNameToAddChanged)
                            .size(25)
                            .width(250),
                        text("JVM path:"),
                        text_input("", &self.jvm_to_add_path)
                            .on_input(Message::JvmPathToAddChanged)
                            .size(25)
                            .width(250),
                        text("JVM flags:"),
                        text_input("", &self.jvm_to_add_flags)
                            .on_input(Message::JvmFlagsToAddChanged)
                            .size(25)
                            .width(250),
                        button(
                            text("Add")
                                .size(20)
                                .horizontal_alignment(alignment::Horizontal::Center)
                        )
                        .width(135)
                        .height(30)
                        .on_press(Message::JvmAdded)
                    ]
                    .spacing(5)
                )
                .style(theme::Container::BlackContainer)
                .padding(15)
            ]
            .spacing(15)
            .max_width(800),
            Screen::GameProfile => column![
                text("Manage game profiles")
                    .size(50)
                    .horizontal_alignment(alignment::Horizontal::Center),
                container(
                    column![
                        text("New game profile"),
                        text("Game profile name:"),
                        text_input("", &self.game_profile_to_add)
                            .on_input(Message::GameProfileToAddChanged)
                            .size(25)
                            .width(250),
                        button(
                            text("Add")
                                .size(20)
                                .horizontal_alignment(alignment::Horizontal::Center)
                        )
                        .width(135)
                        .height(30)
                        .on_press(Message::GameProfileAdded)
                    ]
                    .spacing(15)
                )
                .style(theme::Container::BlackContainer)
                .padding(15)
            ]
            .spacing(15)
            .max_width(800),

            Screen::Logs => column![
                text("Game logs").size(50),
                container(scrollable(text(self.logs.join("\n")).size(10)))
                    .style(theme::Container::BlackContainer)
                    .padding(10)
            ]
            .spacing(15),
            Screen::ModifyCommand => column![
                text("Modify game command").size(50),
                text("advanced settings, only edit if you know what you are doing.")
                    .size(15)
                    .style(theme::Text::Red),
                text("Wraper commands").size(25),
                text_input("Example: command1 command2", &self.game_wrapper_commands)
                    .on_input(Message::GameWrapperCommandsChanged)
                    .size(12),
                text("Enviroment variables").size(25),
                text_input(
                    "Example: KEY1=value1 KEY2=value2",
                    &self.game_enviroment_variables
                )
                .on_input(Message::GameEnviromentVariablesChanged)
                .size(12)
            ]
            .spacing(25),
        };

        container(row![sidebar, content].spacing(65))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(alignment::Vertical::Center)
            .padding(15)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();

        for i in &self.downloaders {
            subscriptions.push(i.subscription())
        }
        subscriptions.push(self.launcher.subscription());

        Subscription::batch(subscriptions)
    }
}

fn action<'a>(widget: Button<'a, Message, Renderer>, tp_text: &str) -> Element<'a, Message> {
    tooltip(widget, tp_text, tooltip::Position::Right)
        .style(theme::Container::BlackerBlackContainer)
        .padding(10)
        .into()
}

// Configuration file options{
fn checksettingsfile() {
    let mut conf_json = match Path::new(&get_config_file_path()).exists() {
        true => getjson(get_config_file_path()),
        false => serde_json::json!({}),
    };

    let mut file = File::create(get_config_file_path()).unwrap();

    if let Value::Object(map) = &mut conf_json {
        if !map.contains_key("JVMs") {
            let jvm = vec![
                Java{name: "Automatic".to_string(), path: String::new(), flags: String::new()},
                Java{name:"System Java".to_string(),path:"java".to_string(),flags:"-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+AlwaysPreTouch -XX:+DisableExplicitGC -XX:+UseNUMA -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=400M -XX:NonNMethodCodeHeapSize=12M -XX:ProfiledCodeHeapSize=194M -XX:NonProfiledCodeHeapSize=194M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -XX:+UseVectorCmov -XX:+PerfDisableSharedMem -XX:+UseFastUnorderedTimeStamps -XX:+UseCriticalJavaThreadPriority -XX:ThreadPriorityPolicy=1 -XX:AllocatePrefetchStyle=3".to_string()}
            ];

            map.insert("JVMs".to_owned(), serde_json::to_value(jvm).unwrap());
        }

        if !map.contains_key("username") {
            map.insert(
                "username".to_owned(),
                serde_json::to_value(String::from("player")).unwrap(),
            );
        }

        if !map.contains_key("current_version") {
            map.insert(
                "current_version".to_owned(),
                serde_json::to_value(String::new()).unwrap(),
            );
        }

        if !map.contains_key("game_ram") {
            map.insert("game_ram".to_owned(), serde_json::to_value(2.5).unwrap());
        }

        if !map.contains_key("current_java_name") {
            map.insert(
                "current_java_name".to_owned(),
                serde_json::to_value(String::from("Automatic")).unwrap(),
            );
        }

        if !map.contains_key("game_enviroment_variables") {
            map.insert(
                "game_enviroment_variables".to_owned(),
                serde_json::to_value(String::new()).unwrap(),
            );
        }

        if !map.contains_key("game_wrapper_commands") {
            map.insert(
                "game_wrapper_commands".to_owned(),
                serde_json::to_value(String::new()).unwrap(),
            );
        }

        if !map.contains_key("current_game_profile") {
            map.insert(
                "current_game_profile".to_owned(),
                serde_json::to_value(String::from("Default")).unwrap(),
            );
        }

        if !map.contains_key("show_all_versions") {
            map.insert(
                "show_all_versions".to_owned(),
                serde_json::to_value(false).unwrap(),
            );
        }
    }
    let serializedjson = serde_json::to_string_pretty(&conf_json).unwrap();

    file.write_all(serializedjson.as_bytes()).unwrap();
}

fn updateusersettingsfile(username: String, version: String) -> std::io::Result<()> {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    let mut file = File::open(get_config_file_path())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Value = serde_json::from_str(&contents)?;

    data["username"] = serde_json::Value::String(username);
    data["current_version"] = serde_json::Value::String(version);

    let serialized = serde_json::to_string_pretty(&data)?;

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(get_config_file_path())?;
    file.write_all(serialized.as_bytes())?;

    Ok(())
}

fn updatesettingsfile(
    ram: f64,
    currentjvm: String,
    current_game_profile: String,
    env_var: String,
    showallversions: bool,
) -> std::io::Result<()> {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    let mut file = File::open(get_config_file_path())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Value = serde_json::from_str(&contents)?;

    data["game_ram"] = serde_json::Value::Number(Number::from_f64(ram).unwrap());
    data["current_java_name"] = serde_json::Value::String(currentjvm);
    data["current_game_profile"] = serde_json::Value::String(current_game_profile);
    data["game_wrapper_commands"] = serde_json::Value::String(env_var);
    data["show_all_versions"] = serde_json::Value::Bool(showallversions);

    let serialized = serde_json::to_string_pretty(&data)?;

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(get_config_file_path())?;
    file.write_all(serialized.as_bytes())?;

    Ok(())
}

// } Configuration file options

// Launcher Struct for subscriptions and interacting with launcher.rs
#[derive(Debug)]
struct Launcher {
    state: LauncherState,
}
#[derive(Debug, PartialEq)]
enum LauncherState {
    Idle,
    Waiting,
    Launching(Box<launcher::GameSettings>),
    GettingLogs,
}
impl Default for Launcher {
    fn default() -> Self {
        Launcher {
            state: LauncherState::Idle,
        }
    }
}
impl Launcher {
    pub fn start(&mut self, game_settings: launcher::GameSettings) {
        self.state = LauncherState::Launching(Box::new(game_settings))
    }
    pub fn subscription(&self) -> Subscription<Message> {
        match &self.state {
            LauncherState::Idle => Subscription::none(),
            LauncherState::Launching(game_settings) => {
                launcher::start(0, Some(game_settings)).map(Message::ManageGameInfo)
            }
            LauncherState::GettingLogs => launcher::start(0, None).map(Message::ManageGameInfo),
            LauncherState::Waiting => Subscription::none(),
        }
    }
}

// Downloader struct for subscriptions and interacting with downloader.rs
struct Downloader {
    state: DownloaderState,
    id: usize,
}
enum DownloaderState {
    Idle,
    Downloading(String, downloader::VersionType),
    JavaDownloading(downloader::Java),
    DownloadingMissingFiles(downloader::DownloadList),
}

impl Default for Downloader {
    fn default() -> Self {
        Downloader {
            state: DownloaderState::Idle,
            id: 0,
        }
    }
}
impl Downloader {
    pub fn new(id: usize) -> Self {
        Downloader {
            state: DownloaderState::Idle,
            id,
        }
    }

    pub fn start(&mut self, version: String, version_type: downloader::VersionType) {
        self.state = DownloaderState::Downloading(version, version_type)
    }
    pub fn start_java(&mut self, java: downloader::Java) {
        self.state = DownloaderState::JavaDownloading(java)
    }
    pub fn start_missing_files(&mut self, files: Vec<downloader::Download>) {
        let download_list = downloader::DownloadList {
            download_list: files,
            client: reqwest::Client::new(),
        };
        self.state = DownloaderState::DownloadingMissingFiles(download_list)
    }
    pub fn subscription(&self) -> Subscription<Message> {
        match &self.state {
            DownloaderState::Idle => Subscription::none(),
            DownloaderState::Downloading(version, version_type) => {
                downloader::start(self.id, version.to_string(), version_type.clone())
                    .map(Message::ManageDownload)
            }
            DownloaderState::JavaDownloading(java) => {
                downloader::start_java(self.id, java.clone()).map(Message::ManageDownload)
            }
            DownloaderState::DownloadingMissingFiles(download_list) => {
                downloader::start_missing_files(self.id, download_list.clone())
                    .map(Message::ManageDownload)
            }
        }
    }
}
// for Theme
mod widget {
    use crate::theme::Theme;

    pub type Renderer = iced::Renderer<Theme>;
    pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
}

// java struct
#[derive(Default, Serialize, Deserialize)]
struct Java {
    name: String,
    path: String,
    flags: String,
}

fn getjson(jpathstring: String) -> Value {
    let jsonpath = Path::new(&jpathstring);

    let mut file = File::open(jsonpath).unwrap();
    let mut fcontent = String::new();
    file.read_to_string(&mut fcontent).unwrap();
    serde_json::from_str(&fcontent).unwrap()
}

fn get_config_file_path() -> String {
    #[cfg(debug_assertions)]
    return format!(
        "{}/siglauncher_settings_debug.json",
        launcher::get_minecraft_dir()
    );

    #[cfg(not(debug_assertions))]
    return format!(
        "{}/siglauncher_settings.json",
        launcher::get_minecraft_dir()
    );
}
