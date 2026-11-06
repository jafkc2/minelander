pub mod checker;
pub mod java_runtime;
pub mod library_builder;
pub mod process;
pub mod version_args;

use iced::{futures::SinkExt, Subscription};
use shared_child::SharedChild;
use std::{
    fs,
    hash::Hash,
    path::PathBuf,
    sync::{mpsc::TryRecvError, Arc},
};
use tokio::time::{sleep, Duration};

use crate::{
    accounts::Account,
    java::JavaType,
    launcher::process::ProcessConfig,
    minecraft::{get_instance_path, get_minecraft_dir},
};

#[derive(Debug, Clone)]
pub struct MinecraftData {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String,
    pub game_version: String,
    pub version_type: String,
    pub game_directory: String,
    pub assets_root: String,
    pub assets_index_name: String,
}

impl MinecraftData {
    pub fn replace_placeholders(&self, text: &str) -> String {
        text.replace("${auth_player_name}", &self.username)
            .replace("${version_name}", &self.game_version)
            .replace("${game_directory}", &self.game_directory)
            .replace("${assets_root}", &self.assets_root)
            .replace("${assets_index_name}", &self.assets_index_name)
            .replace("${auth_uuid}", &self.uuid)
            .replace("${auth_session}", &self.access_token)
            .replace("${auth_access_token}", &self.access_token)
            .replace("${clientid}", &self.uuid)
            .replace("${auth_xuid}", &self.uuid)
            .replace("${user_properties}", "{}")
            .replace("${user_type}", &self.user_type)
            .replace("${version_type}", &self.version_type)
            .replace(
                "${game_assets}",
                &format!("{}/resources", get_minecraft_dir()),
            )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameSettings {
    pub account: Account,
    pub game_version: String,
    pub ram_mib: u32,
    pub game_directory: String,
    pub java_type: JavaType,
    pub custom_java_path: Option<String>,
    pub jvm_args: Vec<String>,
    pub wrapper_commands: Vec<String>,
    pub environment_variables: Vec<(String, String)>,
}

pub enum State {
    Checking(Option<GameSettings>),
    Launching(GameSettings),
    GettingLogs(process::GameProcess),
    Idle,
}

#[derive(Debug, Clone)]
pub enum Progress {
    Checked(Option<checker::CheckResult>),
    Started(Arc<SharedChild>),
    GotLog(String),
    Finished,
    Errored(String),
}

pub fn start<I: 'static + Hash + Copy + Send + Sync + Eq>(
    id: I,
    game_settings: Option<&GameSettings>,
) -> Subscription<(I, Progress)> {
    let settings = game_settings.cloned();
    Subscription::run_with((id, settings), |(id, settings)| {
        let id = *id;
        let settings = settings.clone();
        iced::stream::channel(16, async move |mut output| {
            let mut current_state = State::Checking(settings);
            loop {
                let (prog, next_state) = launcher(current_state).await;
                let is_finished = matches!(prog, Progress::Finished | Progress::Errored(_));
                let _ = output.send((id, prog)).await;
                match next_state {
                    State::Idle => break,
                    _ => current_state = next_state,
                }
                if is_finished {
                    break;
                }
            }
        })
    })
}

async fn launcher(state: State) -> (Progress, State) {
    match state {
        State::Checking(settings_opt) => {
            let Some(settings) = settings_opt else {
                return (Progress::Finished, State::Idle);
            };

            match checker::check_version(&settings.game_version).await {
                Ok(checker::CheckResult::Ready(_meta)) => {
                    (Progress::Checked(None), State::Launching(settings))
                }
                Ok(needs_download) => (Progress::Checked(Some(needs_download)), State::Idle),
                Err(err) => (Progress::Errored(err.to_string()), State::Idle),
            }
        }

        State::Launching(settings) => {
            let mc_dir = PathBuf::from(get_minecraft_dir());

            let meta = match checker::load_version_metadata(&mc_dir, &settings.game_version) {
                Ok(m) => m,
                Err(e) => return (Progress::Errored(e.to_string()), State::Idle),
            };

            // instance directory
            let game_dir = if settings.game_directory.is_empty() {
                mc_dir.clone()
            } else {
                match get_instance_path(&settings.game_directory) {
                    Ok(path) => path,
                    Err(error) => return (Progress::Errored(error.to_string()), State::Idle),
                }
            };
            if let Err(e) = fs::create_dir_all(&game_dir) {
                return (Progress::Errored(e.to_string()), State::Idle);
            }

            // java path and flags
            let java_binary = match java_runtime::get_java_binary(
                &settings.java_type,
                settings.custom_java_path.as_deref(),
            ) {
                Ok(bin) => bin,
                Err(e) => return (Progress::Errored(e.to_string()), State::Idle),
            };

            let jvm_optimizations = if !settings.jvm_args.is_empty() {
                settings.jvm_args.clone()
            } else {
                java_runtime::get_jvm_optimizations(&settings.java_type)
            };

            // clssspath
            let mut libraries = library_builder::get_library_paths(&meta.json, &mc_dir);
            if let Some(ref vjson) = meta.vanilla_json {
                let mut vanilla_libs = library_builder::get_library_paths(vjson, &mc_dir);
                libraries.append(&mut vanilla_libs);
            }

            let client_jar = mc_dir
                .join("versions")
                .join(&settings.game_version)
                .join(format!("{}.jar", settings.game_version));

            let classpath = library_builder::build_classpath(&libraries, &client_jar);

            // jvm args
            let natives_dir = mc_dir
                .join("versions")
                .join(&settings.game_version)
                .join("natives");

            let mut version_jvm_args = version_args::get_version_specific_jvm_args(
                &meta.json,
                &natives_dir.to_string_lossy(),
            );

            if let Some(ref vjson) = meta.vanilla_json {
                let v_jvm_args = version_args::get_version_specific_jvm_args(
                    vjson,
                    &natives_dir.to_string_lossy(),
                );
                version_jvm_args.extend(v_jvm_args);
            }
            if meta.neoforge_loader_version().is_some() {
                let minecraft_jar = mc_dir
                    .join("versions")
                    .join(&settings.game_version)
                    .join(format!("{}.jar", settings.game_version));
                version_jvm_args.push(format!(
                    "-Dforgewrapper.minecraft={}",
                    minecraft_jar.to_string_lossy()
                ));
            }

            // minecraft args
            let data = MinecraftData {
                username: settings.account.username.clone(),
                uuid: settings.account.uuid.clone(),
                access_token: settings.account.access_token.clone(),
                user_type: if settings.account.offline {
                    "legacy".to_string()
                } else {
                    "msa".to_string()
                },
                game_version: meta
                    .inherits_from
                    .as_deref()
                    .unwrap_or(&settings.game_version)
                    .to_string(),
                version_type: "release".to_string(),
                game_directory: game_dir.to_string_lossy().to_string(),
                assets_root: mc_dir.join("assets").to_string_lossy().to_string(),
                assets_index_name: meta.asset_index.clone(),
            };

            let mut game_args = Vec::new();
            let child_has_complete_game_arguments = meta.json["minecraftArguments"].is_string();
            if !child_has_complete_game_arguments {
                if let Some(ref vjson) = meta.vanilla_json {
                    game_args.extend(version_args::get_minecraft_args(vjson, &data));
                }
            }
            game_args.extend(version_args::get_minecraft_args(&meta.json, &data));

            // process
            let process_config = ProcessConfig {
                java_binary,
                ram_mib: settings.ram_mib,
                game_dir,
                main_class: meta.main_class,
                classpath,
                jvm_optimizations,
                version_jvm_args,
                game_args,
                wrapper_commands: settings.wrapper_commands,
                env_vars: settings.environment_variables,
            };

            let cmd = match process::build_command(&process_config) {
                Ok(c) => c,
                Err(e) => return (Progress::Errored(e.to_string()), State::Idle),
            };

            match process::launch_process(cmd) {
                Ok(game_proc) => {
                    let child = game_proc.child.clone();
                    (Progress::Started(child), State::GettingLogs(game_proc))
                }
                Err(e) => (
                    Progress::Errored(format!("Failed to launch: {e}")),
                    State::Idle,
                ),
            }
        }

        State::GettingLogs(game_proc) => loop {
            match game_proc.log_receiver.try_recv() {
                Ok(line) => {
                    break (Progress::GotLog(line), State::GettingLogs(game_proc));
                }
                Err(TryRecvError::Disconnected) => {
                    let _ = game_proc.stdout_thread.join();
                    let _ = game_proc.stderr_thread.join();
                    break (Progress::Finished, State::Idle);
                }
                Err(TryRecvError::Empty) => match game_proc.child.try_wait() {
                    Ok(Some(_)) => sleep(Duration::from_millis(10)).await,
                    Ok(None) => sleep(Duration::from_millis(50)).await,
                    Err(error) => {
                        break (
                            Progress::Errored(format!(
                                "Failed to inspect Minecraft process: {error}"
                            )),
                            State::Idle,
                        );
                    }
                },
            }
        },

        State::Idle => iced::futures::future::pending().await,
    }
}
