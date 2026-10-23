pub mod home;
pub mod settings;

pub enum Screen{
    Home(home::Home),
    Settings(settings::Settings),
    VersionManager,
    About
}