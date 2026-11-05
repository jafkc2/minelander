#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod accounts;
mod auth;
mod config;
mod downloader;
mod java;
mod launcher;
mod messages;
mod minecraft;
mod mods;
mod screen;
mod theme;
mod updater;
mod widgets;

use iced::widget::row;
use iced::{Element, Subscription, Task, Theme};
use reqwest::Client;
use shared_child::SharedChild;
use std::path::Path;
use std::sync::Arc;
use std::{env, fs};

use crate::config::Config;
use crate::downloader::{java_downloader, version_downloader};
use crate::java::JavaType;
use crate::messages::Message;
use crate::screen::Screen;
use crate::widgets::sidebar::{ActiveScreen, Sidebar};

fn main() {
    // Removes old executable after an update
    let old_exec = env::current_exe().unwrap().with_extension("old");
    if Path::new(&old_exec).exists() {
        match fs::remove_file(old_exec) {
            Ok(ok) => ok,
            Err(e) => println!("Failed to delete old executable: {e}"),
        }
    }

    let icon = include_bytes!("../assets/icons/minelander.png");

    iced::application(Minelander::new, Minelander::update, Minelander::view)
        .title(Minelander::title)
        .theme(Minelander::theme)
        .subscription(Minelander::subscription)
        .window(iced::window::Settings {
            icon: Some(iced::window::icon::from_file_data(icon, None).unwrap()),
            ..Default::default()
        })
        .run()
        .unwrap();
}

struct Minelander {
    config: Config,
    versions: Vec<String>,
    account: Option<accounts::Account>,

    screen: Screen,
    sidebar: widgets::sidebar::Sidebar,
    java_request: Option<crate::java::JavaType>,
    version_request: Option<version_downloader::VersionRequest>,
    direct_download_files: Option<Vec<version_downloader::DownloadFile>>,
    available_versions: Option<version_downloader::AvailableVersions>,
    version_download_status: Option<String>,
    microsoft_login_state: Option<screen::account_manager::MicrosoftLoginState>,
    microsoft_login_id: usize,
    launch_settings: Option<launcher::GameSettings>,
    launch_ready: bool,
    launch_id: u64,
    active_process: Option<Arc<SharedChild>>,
    minecraft_logs: Vec<String>,
    logs_visible: bool,
    available_update: Option<(String, String)>,
    checking_for_updates: bool,
    update_status: Option<String>,
}

impl Minelander {
    fn new() -> (Self, Task<Message>) {
        let config = Config::load(&config::default_config_path()).unwrap_or_else(|error| {
            eprintln!("Failed to load configuration: {error:#}");
            Config::new()
        });
        let home = screen::home::Home::from_config(&config);
        let check_for_updates = config.check_for_updates;
        let mut tasks = vec![Task::perform(
            minecraft::get_installed_versions(),
            |versions| Message::GotInstalledVersions(versions),
        )];
        if check_for_updates {
            tasks.push(Task::perform(updater::check_update(), |result| {
                Message::UpdateCheckCompleted(match result {
                    Ok(update) => Ok(Some(update)),
                    Err(error) if error.to_string() == "Last version is already installed." => {
                        Ok(None)
                    }
                    Err(error) => Err(format!("{error:#}")),
                })
            }));
        }

        (
            Self {
                config,
                screen: Screen::Home(home),
                versions: vec![],
                account: None,
                sidebar: Sidebar::default(),
                java_request: None,
                version_request: None,
                direct_download_files: None,
                available_versions: None,
                version_download_status: None,
                microsoft_login_state: None,
                microsoft_login_id: 0,
                launch_settings: None,
                launch_ready: false,
                launch_id: 0,
                active_process: None,
                minecraft_logs: Vec::new(),
                logs_visible: false,
                available_update: None,
                checking_for_updates: check_for_updates,
                update_status: None,
            },
            Task::batch(tasks),
        )
    }

