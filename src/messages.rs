use crate::{launcher, screen, widgets};
use anyhow::Result;

#[derive(Debug)]
pub enum Message {
    HomeScreen(screen::home::Message),
    SettingsScreen(screen::settings::Message),
    Sidebar(widgets::sidebar::Message),
    VersionManagerScreen(screen::version_manager::Message),
    AboutScreen(screen::about::Message),
    AccountManagerScreen(screen::account_manager::Message),
    ModManagerScreen(screen::mod_manager::Message),

    GotInstalledVersions(Vec<String>),
    AvailableVersionsLoaded(
        Result<crate::downloader::version_downloader::AvailableVersions, String>,
    ),
    JavaDownload(
        crate::java::JavaType,
        crate::downloader::java_downloader::Progress,
    ),
    VersionDownload(
        crate::downloader::version_downloader::VersionRequest,
        crate::downloader::version_downloader::Progress,
    ),

    LaunchStageAuthenticate,
    MicrosoftDeviceFlowStarted(
        usize,
        Result<(crate::auth::DeviceCode, crate::auth::DeviceFlow), String>,
    ),
    MicrosoftAuthCompleted(usize, Result<crate::auth::AuthenticatedAccount, String>),
    MicrosoftLaunchAuthCompleted(Result<crate::auth::AuthenticatedAccount, String>),
    LaunchStageCheckJava,
    LaunchStageCheckVersion(Result<launcher::checker::CheckResult, anyhow::Error>),
    LauncherProgress(crate::launcher::Progress),
    StopMinecraft,
}
