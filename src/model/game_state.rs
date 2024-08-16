use anyhow::{anyhow, bail, Context};
use pallas::ledger::primitives::{
    alonzo,
    conway::{Constr, PlutusData},
};

#[derive(Debug, Clone)]
pub struct GameState {
    pub is_over: bool,
    pub owner: Vec<u8>,
    pub admin: Vec<u8>,
    pub player: Player,
    #[allow(dead_code)]
    pub monsters: Vec<MapObject>,
    pub leveltime: Vec<u64>,
    pub level: LevelId,
}

#[derive(Debug, Clone)]
pub struct Player {
    player_state: PlayerState,
    map_object: MapObject,
    pub level_stats: PlayerStats,
    pub total_stats: PlayerStats,
    pub cheats: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerStats {
    pub kill_count: u64,
    pub secret_count: u64,
    pub item_count: u64,
}

#[derive(Debug, Clone)]
pub struct MapObject {
    position: Position,
    health: u64,
}

#[derive(Debug, Clone)]
pub struct Position {
    x: i64,
    y: i64,
    z: i64,
}

#[derive(Debug, Clone)]
pub struct LevelId {
    map: i64,
    skill: i64,
    episode: i64,
    pub demo_playback: bool,
}

#[derive(Debug, Clone)]
pub enum PlayerState {
    LIVE,
    DEAD,
    REBORN,
}

impl Into<PlutusData> for GameState {
    fn into(self) -> PlutusData {
        let is_over = if self.is_over {
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: Some(1),
                fields: vec![],
            })
        } else {
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: Some(0),
                fields: vec![],
            })
        };

        let owner_bytes: alonzo::BoundedBytes = self.owner.into();
        let owner = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![PlutusData::BoundedBytes(owner_bytes)],
        });

        let admin_bytes: alonzo::BoundedBytes = self.admin.into();
        let admin = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![PlutusData::BoundedBytes(admin_bytes)],
        });

        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                is_over,
                owner,
                admin,
                self.player.into(),
                PlutusData::Array(vec![]),
                PlutusData::Array(
                    self.leveltime
                        .into_iter()
                        .map(|x| {
                            let x = x as i64;
                            PlutusData::BigInt(alonzo::BigInt::Int(x.into()))
                        })
                        .collect(),
                ),
                self.level.into(),
            ],
        })
    }
}

impl TryFrom<PlutusData> for GameState {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let is_over = match constr.fields[0].clone() {
                    PlutusData::Constr(constr) => {
                        if constr.tag == 122 {
                            true
                        } else {
                            false
                        }
                    }
                    _ => bail!("Invalid is_over"),
                };

                let owner: Vec<u8> = match constr.fields[1].clone() {
                    PlutusData::Constr(constr) => {
                        let owner_bytes = match constr.fields[0].clone() {
                            PlutusData::BoundedBytes(bytes) => bytes,
                            _ => bail!("Invalid owner bytes"),
                        };

                        owner_bytes.into()
                    }
                    _ => bail!("Invalid owner"),
                };

                let admin: Vec<u8> = match constr.fields[2].clone() {
                    PlutusData::Constr(constr) => {
                        let admin_bytes = match constr.fields[0].clone() {
                            PlutusData::BoundedBytes(bytes) => bytes,
                            _ => bail!("Invalid admin bytes"),
                        };

                        admin_bytes.into()
                    }
                    _ => bail!("Invalid admin"),
                };

                let player = match constr.fields[3].clone() {
                    PlutusData::Constr(constr) => Player::try_from(PlutusData::Constr(constr))?,
                    _ => bail!("Invalid player"),
                };

                let monsters = match constr.fields[4].clone() {
                    PlutusData::Array(array) => {
                        let mut monsters = vec![];
                        for monster in array {
                            monsters.push(MapObject::try_from(monster)?)
                        }
                        monsters
                    }
                    _ => bail!("Invalid monsters"),
                };

