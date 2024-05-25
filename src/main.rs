#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use self::widget::Element;
use iced::{
    alignment,
    event::listen_with,
    executor,
    widget::{button, column, container, row, svg, tooltip, Button},
    window::{self, Id},
    Alignment, Application, Command, Length, Settings, Subscription,
};
use launcher::get_minecraft_dir;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use shared_child::SharedChild;
use std::io::Read;
use std::{collections::HashMap, env::set_current_dir};
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
use std::{fs::File, sync::Arc};
use widget::Renderer;

mod downloader;
mod launcher;
mod theme;
use theme::Theme;
mod screens;

fn main() -> iced::Result {
    if !Path::new(&get_minecraft_dir()).exists() {
        match fs::create_dir_all(get_minecraft_dir()) {
            Ok(_) => println!("Minecraft directory was created."),
            Err(e) => println!("Failed to create Minecraft directory: {e}"),
        };
    }

    let icon = include_bytes!("icons/minelander.png");

    Minelander::run(Settings {
        id: Some(String::from("Minelander")),
        window: window::Settings {
            size: iced::Size {
                width: 900.,
                height: 535.,
            },
            resizable: false,
            icon: Some(window::icon::from_file_data(icon, None).unwrap()),
            exit_on_close_request: false,

            ..window::Settings::default()
        },

        ..Settings::default()
    })
}

#[derive(Default)]
struct Minelander {
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
    current_game_instance: String,
    game_wrapper_commands: String,
    game_enviroment_variables: String,
    show_all_versions_in_download_list: bool,

    all_versions: Vec<String>,
    java_name_list: Vec<String>,
    game_instance_list: Vec<String>,
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

    game_instance_to_add: String,

    restrict_launch: bool,
    java_download_size: u8,

    game_proccess: GameProcess,
}

#[derive(Default)]
enum GameProcess {
    Running(Arc<SharedChild>),
    #[default]
    Null,
}

#[derive(PartialEq, Debug, Clone, Default)]
pub enum Screen {
    #[default]
    Main,
    Settings,
    Installation,
    Java,
    GameInstance,
    Logs,
    ModifyCommand,
    Info,
}
#[derive(Debug, Clone)]
enum Message {
    LoadVersionList(Vec<String>),

    Launch,
    CloseGame,
    ManageGameInfo((usize, launcher::Progress)),

    UsernameChanged(String),
    VersionChanged(String),

    JavaChanged(String),
    GameInstanceChanged(String),
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
    OpenGameInstanceFolder,

    ChangeScreen(Screen),

    JvmNameToAddChanged(String),
    JvmPathToAddChanged(String),
    JvmFlagsToAddChanged(String),
    JvmAdded,

    GameInstanceToAddChanged(String),
    GameInstanceAdded,

    Github,

    Exit,
}

impl Minelander {
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

        let java_type = match self.current_java_name.as_str() {
            "Automatic" => launcher::JavaType::Automatic,
            "System Java" => launcher::JavaType::System,
            "Java 8 (Minelander)" => launcher::JavaType::LauncherJava8,
            "Java 17 (Minelander)" => launcher::JavaType::LauncherJava17,
            _ => launcher::JavaType::Custom,
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
            game_directory: self.current_game_instance.clone(),
            java_type,
            enviroment_variables: enviroment_variables_hash_map,
        };
        self.launcher.start(game_settings);
        self.logs.clear();
    }
}

impl Application for Minelander {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = theme::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        // Configuration file
        backward_compatibility_measures();
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
        currentjava.name = p["current_java_name"].as_str().unwrap().to_string();

        let mut jvmnames: Vec<String> = Vec::new();
        if let Some(jvms) = p["JVMs"].as_array() {
            for jvm in jvms {
                jvmnames.push(jvm["name"].as_str().unwrap().to_owned());
                if jvm["name"] == p["current_java_name"] {
                    currentjava.path = jvm["path"].as_str().unwrap().to_owned();
                    currentjava.flags = jvm["flags"].as_str().unwrap().to_owned();
                }
            }
        }

        jvmnames.push("Automatic".to_owned());
        jvmnames.push("System Java".to_owned());
        jvmnames.push("Java 8 (Minelander)".to_owned());
        jvmnames.push("Java 17 (Minelander)".to_owned());
        // Get Java info

