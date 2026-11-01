use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Themes {
    CatppuccinMocha,
    Nightfly,
    Dracula,
    GruvboxDark,
}