                let leveltime = match constr.fields[5].clone() {
                    PlutusData::Array(array) => {
                        let mut leveltime = vec![];
                        for time in array {
                            match time {
                                PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                                    leveltime.push(u64::try_from(v.0)?)
                                }
                                _ => bail!("Invalid leveltime value"),
                            }
                        }
                        leveltime
                    }
                    _ => bail!("Invalid leveltime"),
                };

                let level = match constr.fields[6].clone() {
                    PlutusData::Constr(constr) => {
                        LevelId::try_from(PlutusData::Constr(constr)).context("level_id")?
                    }
                    _ => bail!("Invalid level"),
                };

                Ok(GameState {
                    is_over,
                    owner,
                    admin,
                    player,
                    monsters,
                    leveltime,
                    level,
                })
            }
            _ => Err(anyhow!("Invalid PlutusData variant")),
        }
    }
}

impl GameState {
    pub fn new(owner: Vec<u8>, admin: Vec<u8>) -> GameState {
        GameState {
            is_over: false,
            owner,
            admin,
            player: Player::new(),
            monsters: Vec::new(),
            leveltime: Vec::new(),
            level: LevelId::default(),
        }
    }
}

impl Player {
    pub fn new() -> Player {
        Player {
            player_state: PlayerState::LIVE,
            map_object: MapObject::default(),
            level_stats: PlayerStats::default(),
            total_stats: PlayerStats::default(),
            cheats: 0,
        }
    }
}

impl Into<PlutusData> for Player {
    fn into(self) -> PlutusData {
        let cheats = self.cheats as i64;
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                self.player_state.into(),
                self.map_object.into(),
                self.total_stats.into(),
                self.level_stats.into(),
                PlutusData::BigInt(alonzo::BigInt::Int(cheats.into())),
            ],
        })
    }
}

impl TryFrom<PlutusData> for Player {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let fields = constr.fields;
                if fields.len() != 5 {
                    bail!("Invalid number of fields");
                }

                let player_state = match fields[0].clone() {
                    PlutusData::Constr(constr) => {
                        PlayerState::try_from(PlutusData::Constr(constr))?
                    }
                    _ => bail!("Invalid field type"),
                };

                let map_object = match fields[1].clone() {
                    PlutusData::Constr(constr) => MapObject::try_from(PlutusData::Constr(constr))?,
                    _ => bail!("Invalid field type"),
                };

                let total_stats = match fields[2].clone() {
                    PlutusData::Constr(constr) => {
                        PlayerStats::try_from(PlutusData::Constr(constr))?
                    }
                    _ => bail!("Invalid field type"),
                };

                let level_stats = match fields[3].clone() {
                    PlutusData::Constr(constr) => {
                        PlayerStats::try_from(PlutusData::Constr(constr))?
                    }
                    _ => bail!("Invalid field type"),
                };

                let cheats = match fields[4] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                        let r = u128::try_from(v.0);
                        r.context(format!("cheats: {:?}", v.0))?
                    }
                    _ => bail!("Invalid field type"),
                };

                Ok(Player {
                    player_state,
                    map_object,
                    total_stats,
                    level_stats,
                    cheats,
                })
            }
            _ => Err(anyhow!("Invalid PlutusData type")),
        }
    }
}

impl Into<PlutusData> for PlayerStats {
    fn into(self) -> PlutusData {
        let kill_count = self.kill_count as i64;
        let secret_count = self.secret_count as i64;
        let item_count = self.item_count as i64;
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                PlutusData::BigInt(alonzo::BigInt::Int(kill_count.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(secret_count.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(item_count.into())),
            ],
        })
    }
}

impl TryFrom<PlutusData> for PlayerStats {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let fields = constr.fields;
                if fields.len() != 3 {
                    bail!("Invalid number of fields");
                }

                let kill_count = match fields[0] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => u64::try_from(v.0)?,
                    _ => bail!("Invalid field type"),
                };

                let secret_count = match fields[1] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => u64::try_from(v.0)?,
                    _ => bail!("Invalid field type"),
                };

                let item_count = match fields[2] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => u64::try_from(v.0)?,
                    _ => bail!("Invalid field type"),
                };

                Ok(PlayerStats {
                    kill_count,
                    secret_count,
                    item_count,
                })
            }
            _ => Err(anyhow!("Invalid PlutusData type")),
        }
    }
}

impl Default for MapObject {
    fn default() -> Self {
        MapObject {
            position: Position::default(),
            health: 100,
        }
    }
}

impl Into<PlutusData> for MapObject {
    fn into(self) -> PlutusData {
        let health: i64 = self.health as i64;
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                self.position.into(),
                PlutusData::BigInt(alonzo::BigInt::Int(health.into())),
            ],
        })
    }
}