        // Game instance folder creation if it doesn't exist
        let mc_dir = launcher::get_minecraft_dir();
        let game_instance_folder_path = format!("{}/minelander_instances", mc_dir);
        if !Path::new(&game_instance_folder_path).exists() {
            match fs::create_dir_all(&game_instance_folder_path) {
                Ok(_) => println!("Created game instances folder"),
                Err(e) => println!("Failed to create game instances folder: {}", e),
            }
        }
        // Game instance folder creation if it doesn't exist

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
        let entries = fs::read_dir(game_instance_folder_path).unwrap();
        let mut new_game_instance_list = entries
            .filter_map(|entry| {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    Some(path.file_name().unwrap().to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        new_game_instance_list.push("Default".to_string());

        (
            Minelander {
                screen: Screen::Main,
                username: p["username"].as_str().unwrap().to_owned(),
                current_version: p["current_version"].as_str().unwrap().to_owned(),
                game_ram: p["game_ram"].as_f64().unwrap(),
                current_java_name: currentjava.name.clone(),
                current_java: currentjava,
                current_game_instance: p["current_game_instance"].as_str().unwrap().to_owned(),
                game_wrapper_commands: p["game_wrapper_commands"].as_str().unwrap().to_owned(),
                game_enviroment_variables: p["game_enviroment_variables"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
                show_all_versions_in_download_list: p["show_all_versions"].as_bool().unwrap(),
                java_name_list: jvmnames,
                game_instance_list: new_game_instance_list,
                needs_to_update_download_list: true,
                ..Default::default()
            },
            Command::perform(launcher::getinstalledversions(), Message::LoadVersionList),
        )
    }

    fn title(&self) -> String {
        format!("Minelander {}", env!("CARGO_PKG_VERSION"))
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
                    launcher::Progress::Started(child) => {
                        self.launcher.state = LauncherState::GettingLogs;
                        self.game_proccess = GameProcess::Running(child);
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
                if self.screen == Screen::Settings {
                    updatesettingsfile(
                        self.game_ram,
                        self.current_java_name.clone(),
                        self.current_game_instance.clone(),
                        self.game_wrapper_commands.clone(),
                        self.game_enviroment_variables.clone(),
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
            Message::OpenGameInstanceFolder => {
                if self.current_game_instance == "Default" {
                    open::that(launcher::get_minecraft_dir()).unwrap();
                } else {
                    open::that(format!(
                        "{}/minelander_instances/{}",
                        launcher::get_minecraft_dir(),
                        self.current_game_instance
                    ))
                    .unwrap();
                }
                Command::none()
            }
            Message::JavaChanged(selected_jvm_name) => {
                set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

                let mut newjvm: Vec<String> = Vec::new();
                let mut newjvmname: String = String::new();

                if selected_jvm_name.as_str() == "System Java"
                    || selected_jvm_name.as_str() == "Automatic"
                    || selected_jvm_name.as_str() == "Java 8 (Minelander)"
                    || selected_jvm_name.as_str() == "Java 17 (Minelander)"
                {
                    newjvm.push(selected_jvm_name.clone());
                    newjvm.push(String::new());
                    newjvm.push(String::new());

                    newjvmname = selected_jvm_name;
                } else {
                    let mut file = File::open(get_config_file_path()).unwrap();
                    let mut fcontent = String::new();
                    file.read_to_string(&mut fcontent).unwrap();
                    let content = serde_json::from_str(&fcontent);
                    let p: Value = content.unwrap();

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
                }

                self.current_java_name = newjvmname;
                self.current_java = Java {
                    name: newjvm[0].clone(),
                    path: newjvm[1].clone(),
                    flags: newjvm[2].clone(),
                };
                Command::none()
            }
            Message::GameInstanceChanged(new_game_instance) => {
                self.current_game_instance = new_game_instance;
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
                Command::perform(
                    async move { downloader::get_downloadable_version_list(bool).await },
                    Message::GotDownloadList,
                )
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
                    self.screen = Screen::Settings;
                }
                Command::none()
            }
            Message::GameInstanceToAddChanged(game_prof) => {
                self.game_instance_to_add = game_prof;
                Command::none()
            }
            Message::GameInstanceAdded => {
                if !self.game_instance_to_add.is_empty() {
                    fs::create_dir_all(format!(
                        "{}/minelander_instances/{}",
                        launcher::get_minecraft_dir(),
                        self.game_instance_to_add
                    ))
                    .expect("Failed to create directory!");

                    let entries = fs::read_dir(format!(
                        "{}/minelander_instances",
                        launcher::get_minecraft_dir()
                    ))
                    .unwrap();

                    let mut new_game_instance_list = entries
                        .filter_map(|entry| {
                            let path = entry.unwrap().path();
                            if path.is_dir() {
                                Some(path.file_name().unwrap().to_string_lossy().to_string())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    new_game_instance_list.push("Default".to_string());

                    self.game_instance_list = new_game_instance_list;

                    self.screen = Screen::Settings;
                }
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
            Message::Exit => {
                self.launcher.state = LauncherState::Idle;
                self.downloaders.clear();
                window::close(Id::MAIN)
            }
            Message::CloseGame => {
                match &self.game_proccess {
                    GameProcess::Running(process) => match process.kill() {
                        Ok(ok) => ok,
                        Err(e) => panic!("{}", e),
                    },
                    GameProcess::Null => todo!(),
                }

                Command::none()
            }
            Message::Github => {
                match open::that_detached("https://github.com/jafkc2/minelander") {
                    Ok(ok) => ok,
                    Err(e) => println!("Failed to open Github repository in browser: {e}"),
                }

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
                    .width(Length::Fixed(42.))
                    .height(Length::Fixed(42.)),
                    "Main Screen"
                ),
                // Settings
                action(
                    button(svg(svg::Handle::from_memory(
                        include_bytes!("icons/settings.svg").as_slice()
                    )))
                    .on_press(Message::ChangeScreen(Screen::Settings))
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(42.))
                    .height(Length::Fixed(42.)),
                    "Settings"
                ),
                //download screen
                action(
                    button(svg(svg::Handle::from_memory(
                        include_bytes!("icons/download.svg").as_slice()
                    )))
                    .on_press(Message::ChangeScreen(Screen::Installation))
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(42.))
                    .height(Length::Fixed(42.)),
                    "Installer"
                ),
                //account
                action(
                    button(svg(svg::Handle::from_memory(
                        include_bytes!("icons/account.svg").as_slice()
                    )))
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(42.))
                    .height(Length::Fixed(42.)),
                    "Account (WIP)"
                ),
                // Info
                action(
                    button(svg(svg::Handle::from_memory(
                        include_bytes!("icons/info.svg").as_slice()
                    )))
                    .on_press(Message::ChangeScreen(Screen::Info))
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(42.))
                    .height(Length::Fixed(42.)),
                    "Info"
                )
            ]
            .spacing(20)
            .align_items(Alignment::Center),
        )
        .style(theme::Container::BlackContainer)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .width(50)
        .height(Length::Fixed(400.));

        let content = screens::get_screen_content(self);

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

        let events = listen_with(|event, _status| match event {
            iced::Event::Window(Id::MAIN, window::Event::CloseRequested) => Some(Message::Exit),
            _ => None,
        });

        subscriptions.push(events);

        Subscription::batch(subscriptions)
    }
}

fn action<'a>(
    widget: Button<'a, Message, Theme, Renderer>,
    tp_text: &'a str,
) -> Element<'a, Message> {
    tooltip(widget, tp_text, tooltip::Position::Right)
        .style(theme::Container::BlackContainer)
        .padding(10)
        .into()
}

// Configuration file settings{
fn checksettingsfile() {
    let mut conf_json = match Path::new(&get_config_file_path()).exists() {
        true => getjson(get_config_file_path()),
        false => serde_json::json!({}),
    };

    let mut file = File::create(get_config_file_path()).unwrap();

    if let Value::Object(map) = &mut conf_json {
        if !map.contains_key("JVMs") {
            let jvm: Vec<Java> = vec![];

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

        if !map.contains_key("current_game_instance") {
            map.insert(
                "current_game_instance".to_owned(),
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
    current_game_instance: String,
    wrapper_commands: String,
    env_variables: String,
    showallversions: bool,
) -> std::io::Result<()> {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    let mut file = File::open(get_config_file_path())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Value = serde_json::from_str(&contents)?;

    data["game_ram"] = serde_json::Value::Number(Number::from_f64(ram).unwrap());
    data["current_java_name"] = serde_json::Value::String(currentjvm);
    data["current_game_instance"] = serde_json::Value::String(current_game_instance);
    data["game_wrapper_commands"] = serde_json::Value::String(wrapper_commands);
    data["show_all_versions"] = serde_json::Value::Bool(showallversions);
    data["game_enviroment_variables"] = serde_json::Value::String(env_variables);

    let serialized = serde_json::to_string_pretty(&data)?;

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(get_config_file_path())?;
    file.write_all(serialized.as_bytes())?;

    Ok(())
}

// } Configuration file Settings

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

    pub type Renderer = iced::Renderer;
    pub type Element<'a, Message> = iced::Element<'a, Message, Theme, Renderer>;
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
        "{}/minelander_settings_debug.json",
        launcher::get_minecraft_dir()
    );

    #[cfg(not(debug_assertions))]
    return format!("{}/minelander_settings.json", launcher::get_minecraft_dir());
}

fn is_file_empty(file_path: &str) -> bool {
    let mut file = File::open(file_path).unwrap();
    let mut buffer = [0; 1];

    match file.read(&mut buffer).unwrap() {
        0 => true,
        _ => false,
    }
}

fn backward_compatibility_measures() {
    let old_game_instances_path = format!("{}/minelander_profiles", get_minecraft_dir());
    let new_game_instances_path = format!("{}/minelander_instances", get_minecraft_dir());

    if Path::new(&old_game_instances_path).is_dir() {
        match fs::rename(old_game_instances_path, new_game_instances_path) {
            Ok(ok) => ok,
            Err(e) => println!("Failed to rename minelander_profiles folder: {e}"),
        }
    }
}
