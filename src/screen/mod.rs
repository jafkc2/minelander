pub mod home;

pub enum Screen{
    Home(home::Home),
    Settings,
    VersionManager,
    About
}