impl TryFrom<PlutusData> for MapObject {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let fields = constr.fields;
                if fields.len() != 2 {
                    bail!("Invalid number of fields");
                }

                let position = match fields[0].clone() {
                    PlutusData::Constr(constr) => Position::try_from(PlutusData::Constr(constr))?,
                    _ => bail!("Invalid field type"),
                };

                let health = match fields[1] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => u64::try_from(v.0)?,
                    _ => bail!("Invalid field type"),
                };

                Ok(MapObject { position, health })
            }
            _ => Err(anyhow!("Invalid PlutusData type")),
        }
    }
}
impl Default for Position {
    fn default() -> Self {
        Position { x: 0, y: 0, z: 0 }
    }
}

impl Into<PlutusData> for Position {
    fn into(self) -> PlutusData {
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                PlutusData::BigInt(alonzo::BigInt::Int(self.x.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(self.y.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(self.z.into())),
            ],
        })
    }
}

impl TryFrom<PlutusData> for Position {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let fields = constr.fields;
                if fields.len() != 3 {
                    bail!("Invalid number of fields");
                }

                let x = match fields[0] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                        i64::try_from(v.0).context("invalid x")?
                    }
                    _ => bail!("Invalid field type"),
                };

                let y = match fields[1] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                        i64::try_from(v.0).context("invalid y")?
                    }
                    _ => bail!("Invalid field type"),
                };

                let z = match fields[2] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                        i64::try_from(v.0).context("invalid z")?
                    }
                    _ => bail!("Invalid field type"),
                };

                Ok(Position { x, y, z })
            }
            _ => Err(anyhow!("Invalid PlutusData type")),
        }
    }
}

impl Into<PlutusData> for PlayerState {
    fn into(self) -> PlutusData {
        PlutusData::Constr(match self {
            PlayerState::LIVE => Constr {
                tag: 121,
                any_constructor: Some(0),
                fields: vec![],
            },
            PlayerState::DEAD => Constr {
                tag: 121,
                any_constructor: Some(1),
                fields: vec![],
            },
            // Constr(1, [])
            PlayerState::REBORN => Constr {
                tag: 121,
                any_constructor: Some(2),
                fields: vec![],
            }, // Constr(2, [])
        })
    }
}

impl TryFrom<PlutusData> for PlayerState {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => match constr.tag {
                121 => Ok(PlayerState::LIVE),
                122 => Ok(PlayerState::DEAD),
                123 => Ok(PlayerState::REBORN),
                _ => Err(anyhow!("Invalid tag for PlayerState")),
            },
            _ => Err(anyhow!("Invalid PlutusData for PlayerState")),
        }
    }
}

impl Default for LevelId {
    fn default() -> Self {
        LevelId {
            map: -1,
            skill: -1,
            episode: -1,
            demo_playback: false,
        }
    }
}

impl TryFrom<PlutusData> for LevelId {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let fields = constr.fields;
                if fields.len() != 4 {
                    bail!("Invalid number of fields");
                }

                let map = match fields[0] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => i64::try_from(v.0)?,
                    _ => bail!("Invalid field type"),
                };

                let skill = match fields[1] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => i64::try_from(v.0)?,
                    _ => bail!("Invalid field type"),
                };

                let episode = match fields[2] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => i64::try_from(v.0)?,
                    _ => bail!("Invalid field type"),
                };

                let demo_playback = match fields[3].clone() {
                    PlutusData::Constr(constr) => {
                        if constr.tag == 122 {
                            true
                        } else {
                            false
                        }
                    }
                    _ => bail!("Invalid demoplayback"),
                };

                Ok(LevelId {
                    map,
                    skill,
                    episode,
                    demo_playback,
                })
            }
            _ => bail!("Invalid PlutusData for LevelId"),
        }
    }
}

impl Into<PlutusData> for LevelId {
    fn into(self) -> PlutusData {
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                PlutusData::BigInt(alonzo::BigInt::Int(self.map.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(self.skill.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(self.episode.into())),
                PlutusData::Constr(Constr {
                    tag: 121,
                    any_constructor: Some(if self.demo_playback { 1 } else { 0 }),
                    fields: vec![],
                }),
            ],
        })
    }
}
