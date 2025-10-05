#[derive(Debug, Clone, PartialEq)]
pub struct GameId {
    pub name: &'static str,
    pub exe: &'static str,
    pub steam_app_id: u32,
}

impl GameId {
    pub const ELDEN_RING: Self = Self::new("ELDEN RING", "Game/eldenring.exe", 1245620);
    pub const NIGHTREIGN: Self =
        Self::new("ELDEN RING: NIGHTREIGN", "Game/nightreign.exe", 2622380);

    pub const fn new(name: &'static str, exe: &'static str, steam_app_id: u32) -> Self {
        GameId {
            name,
            exe,
            steam_app_id,
        }
    }
}
