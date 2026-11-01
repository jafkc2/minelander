pub mod about;
pub mod account_manager;
pub mod home;
pub mod mod_manager;
pub mod settings;
pub mod version_manager;

pub enum Screen {
    Home(home::Home),
    Settings(settings::Settings),
    VersionManager(version_manager::VersionManager),
    About(about::About),
    AccountManager(account_manager::AccountManager),
    ModManager(mod_manager::ModManager),
}