    fn title(&self) -> String {
        format!("Minelander 1.0")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Sidebar(sidebar_message) => {
                match sidebar_message {
                    widgets::sidebar::Message::HomeClicked => {
                        self.screen = Screen::Home(screen::home::Home::from_config(&self.config));
                    }
                    widgets::sidebar::Message::SettingsClicked => {
                        self.screen =
                            Screen::Settings(screen::settings::Settings::from_config(&self.config));
                    }
                    widgets::sidebar::Message::VersionClicked => {
                        self.screen = Screen::VersionManager(
                            screen::version_manager::VersionManager::default(),
                        );
                        if self.available_versions.is_none() {
                            let client = Client::new();
                            return Task::perform(
                                version_downloader::available_versions(client),
                                |result| {
                                    Message::AvailableVersionsLoaded(
                                        result.map_err(|error| format!("{error:#}")),
                                    )
                                },
                            );
                        }
                    }
                    widgets::sidebar::Message::AboutClicked => {
                        self.screen = Screen::About(screen::about::About::default());
                    }
                    widgets::sidebar::Message::UpdateClicked => {
                        if let Some((version, url)) = &self.available_update {
                            self.update_status = Some(format!("Installing update v{}...", version));
                            return Task::perform(updater::install_update(url.clone()), |result| {
                                Message::UpdateInstalled(
                                    result.map_err(|error| format!("{error:#}")),
                                )
                            });
                        }
                    }
                    widgets::sidebar::Message::ModsClicked => {
                        self.screen =
                            Screen::ModManager(screen::mod_manager::ModManager::from_instance(
                                &self.config.current_instance,
                            ));
                    }
                    widgets::sidebar::Message::AccountsClicked => {
                        self.screen = Screen::AccountManager(
                            screen::account_manager::AccountManager::default(),
                        );
                    }
                }

                Task::none()
            }
            Message::HomeScreen(screen_message) => {
                if let Screen::Home(home) = &mut self.screen {
                    let launch_clicked =
                        matches!(screen_message, screen::home::Message::LaunchClicked);
                    let stop_clicked = matches!(screen_message, screen::home::Message::StopClicked);
                    let logs_clicked = matches!(screen_message, screen::home::Message::LogsClicked);
                    let task = home.update(screen_message);
                    home.apply_to_config(&mut self.config);
                    if let Err(error) = self.config.save() {
                        eprintln!("Failed to save configuration: {error:#}");
                    }

                    // Launch button clicked
                    if launch_clicked {
                        self.minecraft_logs.clear();
                        self.logs_visible = false;
                        return Task::done(Message::LaunchStageAuthenticate);
                    }
                    if stop_clicked {
                        return Task::done(Message::StopMinecraft);
                    }
                    if logs_clicked {
                        self.logs_visible = !self.logs_visible;
                    }
                    task.map(Message::HomeScreen)
                } else {
                    Task::none()
                }
            }
            Message::LaunchStageAuthenticate => {
                self.launch_ready = false;
                let username = self.config.current_account.trim();
                let account = self
                    .config
                    .accounts
                    .iter()
                    .find(|account| account.username == username)
                    .map(|account| (account.username.clone(), account.offline));

                let Some((username, is_offline)) = account else {
                    self.set_home_status("Select an account first.");
                    return Task::none();
                };

                if is_offline {
                    self.account = Some(accounts::Account::new_offline(username));
                    self.set_home_status("Account ready.");
                    Task::done(Message::LaunchStageCheckJava)
                } else if self.account.as_ref().is_some_and(|account| {
                    !account.offline && account.username == username
                })
                {
                    self.set_home_status("Account ready.");
                    Task::done(Message::LaunchStageCheckJava)
                } else {
                    self.set_home_status("Authenticating Microsoft account...");
                    let refresh_token = match auth::get_refresh_token(&username) {
                        Ok(token) => token,
                        Err(error) => {
                            self.set_home_status(format!(
                                "Failed to retrieve credentials: {error:#}"
                            ));
                            return Task::none();
                        }
                    };

                    Task::perform(
                        async move { auth::authenticate_with_refresh_token(&refresh_token).await },
                        |res| Message::MicrosoftLaunchAuthCompleted(res.map_err(|e| e.to_string())),
                    )
                }
            }
            Message::MicrosoftDeviceFlowStarted(id, result) => {
                if id == self.microsoft_login_id {
                    match result {
                        Ok((code, flow)) => {
                            self.microsoft_login_state = Some(
                                screen::account_manager::MicrosoftLoginState::WaitingForUser {
                                    verification_uri: code.verification_uri.clone(),
                                    user_code: code.user_code.clone(),
                                },
                            );
                            return Task::perform(
                                auth::poll_device_flow_until_complete(flow),
                                move |res| {
                                    Message::MicrosoftAuthCompleted(
                                        id,
                                        res.map_err(|e| e.to_string()),
                                    )
                                },
                            );
                        }
                        Err(err) => {
                            self.microsoft_login_state =
                                Some(screen::account_manager::MicrosoftLoginState::Error(err));
                        }
                    }
                }
                Task::none()
            }
            Message::MicrosoftAuthCompleted(id, result) => {
                if id == self.microsoft_login_id {
                    match result {
                        Ok(authenticated) => {
                            if let Err(error) = auth::save_refresh_token(
                                &authenticated.minecraft_username,
                                &authenticated.refresh_token,
                            ) {
                                eprintln!("Failed to save refresh token to keyring: {error:#}");
                                self.microsoft_login_state =
                                    Some(screen::account_manager::MicrosoftLoginState::Error(
                                        format!("Keyring error: {error:#}"),
                                    ));
                                return Task::none();
                            }

                            if let Some(existing) = self
                                .config
                                .accounts
                                .iter_mut()
                                .find(|a| a.username == authenticated.minecraft_username)
                            {
                                existing.offline = false;
                            } else {
                                self.config.accounts.push(accounts::AccountInfo {
                                    username: authenticated.minecraft_username.clone(),
                                    offline: false,
                                });
                            }

                            self.account = Some(accounts::Account {
                                username: authenticated.minecraft_username.clone(),
                                offline: false,
                                uuid: authenticated.uuid.clone(),
                                access_token: authenticated.minecraft_access_token.clone(),
                            });
                            self.config.current_account = authenticated.minecraft_username;
                            if let Err(error) = self.config.save() {
                                eprintln!("Failed to save config: {error:#}");
                            }

                            self.microsoft_login_state = None;
                        }
                        Err(err) => {
                            self.microsoft_login_state =
                                Some(screen::account_manager::MicrosoftLoginState::Error(err));
                        }
                    }
                }
                Task::none()
            }
            Message::MicrosoftLaunchAuthCompleted(result) => match result {
                Ok(authenticated) => {
                    let _ = auth::save_refresh_token(
                        &authenticated.minecraft_username,
                        &authenticated.refresh_token,
                    );

                    self.account = Some(accounts::Account {
                        username: authenticated.minecraft_username,
                        offline: false,
                        uuid: authenticated.uuid,
                        access_token: authenticated.minecraft_access_token,
                    });
                    self.set_home_status("Account ready.");
                    Task::done(Message::LaunchStageCheckJava)
                }
                Err(err) => {
                    self.launch_settings = None;
                    self.launch_ready = false;
                    self.set_home_status(format!("Microsoft authentication failed: {err}"));
                    Task::none()
                }
            },
            Message::LaunchStageCheckJava => {
                let Some(account) = self.account.clone() else {
                    self.set_home_status("Authentication is required before launching.");
                    return Task::none();
                };

                let version = self.config.current_version.trim().to_string();
                if version.is_empty() {
                    self.set_home_status("Select a Minecraft version first.");
                    return Task::none();
                }

                let java_type = match crate::java::resolve_java_type(&self.config.java, &version) {
                    Ok(java_type) => java_type,
                    Err(error) => {
                        self.set_home_status(format!(
                            "Could not determine Java version: {error:#}"
                        ));
                        return Task::none();
                    }
                };
                let settings = launcher::GameSettings {
                    account,
                    game_version: version.clone(),
                    ram_mib: self.config.ram_mib,
                    game_directory: self.config.current_instance.clone(),
                    java_type: java_type.clone(),
                    custom_java_path: (!self.config.current_java.is_empty())
                        .then(|| self.config.current_java.clone()),
                    jvm_args: Vec::new(),
                    wrapper_commands: self.config.wrapper_commands.clone(),
                    environment_variables: self.config.environment_variables.clone(),
                };
                self.launch_settings = Some(settings);

                if matches!(java_type, JavaType::Custom) {
                    self.set_home_status("Using custom Java.");
                    return Task::perform(
                        async move { launcher::checker::check_version(&version).await },
                        Message::LaunchStageCheckVersion,
                    );
                }

                self.java_request = Some(java_type);
                self.set_home_status("Preparing Java...");
                Task::none()
            }
            Message::JavaDownload(_request, progress) => {
                let status = match progress {
                    java_downloader::Progress::Started => "Downloading Java...".to_string(),
                    java_downloader::Progress::Downloaded { bytes, .. } => {
                        format!("Downloaded Java archive ({bytes} bytes)...")
                    }
                    java_downloader::Progress::Finished => {
                        self.java_request = None;
                        let version = self.config.current_version.clone();
                        return Task::perform(
                            async move { launcher::checker::check_version(&version).await },
                            Message::LaunchStageCheckVersion,
                        );
                    }
                    java_downloader::Progress::Errored(ref error) => {
                        self.java_request = None;
                        format!("Java error: {error}")
                    }
                };
                let errored = matches!(progress, java_downloader::Progress::Errored(_));
                self.set_home_status(status);
                if errored {
                    self.launch_settings = None;
                    self.launch_ready = false;
                }
                Task::none()
            }
            Message::LaunchStageCheckVersion(check_result) => match check_result {
                Ok(launcher::checker::CheckResult::Ready(_)) => {
                    self.launch_id = self.launch_id.wrapping_add(1);
                    self.launch_ready = true;
                    self.set_home_status("Launching Minecraft...");
                    Task::none()
                }
                Ok(launcher::checker::CheckResult::NeedsVanillaJson { version, .. }) => {
                    println!("Vanilla JSON is missing. Downloading...");
                    self.version_request = Some(version_downloader::VersionRequest {
                        version,
                        version_type: version_downloader::VersionType::Vanilla,
                        fabric_loader: None,
                        neoforge_loader: None,
                    });
                    self.set_home_status("Downloading Minecraft files...");
                    Task::none()
                }
                Ok(launcher::checker::CheckResult::NeedsDownload {
                    missing_files,
                    metadata,
                }) => {
                    println!("Minecraft files are missing. Downloading...");
                    let is_neoforge = metadata.neoforge_loader_version().is_some()
                        || metadata.main_class.contains("forgewrapper");
                    let configured_version = self.config.current_version.clone();
                    let base_version = configured_version
                        .strip_suffix("-neoforge")
                        .unwrap_or(&configured_version)
                        .to_string();
                    let is_fabric = metadata.fabric_loader_version().is_some()
                        || metadata.main_class.contains("fabric");
                    self.version_request = if is_neoforge {
                        let Some(loader) = metadata.neoforge_loader_version() else {
                            self.launch_settings = None;
                            self.launch_ready = false;
                            self.set_home_status("The NeoForge version is invalid.");
                            return Task::none();
                        };
                        Some(version_downloader::VersionRequest {
                            version: base_version,
                            version_type: version_downloader::VersionType::NeoForge,
                            fabric_loader: None,
                            neoforge_loader: Some(loader),
                        })
                    } else if is_fabric {
                        let Some(loader) = metadata.fabric_loader_version() else {
                            self.launch_settings = None;
                            self.launch_ready = false;
                            self.set_home_status("The fabric version is invalid.");
                            return Task::none();
                        };
                        Some(version_downloader::VersionRequest {
                            version: base_version,
                            version_type: version_downloader::VersionType::Fabric,
                            fabric_loader: Some(loader),
                            neoforge_loader: None,
                        })
                    } else {
                        self.direct_download_files = Some(missing_files);
                        Some(version_downloader::VersionRequest {
                            version: configured_version,
                            version_type: version_downloader::VersionType::Vanilla,
                            fabric_loader: None,
                            neoforge_loader: None,
                        })
                    };
                    self.set_home_status("Downloading Minecraft files...");
                    Task::none()
                }
                Err(error) => {
                    self.launch_settings = None;
                    self.launch_ready = false;
                    self.set_home_status(format!("Version check error: {error:#}"));
                    Task::none()
                }
            },
            Message::AvailableVersionsLoaded(result) => {
                match result {
                    Ok(available) => {
                        self.available_versions = Some(available);
                    }
                    Err(error) => {
                        self.version_download_status =
                            Some(format!("Failed to load versions: {error}"));
                    }
                }
                Task::none()
            }
            Message::VersionDownload(_request, progress) => {
                let is_launching = self.launch_settings.is_some();
                match progress {
                    version_downloader::Progress::Started { total } => {
                        let status = format!("Preparing {total} files...");
                        self.version_download_status = Some(status.clone());
                        if is_launching {
                            self.set_home_status(status);
                        }
                    }
                    version_downloader::Progress::Downloaded { completed, total } => {
                        let status = format!("Downloaded {completed}/{total} Minecraft files...");
                        self.version_download_status = Some(status.clone());
                        if is_launching {
                            self.set_home_status(status);
                        }
                    }
                    version_downloader::Progress::Finished => {
                        self.version_request = None;
                        self.direct_download_files = None;
                        self.version_download_status = Some("Installation finished.".to_string());
                        let refresh_task = Task::perform(
                            minecraft::get_installed_versions(),
                            Message::GotInstalledVersions,
                        );

                        if is_launching {
                            let version = self.config.current_version.clone();
                            return refresh_task.chain(Task::perform(
                                async move { launcher::checker::check_version(&version).await },
                                Message::LaunchStageCheckVersion,
                            ));
                        }
                        return refresh_task;
                    }
                    version_downloader::Progress::Errored(error) => {
                        self.version_request = None;
                        self.direct_download_files = None;
                        let status = format!("Minecraft download error: {error}");
                        self.version_download_status = Some(status.clone());
                        if is_launching {
                            self.launch_settings = None;
                            self.launch_ready = false;
                            self.set_home_status(status);
                        }
                    }
                }
                Task::none()
            }
            Message::LauncherProgress(progress) => {
                match progress {
                    launcher::Progress::Checked(_) => {}
                    launcher::Progress::Started(process) => {
                        self.active_process = Some(process);
                        self.set_home_status("Minecraft is running.");
                    }
                    launcher::Progress::GotLog(line) => {
                        println!("[Minecraft] {line}");
                        self.minecraft_logs.push(line);
                    }
                    launcher::Progress::Finished => {
                        self.active_process = None;
                        self.launch_settings = None;
                        self.launch_ready = false;
                        self.set_home_status("Minecraft exited.");
                    }
                    launcher::Progress::Errored(error) => {
                        self.active_process = None;
                        self.launch_settings = None;
                        self.launch_ready = false;
                        self.set_home_status(format!("Launch error: {error}"));
                    }
                }
                Task::none()
            }
            Message::StopMinecraft => {
                if let Some(process) = &self.active_process {
                    if let Err(error) = process.kill() {
                        self.set_home_status(format!("Failed to stop Minecraft: {error}"));
                    } else {
                        self.set_home_status("Stopping Minecraft...");
                    }
                }
                Task::none()
            }
            Message::SettingsScreen(screen_message) => {
                if let Screen::Settings(settings) = &mut self.screen {
                    let task = settings.update(screen_message);
                    settings.apply_to_config(&mut self.config);
                    if let Err(error) = self.config.save() {
                        eprintln!("Failed to save configuration: {error:#}");
                    }
                    task.map(Message::SettingsScreen)
                } else {
                    Task::none()
                }
            }
            Message::VersionManagerScreen(message) => {
                if let Screen::VersionManager(version_manager) = &mut self.screen {
                    match message {
                        screen::version_manager::Message::InstallVanilla => {
                            if let Some(version) = &version_manager.selected_version {
                                self.version_request = Some(version_downloader::VersionRequest {
                                    version: version.clone(),
                                    version_type: version_downloader::VersionType::Vanilla,
                                    fabric_loader: None,
                                    neoforge_loader: None,
                                });
                                self.version_download_status =
                                    Some(format!("Preparing installation for {version}..."));
                            }
                            Task::none()
                        }
                        screen::version_manager::Message::InstallFabric => {
                            if let Some(version) = &version_manager.selected_version {
                                if let Some(available) = &self.available_versions {
                                    self.version_request =
                                        Some(version_downloader::VersionRequest {
                                            version: version.clone(),
                                            version_type: version_downloader::VersionType::Fabric,
                                            fabric_loader: Some(available.fabric_loader.clone()),
                                            neoforge_loader: None,
                                        });
                                    self.version_download_status = Some(format!(
                                        "Preparing Fabric installation for {version}..."
                                    ));
                                } else {
                                    self.version_download_status = Some(
                                        "Fabric loader information is unavailable.".to_string(),
                                    );
                                }
                            }
                            Task::none()
                        }
                        screen::version_manager::Message::InstallNeoForge => {
                            if let Some(version) = &version_manager.selected_version {
                                if let Some(available) = &self.available_versions {
                                    if let Some(loader) = available.neoforge_loaders.get(version) {
                                        self.version_request =
                                            Some(version_downloader::VersionRequest {
                                                version: version.clone(),
                                                version_type:
                                                    version_downloader::VersionType::NeoForge,
                                                fabric_loader: None,
                                                neoforge_loader: Some(loader.clone()),
                                            });
                                        self.version_download_status = Some(format!(
                                            "Preparing NeoForge installation for {version}..."
                                        ));
                                    } else {
                                        self.version_download_status = Some(
                                            "NeoForge is unavailable for this Minecraft version."
                                                .to_string(),
                                        );
                                    }
                                } else {
                                    self.version_download_status = Some(
                                        "NeoForge version information is unavailable.".to_string(),
                                    );
                                }
                            }
                            Task::none()
                        }
                        other => version_manager
                            .update(other)
                            .map(Message::VersionManagerScreen),
                    }
                } else {
                    Task::none()
                }
            }
            Message::AboutScreen(message) => {
                if let Screen::About(about) = &mut self.screen {
                    match message {
                        screen::about::Message::CheckForUpdates => {
                            self.checking_for_updates = true;
                            self.update_status = None;
                            Task::perform(updater::check_update(), |result| {
                                Message::UpdateCheckCompleted(match result {
                                    Ok(update) => Ok(Some(update)),
                                    Err(error)
                                        if error.to_string()
                                            == "Last version is already installed." =>
                                    {
                                        Ok(None)
                                    }
                                    Err(error) => Err(format!("{error:#}")),
                                })
                            })
                        }
                        screen::about::Message::InstallUpdate => {
                            if let Some((_version, url)) = &self.available_update {
                                self.update_status = Some("Installing update...".to_string());
                                Task::perform(updater::install_update(url.clone()), |result| {
                                    Message::UpdateInstalled(
                                        result.map_err(|error| format!("{error:#}")),
                                    )
                                })
                            } else {
                                Task::none()
                            }
                        }
                        other => about.update(other).map(Message::AboutScreen),
                    }
                } else {
                    Task::none()
                }
            }
            Message::AccountManagerScreen(message) => {
                if let Screen::AccountManager(account_manager) = &mut self.screen {
                    match message {
                        screen::account_manager::Message::AddOfflineAccountClicked(username) => {
                            //self.accounts.push(accounts::Account::new_offline(name.clone()));
                            self.config.accounts.push(accounts::AccountInfo {
                                username,
                                offline: true,
                            });
                            self.config.save().unwrap_or_else(|error| {
                                eprintln!(
                                    "Failed to save configuration after adding account: {error:#}"
                                );
                            });
                            Task::none()
                        }
                        screen::account_manager::Message::AddMicrosoftAccountClicked => {
                            self.microsoft_login_id = self.microsoft_login_id.wrapping_add(1);
                            let login_id = self.microsoft_login_id;
                            self.microsoft_login_state =
                                Some(screen::account_manager::MicrosoftLoginState::RequestingCode);
                            Task::perform(auth::begin_device_flow(), move |res| {
                                Message::MicrosoftDeviceFlowStarted(
                                    login_id,
                                    res.map_err(|e| e.to_string()),
                                )
                            })
                        }
                        screen::account_manager::Message::CancelMicrosoftLoginClicked => {
                            self.microsoft_login_id = self.microsoft_login_id.wrapping_add(1);
                            self.microsoft_login_state = None;
                            Task::none()
                        }
                        screen::account_manager::Message::RemoveAccount(username) => {
                            let _ = auth::delete_refresh_token(&username);
                            self.config
                                .accounts
                                .retain(|account| account.username != username);

                            if self.config.current_account == username {
                                self.config.current_account.clear();
                            }
                            if self
                                .account
                                .as_ref()
                                .is_some_and(|account| account.username == username)
                            {
                                self.account = None;
                            }

                            if let Err(error) = self.config.save() {
                                eprintln!(
                                    "Failed to save configuration after removing account: {error:#}"
                                );
                            }

                            Task::none()
                        }
                        _ => account_manager
                            .update(message)
                            .map(Message::AccountManagerScreen),
                    }
                } else {
                    Task::none()
                }
            }
            Message::ModManagerScreen(message) => {
                if let Screen::ModManager(mod_manager) = &mut self.screen {
                    if let screen::mod_manager::Message::InstanceSelected(instance) = &message {
                        self.config.current_instance = if instance == "Default" {
                            String::new()
                        } else {
                            instance.clone()
                        };
                        if let Err(error) = self.config.save() {
                            eprintln!("Failed to save configuration: {error:#}");
                        }
                    }
                    mod_manager.update(message).map(Message::ModManagerScreen)
                } else {
                    Task::none()
                }
            }
            Message::GotInstalledVersions(versions) => {
                self.versions = versions;
                Task::none()
            }
            Message::UpdateCheckCompleted(result) => {
                self.checking_for_updates = false;
                match result {
                    Ok(Some((version, url))) => {
                        self.update_status = Some(format!("Update v{} is available.", version));
                        self.available_update = Some((version, url));
                    }
                    Ok(None) => {
                        self.available_update = None;
                        self.update_status = Some("You are up to date.".to_string());
                    }
                    Err(error) => {
                        self.available_update = None;
                        self.update_status = Some(format!("Update check failed: {error}"));
                    }
                }
                Task::none()
            }
            Message::UpdateInstalled(result) => match result {
                Ok(()) => match updater::replace_and_restart() {
                    Ok(()) => std::process::exit(0),
                    Err(error) => {
                        self.update_status = Some(format!("Update failed: {error:#}"));
                        Task::none()
                    }
                },
                Err(error) => {
                    self.update_status = Some(format!("Update failed: {error}"));
                    Task::none()
                }
            },
        }
    }

    fn set_home_status(&mut self, status: impl Into<String>) {
        if let Screen::Home(home) = &mut self.screen {
            home.set_java_status(Some(status.into()));
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let active_screen = match &self.screen {
            Screen::Home(_) => ActiveScreen::Home,
            Screen::Settings(_) => ActiveScreen::Settings,
            Screen::VersionManager(_) => ActiveScreen::VersionManager,
            Screen::About(_) => ActiveScreen::About,
            Screen::AccountManager(_) => ActiveScreen::AccountManager,
            Screen::ModManager(_) => ActiveScreen::ModManager,
        };
        let sidebar = self
            .sidebar
            .view(
                active_screen,
                self.config.check_for_updates && self.available_update.is_some(),
            )
            .map(Message::Sidebar);

        let view = match &self.screen {
            Screen::Home(home) => home
                .view(
                    &self.config.accounts,
                    &self.versions,
                    self.active_process.is_some(),
                    self.logs_visible,
                    &self.minecraft_logs,
                )
                .map(Message::HomeScreen),
            Screen::Settings(settings) => settings.view().map(Message::SettingsScreen),
            Screen::VersionManager(version_manager) => version_manager
                .view(
                    &self.versions,
                    self.available_versions.as_ref(),
                    self.version_request.is_some(),
                    self.version_download_status.as_deref(),
                )
                .map(Message::VersionManagerScreen),
            Screen::About(about) => about
                .view(
                    self.config.check_for_updates,
                    self.checking_for_updates,
                    self.available_update.is_some(),
                    self.update_status.clone(),
                )
                .map(Message::AboutScreen),
            Screen::AccountManager(account_manager) => account_manager
                .view(&self.config.accounts, self.microsoft_login_state.as_ref())
                .map(Message::AccountManagerScreen),
            Screen::ModManager(mod_manager) => {
                let instances = minecraft::get_instances().unwrap_or_else(|error| {
                    eprintln!("Failed to load instances: {error:#}");
                    vec!["Default".to_string()]
                });
                mod_manager
                    .view(&instances, &self.config.current_instance)
                    .map(Message::ModManagerScreen)
            }
        };

        row![sidebar, view].spacing(20.).into()
    }

    fn theme(&self) -> Theme {
        match self.config.theme {
            theme::Themes::CatppuccinMocha => Theme::CatppuccinMocha,
            theme::Themes::Nightfly => Theme::Nightfly,
            theme::Themes::Dracula => Theme::Dracula,
            theme::Themes::GruvboxDark => Theme::GruvboxDark,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();

        if let Some(request) = &self.java_request {
            subscriptions.push(
                java_downloader::download(request.clone())
                    .map(|(request, progress)| Message::JavaDownload(request, progress)),
            );
        }
        if let Some(request) = &self.version_request {
            let subscription = if let Some(files) = &self.direct_download_files {
                version_downloader::download_files(request.clone(), files.clone())
            } else {
                version_downloader::download(request.clone())
            };
            subscriptions.push(
                subscription.map(|(request, progress)| Message::VersionDownload(request, progress)),
            );
        }
        if self.launch_ready {
            if let Some(settings) = &self.launch_settings {
                subscriptions.push(
                    launcher::start(self.launch_id, Some(settings))
                        .map(|(_, progress)| Message::LauncherProgress(progress)),
                );
            }
        }

        Subscription::batch(subscriptions)
    }
}
