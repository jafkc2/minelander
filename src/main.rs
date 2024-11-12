#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use self::widget::Element;
use iced::{
    alignment, clipboard,
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
mod auth;
mod screens;
mod update_manager;

fn main() -> iced::Result {
    if !Path::new(&get_minecraft_dir()).exists() {
        match fs::create_dir_all(get_minecraft_dir()) {
            Ok(_) => println!("Minecraft directory was created."),
            Err(e) => println!("Failed to create Minecraft directory: {e}"),
        };
    }

    let old_exec = env::current_exe().unwrap().with_extension("old");
    if Path::new(&old_exec).exists() {
        match fs::remove_file(old_exec) {
            Ok(ok) => ok,
            Err(e) => println!("Failed to delete old executable: {e}"),
        }
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

    current_account: Account,
    current_account_mc_data: auth::MinecraftAccount,

    current_version: String,
    game_state_text: String,
    game_state_text_2: String,

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

    update_available: bool,
    last_version: String,
    update_url: String,
    update_text: String,

    accounts: Vec<Account>,

    auth_code: auth::AuthCode,
    auth_token: auth::AuthToken,
    auth_xbox_data: auth::XboxLiveData,
    auth_status: String,

    local_account_to_add_name: String,

    is_first_launcher_use: bool
}

#[derive(Default, Serialize, Deserialize, Clone)]
struct Account {
    microsoft: bool,
    username: String,
    refresh_token: String,
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
    InfoAndUpdates,
    Accounts,
    MicrosoftAccount,
    LocalAccount,
    GettingStarted,
    GettingStarted2
}
#[derive(Debug, Clone)]
enum Message {
    LoadVersionList(Vec<String>),

    Launch,
    CloseGame,
    ManageGameInfo((usize, launcher::Progress)),

    CurrentAccountChanged(String),
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

    CheckedUpdates(Result<(String, String), String>),
    Update,

    OpenURL(String),
    CopyToClipboard(String),

    GotAuthCode(auth::AuthCode),
    ManageAuth((usize, auth::WaitProgress)),
    GotXboxToken(auth::XboxLiveData),
    GotMinecraftAuthData(auth::MinecraftAccount),
    RefreshLogin(Option<auth::MinecraftAccount>),

    LocalAccountNameChanged(String),
    AddedLocalAccount,
    RemoveAccount(String),

    Exit,
}

impl Minelander {
    pub fn launch(&mut self) {
        if updateusersettingsfile(self.current_account.clone(), self.current_version.clone())
            .is_err()
        {
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
            "Java 21 (Minelander)" => launcher::JavaType::LauncherJava21,
            _ => launcher::JavaType::Custom,
        };

        let game_settings = launcher::GameSettings {
            account: self.current_account_mc_data.clone(),
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
        self.current_account_mc_data.token = String::new();
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
        let is_first_launcher_use = checksettingsfile();

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
        jvmnames.push("Java 21 (Minelander)".to_owned());

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

        let mut accounts = vec![];

        if let Some(accounts_vec) = p["accounts"].as_array() {
            for account in accounts_vec {
                let microsoft = account["microsoft"].as_bool().unwrap();
                let username = account["username"].as_str().unwrap().to_string();
                let refresh_token = account["refresh_token"].as_str().unwrap().to_string();

                accounts.push(Account {
                    microsoft,
                    username,
                    refresh_token,
                })
            }
        }

        let current_account = Account {
            microsoft: p["current_account"]["microsoft"].as_bool().unwrap(),
            username: p["current_account"]["username"]
                .as_str()
                .unwrap()
                .to_owned(),
            refresh_token: p["current_account"]["refresh_token"]
                .as_str()
                .unwrap()
                .to_owned(),
        };

        let initial_screen = match is_first_launcher_use{
            true => Screen::GettingStarted,
            false => Screen::Main,
        };

        (
            Minelander {
                screen: initial_screen,
                current_account: current_account,
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
                accounts,
                is_first_launcher_use,
                ..Default::default()
            },
            Command::batch(vec![
                Command::perform(launcher::getinstalledversions(), Message::LoadVersionList),
                Command::perform(
                    update_manager::check_launcher_updates(),
                    Message::CheckedUpdates,
                ),
            ]),
        )
    }

    fn title(&self) -> String {
        format!("Minelander {}", env!("CARGO_PKG_VERSION"))
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Launch => {
                if !self.restrict_launch
                    && !self.current_account.username.is_empty()
                    && !self.current_version.is_empty()
                {
                    if self.current_account.microsoft
                        && self.current_account_mc_data.token.is_empty()
                    {
                        self.game_state_text = String::from("Fetching account data...");

                        return Command::perform(
                            auth::login_with_refresh_token(
                                self.current_account.refresh_token.clone(),
                            ),
                            Message::RefreshLogin,
                        );
                    } else {
                        self.current_account_mc_data = auth::MinecraftAccount {
                            username: self.current_account.username.clone(),
                            token: "[pro]".to_string(),
                            uuid: String::new(),
                        }
                    }
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
                                launcher::Missing::Java21 => {
                                    self.launcher.state = LauncherState::Waiting;
                                    self.downloaders.push(Downloader {
                                        state: DownloaderState::Idle,
                                        id: self.downloaders.len(),
                                    });
                                    let index = self.downloaders.len() - 1;
                                    self.downloaders[index].start_java(downloader::Java::J21)
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

                match new_screen {
                    Screen::Main => {
                        self.is_first_launcher_use = false;
                        Command::perform(launcher::getinstalledversions(), Message::LoadVersionList)
                    }

                    Screen::Installation => {
                        if !self.vanilla_versions_download_list.is_empty()
                            || !self.fabric_versions_download_list.is_empty()
                            || self.needs_to_update_download_list
                        {
                            let show_all_versions = self.show_all_versions_in_download_list;
                            return Command::perform(
                                async move {
                                    downloader::get_downloadable_version_list(show_all_versions)
                                        .await
                                },
                                Message::GotDownloadList,
                            );
                        } else {
                            Command::none()
                        }
                    }
                    Screen::MicrosoftAccount => {
                        self.auth_status = String::from("Getting code and link...");
                        Command::perform(
                            async move { auth::request_code().await },
                            Message::GotAuthCode,
                        )
                    }

                    _ => Command::none(),
                }
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
                    || selected_jvm_name.as_str() == "Java 21 (Minelander)"
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

                        if self.is_first_launcher_use{
                            self.is_first_launcher_use = false;
                            self.screen = Screen::Main;
                            
                            return Command::perform(launcher::getinstalledversions(), Message::LoadVersionList)
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
                    downloader::Progress::UpdateStarted(total) => {
                        self.update_text = format!("Downloading update. 0 / {total} MiB (0%)")
                    }
                    downloader::Progress::UpdateProgressed(downloaded, percentage, total) => {
                        self.update_text = format!(
                            "Downloading update. {downloaded} / {total} MiB ({percentage}%)"
                        )
                    }
                    downloader::Progress::UpdateFinished => {
                        self.update_text = String::from("Update installed successfully.");
                        for (index, downloader) in self.downloaders.iter().enumerate() {
                            if downloader.id == id {
                                self.downloaders.remove(index);
                                break;
                            }
                        }
                        
                        let exec_path = env::current_exe().unwrap();

                        // renames current executable to minelander.old and renames updated executable to minelander
                        fs::rename(&exec_path, exec_path.with_extension("old")).unwrap();
                        fs::rename(exec_path.with_extension("new"), exec_path).unwrap();

                        
                        let mut new_exe_path = env::current_exe().unwrap();
                        new_exe_path.pop();
                        new_exe_path = new_exe_path.join("minelander");

                        match std::process::Command::new(new_exe_path).spawn() {
                            Ok(_) => std::process::exit(0),
                            Err(e) => panic!("{}", e),
                        }
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
                self.all_versions = ver_list.clone();


                if ver_list.len() == 1{
                    self.current_version = ver_list[0].clone()
                }

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
            Message::OpenURL(url) => {
                match open::that_detached(url) {
                    Ok(ok) => ok,
                    Err(e) => println!("Failed to open URL: {e}"),
                }

                Command::none()
            }
            Message::CheckedUpdates(result) => {
                match result {
                    Ok((url, last_version)) => {
                        self.update_available = true;
                        self.update_url = url;
                        self.last_version = last_version;
                    }
                    Err(e) => self.last_version = e,
                }
                Command::none()
            }
            Message::Update => {
                self.downloaders.push(Downloader {
                    state: DownloaderState::Idle,
                    id: self.downloaders.len(),
                });
                let index = self.downloaders.len() - 1;
                self.downloaders[index].start_update(self.update_url.clone());

                Command::none()
            }
            Message::GotAuthCode(code) => {
                self.auth_status = String::from("Waiting for login...");
                self.auth_code = code;
                Command::none()
            }
            Message::ManageAuth((_id, progress)) => {
                match progress {
                    auth::WaitProgress::GotAuthToken(auth_token) => {
                        self.auth_token = auth_token.clone();
                        self.auth_status = String::from("Logging into Xbox Services...");

                        return Command::perform(
                            async move { auth::login_to_xbox(auth_token.access_token).await },
                            Message::GotXboxToken,
                        );
                    }
                    auth::WaitProgress::Waiting => (),
                    auth::WaitProgress::Error(e) => println!("auth error: {e}"),
                    auth::WaitProgress::Finished => {
                        self.auth_code.code = String::new();
                        self.auth_code.link = String::new();
                    }
                }

                Command::none()
            }
            Message::GotXboxToken(xbox_data) => {
                self.auth_xbox_data = xbox_data.clone();
                self.auth_status = String::from("Logging into Minecraft...");

                Command::perform(
                    async move { auth::login_to_minecraft(xbox_data).await },
                    Message::GotMinecraftAuthData,
                )
            }
            Message::GotMinecraftAuthData(mc_account) => {
                let refresh_token = self.auth_token.refresh_token.clone();
                let account = Account {
                    microsoft: true,
                    username: mc_account.username,
                    refresh_token,
                };
                self.accounts = save_account(account.clone());
                self.current_account = account;

                self.auth_status = String::from("Account added successfully!");

                if self.screen == Screen::MicrosoftAccount{
                    if self.is_first_launcher_use{
                        if self.all_versions.is_empty(){
                            self.screen = Screen::GettingStarted2;
                        } else{
                            self.screen = Screen::Main;
                            self.is_first_launcher_use = false;
                        }
                    } else{
                        self.screen = Screen::Accounts;
                    }                }

                Command::none()
            }
            Message::CopyToClipboard(content) => clipboard::write(content),
            Message::CurrentAccountChanged(account_name) => {
                for i in &self.accounts {
                    if i.username == account_name {
                        self.current_account = i.clone();
                    }
                }

                Command::none()
            }
            Message::RefreshLogin(mc_account) => {
                if let Some(mc_account) = mc_account{
                    self.current_account_mc_data = mc_account;
                } else{
                    self.current_account_mc_data.username = self.current_account.username.clone();
                    self.game_state_text_2 = String::from("Game will run in offline mode. Check your internet connection.");                    
                }

                self.launch();

                Command::none()
            }
            Message::LocalAccountNameChanged(mut username) => {
                if username.chars().count() <= 16{
                    for (i, char) in username.clone().chars().enumerate(){
                        if !char.is_alphanumeric() && char != '_' {
                            username.remove(i);
                        }
                    }
                    self.local_account_to_add_name = username.replace(" ", "");
                }
                Command::none()
            }
            Message::AddedLocalAccount => {
                if self.local_account_to_add_name.chars().count() >= 3
                    && self.local_account_to_add_name.chars().count() <= 16
                {
                    let account = Account {
                        microsoft: false,
                        username: self.local_account_to_add_name.clone(),
                        refresh_token: String::new(),
                    };

                    self.accounts = save_account(account.clone());
                    self.current_account = account;

                    if self.is_first_launcher_use{
                        if self.all_versions.is_empty(){
                            self.screen = Screen::GettingStarted2;
                        } else{
                            self.screen = Screen::Main;
                            self.is_first_launcher_use = false;
                        }
                    } else{
                        self.screen = Screen::Accounts;
                    }

                    self.local_account_to_add_name = String::new();
                }
                Command::none()
            }
            Message::RemoveAccount(account_name) => {
                let mut config_file = getjson(get_config_file_path());

                let mut updated_account_list = vec![];

                if let Some(arr) = config_file["accounts"].as_array() {
                    for account in arr {
                        if account["username"].as_str().unwrap() != &account_name {
                            let microsoft = account["microsoft"].as_bool().unwrap();
                            let username = account["username"].as_str().unwrap().to_owned();
                            let refresh_token =
                                account["refresh_token"].as_str().unwrap().to_owned();

                            updated_account_list.push(Account {
                                microsoft,
                                username,
                                refresh_token,
                            })
                        }
                    }
                }

                config_file["accounts"] = serde_json::json!(updated_account_list);

                let serialized = serde_json::to_string_pretty(&config_file).unwrap();

                let mut file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(get_config_file_path())
                    .unwrap();
                file.write_all(serialized.as_bytes()).unwrap();

                self.accounts = updated_account_list;
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
                    .on_press(Message::ChangeScreen(Screen::Accounts))
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(42.))
                    .height(Length::Fixed(42.)),
                    "Account (WIP)"
                ),
                // Info and updates
                action(
                    button(svg(svg::Handle::from_memory(
                        include_bytes!("icons/info.svg").as_slice()
                    )))
                    .on_press(Message::ChangeScreen(Screen::InfoAndUpdates))
                    .style(theme::Button::Transparent)
                    .width(Length::Fixed(42.))
                    .height(Length::Fixed(42.)),
                    "Info and updates"
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

        let screen = screens::get_screen_content(self);

        match self.is_first_launcher_use{
            true =>      container(screen.height(Length::Fixed(400.))
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(alignment::Vertical::Center)
            .padding(15)
            .into(),
            false =>      container(row![sidebar, screen].spacing(65))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(alignment::Vertical::Center)
            .padding(15)
            .into(),
        }
        
   
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

        if !self.auth_code.code.is_empty() {
            let auth_sub = auth::start_wait_for_login(0, self.auth_code.device_code.clone())
                .map(Message::ManageAuth);

            subscriptions.push(auth_sub)
        };

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

// Configuration file settings, returns true if config file didn't exist.
fn checksettingsfile() -> bool {
    let file_exists = Path::new(&get_config_file_path()).exists();
    let mut conf_json = match file_exists {
        true => getjson(get_config_file_path()),
        false => serde_json::json!({}),
    };

    let mut file = File::create(get_config_file_path()).unwrap();

    if let Value::Object(map) = &mut conf_json {
        if !map.contains_key("JVMs") {
            let jvm: Vec<Java> = vec![];

            map.insert("JVMs".to_owned(), serde_json::to_value(jvm).unwrap());
        }

        if !map.contains_key("accounts") {
            let accounts: Vec<Account> = vec![];

            map.insert(
                "accounts".to_owned(),
                serde_json::to_value(accounts).unwrap(),
            );
        }

        if !map.contains_key("current_account") {
            map.insert(
                "current_account".to_owned(),
                serde_json::json!(Account {
                    microsoft: false,
                    username: String::new(),
                    refresh_token: String::new()
                }),
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

    !file_exists
}

fn updateusersettingsfile(current_account: Account, version: String) -> std::io::Result<()> {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    let mut file = File::open(get_config_file_path())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Value = serde_json::from_str(&contents)?;

    data["current_account"] = serde_json::json!(current_account);
    data["current_version"] = serde_json::Value::String(version);

    let serialized = serde_json::to_string_pretty(&data)?;

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(get_config_file_path())?;
    file.write_all(serialized.as_bytes())?;

    Ok(())
}

fn save_account(account: Account) -> Vec<Account> {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    let mut file = File::open(get_config_file_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut data: Value = serde_json::from_str(&contents).unwrap();

    if let Value::Array(arr) = &mut data["accounts"] {
        arr.push(serde_json::json!(account));
        data["accounts"] = serde_json::json!(arr);
    }

    let mut updated_account_list = vec![];
    if let Some(arr) = data["accounts"].as_array() {
        for account in arr {
            let microsoft = account["microsoft"].as_bool().unwrap();
            let username = account["username"].as_str().unwrap().to_owned();
            let refresh_token = account["refresh_token"].as_str().unwrap().to_owned();

            updated_account_list.push(Account {
                microsoft,
                username,
                refresh_token,
            })
        }
    }

    let serialized = serde_json::to_string_pretty(&data).unwrap();

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(get_config_file_path())
        .unwrap();
    file.write_all(serialized.as_bytes()).unwrap();

    updated_account_list
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
    Update(String),
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
    pub fn start_update(&mut self, url: String) {
        self.state = DownloaderState::Update(url)
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
            DownloaderState::Update(url) => {
                downloader::start_update(self.id, url.to_string()).map(Message::ManageDownload)
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
