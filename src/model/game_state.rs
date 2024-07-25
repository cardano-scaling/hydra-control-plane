use anyhow::{anyhow, bail, Context};
use pallas::ledger::primitives::{
    alonzo,
    conway::{Constr, PlutusData},
};

#[derive(Debug, Clone)]
pub struct GameState {
    pub is_over: bool,
    pub owner: Vec<u8>,
    pub player: Player,
    #[allow(dead_code)]
    pub monsters: Vec<MapObject>,
}

#[derive(Debug, Clone)]
pub struct Player {
    player_state: PlayerState,
    map_object: MapObject,
    pub kill_count: u64,
}

#[derive(Debug, Clone)]
pub struct MapObject {
    position: Position,
    health: u64,
}

#[derive(Debug, Clone)]
pub struct Position {
    momentum_x: i64,
    momentum_y: i64,
    momentum_z: i64,
    angle: i64,
    z: i64,
    floor_z: i64,
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
                any_constructor: Some(0),
                fields: vec![],
            })
        } else {
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: Some(1),
                fields: vec![],
            })
        };

        let admin_bytes: alonzo::BoundedBytes = self.owner.into();

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
                admin,
                self.player.into(),
                PlutusData::Array(vec![]),
            ],
        })
    }
}

impl TryFrom<PlutusData> for GameState {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let is_over = if constr.any_constructor == Some(0) {
                    true
                } else {
                    false
                };

                let owner: Vec<u8> = match constr.fields[1].clone() {
                    PlutusData::Constr(constr) => {
                        let owner_bytes = match constr.fields[0].clone() {
                            PlutusData::BoundedBytes(bytes) => bytes,
                            _ => bail!("Invalid admin bytes"),
                        };

                        owner_bytes.into()
                    }
                    _ => bail!("Invalid admin"),
                };

                let player = match constr.fields[2].clone() {
                    PlutusData::Constr(constr) => Player::try_from(PlutusData::Constr(constr))?,
                    _ => bail!("Invalid player"),
                };

                let monsters = match constr.fields[3].clone() {
                    PlutusData::Array(array) => {
                        let mut monsters = vec![];
                        for monster in array {
                            monsters.push(MapObject::try_from(monster)?)
                        }
                        monsters
                    }
                    _ => bail!("Invalid monsters"),
                };

                Ok(GameState {
                    is_over,
                    owner,
                    player,
                    monsters,
                })
            }
            _ => Err(anyhow!("Invalid PlutusData variant")),
        }
    }
}

impl GameState {
    pub fn new(owner: Vec<u8>) -> GameState {
        GameState {
            is_over: false,
            owner,
            player: Player::new(),
            monsters: Vec::new(),
        }
    }
}

impl Player {
    pub fn new() -> Player {
        Player {
            player_state: PlayerState::LIVE,
            map_object: MapObject::default(),
            kill_count: 0,
        }
    }
}

impl Into<PlutusData> for Player {
    fn into(self) -> PlutusData {
        let kill_count = self.kill_count as i64;
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                self.player_state.into(),
                self.map_object.into(),
                PlutusData::BigInt(alonzo::BigInt::Int(kill_count.into())),
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
                if fields.len() != 3 {
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

                let kill_count = match fields[2] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => u64::try_from(v.0)?,
                    _ => bail!("Invalid field type"),
                };

                Ok(Player {
                    player_state,
                    map_object,
                    kill_count,
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
    // TODO: Determine if this is correct, or if we should have "map based" defaults
    fn default() -> Self {
        Position {
            momentum_x: 0,
            momentum_y: 0,
            momentum_z: 0,
            angle: 0,
            z: 0,
            floor_z: 0,
        }
    }
}

impl Into<PlutusData> for Position {
    fn into(self) -> PlutusData {
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                PlutusData::BigInt(alonzo::BigInt::Int(self.momentum_x.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(self.momentum_y.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(self.momentum_z.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(self.angle.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(self.z.into())),
                PlutusData::BigInt(alonzo::BigInt::Int(self.floor_z.into())),
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
                if fields.len() != 6 {
                    bail!("Invalid number of fields");
                }

                let momentum_x = match fields[0] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                        i64::try_from(v.0).context("invalid momentum_x")?
                    }
                    _ => bail!("Invalid field type"),
                };

                let momentum_y = match fields[1] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                        i64::try_from(v.0).context("invalid momentum_y")?
                    }
                    _ => bail!("Invalid field type"),
                };

                let momentum_z = match fields[2] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                        i64::try_from(v.0).context("invalid momentum_z")?
                    }
                    _ => bail!("Invalid field type"),
                };

                let angle = match fields[3] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                        i64::try_from(v.0).context("Invalid angle")?
                    }
                    _ => bail!("Invalid field type"),
                };

                let z = match fields[4] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                        i64::try_from(v.0).context("invalid z")?
                    }
                    _ => bail!("Invalid field type"),
                };

                let floor_z = match fields[5] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => {
                        i64::try_from(v.0).context("Invalid floor_z")?
                    }
                    _ => bail!("Invalid field type"),
                };

                Ok(Position {
                    momentum_x,
                    momentum_y,
                    momentum_z,
                    angle,
                    z,
                    floor_z,
                })
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
            }, // Constr(0, [])
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
