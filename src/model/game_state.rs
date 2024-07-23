use pallas::ledger::primitives::{
    alonzo,
    conway::{Constr, PlutusData},
};

#[derive(Debug, Clone)]
pub struct GameState {
    pub is_over: bool,
    pub admin: Vec<u8>,
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
                admin,
                self.player.into(),
                PlutusData::Array(vec![]),
            ],
        })
    }
}

impl TryFrom<PlutusData> for GameState {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let is_over = if constr.any_constructor == Some(0) {
                    true
                } else {
                    false
                };

                let admin: Vec<u8> = match constr.fields[1].clone() {
                    PlutusData::Constr(constr) => {
                        let admin_bytes = match constr.fields[0].clone() {
                            PlutusData::BoundedBytes(bytes) => bytes,
                            _ => return Err("Invalid admin bytes".into()),
                        };

                        admin_bytes.into()
                    }
                    _ => return Err("Invalid admin".into()),
                };

                let player = match constr.fields[2].clone() {
                    PlutusData::Constr(constr) => {
                        match Player::try_from(PlutusData::Constr(constr)) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        }
                    }
                    _ => return Err("Invalid player".into()),
                };

                let monsters = match constr.fields[3].clone() {
                    PlutusData::Array(array) => {
                        let mut monsters = vec![];
                        for monster in array {
                            match MapObject::try_from(monster) {
                                Ok(v) => monsters.push(v),
                                Err(e) => return Err(e),
                            }
                        }
                        monsters
                    }
                    _ => return Err("Invalid monsters".into()),
                };

                Ok(GameState {
                    is_over,
                    admin,
                    player,
                    monsters,
                })
            }
            _ => Err("Invalid PlutusData variant".into()),
        }
    }
}

impl GameState {
    pub fn new(admin: Vec<u8>) -> GameState {
        GameState {
            is_over: false,
            admin,
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
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let fields = constr.fields;
                if fields.len() != 3 {
                    return Err("Invalid number of fields".into());
                }

                let player_state = match fields[0].clone() {
                    PlutusData::Constr(constr) => {
                        match PlayerState::try_from(PlutusData::Constr(constr)) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        }
                    }
                    _ => return Err("Invalid field type".into()),
                };

                let map_object = match fields[1].clone() {
                    PlutusData::Constr(constr) => {
                        match MapObject::try_from(PlutusData::Constr(constr)) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        }
                    }
                    _ => return Err("Invalid field type".into()),
                };

                let kill_count = match fields[2] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => match u64::try_from(v.0) {
                        Ok(v) => v,
                        Err(_) => return Err("Invalid kill count value".into()),
                    },
                    _ => return Err("Invalid field type".into()),
                };

                Ok(Player {
                    player_state,
                    map_object,
                    kill_count,
                })
            }
            _ => Err("Invalid PlutusData type".into()),
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
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let fields = constr.fields;
                if fields.len() != 2 {
                    return Err("Invalid number of fields".into());
                }

                let position = match fields[0].clone() {
                    PlutusData::Constr(constr) => {
                        match Position::try_from(PlutusData::Constr(constr)) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        }
                    }
                    _ => return Err("Invalid field type".into()),
                };

                let health = match fields[1] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => match u64::try_from(v.0) {
                        Ok(v) => v,
                        Err(_) => return Err("Invalid health value".into()),
                    },
                    _ => return Err("Invalid field type".into()),
                };

                Ok(MapObject { position, health })
            }
            _ => Err("Invalid PlutusData type".into()),
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
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                let fields = constr.fields;
                if fields.len() != 6 {
                    return Err("Invalid number of fields".into());
                }

                let momentum_x = match fields[0] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => match i64::try_from(v.0) {
                        Ok(v) => v,
                        Err(_) => return Err("Invalid momentum_x value".into()),
                    },
                    _ => return Err("Invalid field type".into()),
                };

                let momentum_y = match fields[1] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => match i64::try_from(v.0) {
                        Ok(v) => v,
                        Err(_) => return Err("Invalid momentum_y value".into()),
                    },
                    _ => return Err("Invalid field type".into()),
                };

                let momentum_z = match fields[2] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => match i64::try_from(v.0) {
                        Ok(v) => v,
                        Err(_) => return Err("Invalid momentum_z value".into()),
                    },
                    _ => return Err("Invalid field type".into()),
                };

                let angle = match fields[3] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => match i64::try_from(v.0) {
                        Ok(v) => v,
                        Err(_) => return Err("Invalid angle value".into()),
                    },
                    _ => return Err("Invalid field type".into()),
                };

                let z = match fields[4] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => match i64::try_from(v.0) {
                        Ok(v) => v,
                        Err(_) => return Err("Invalid z value".into()),
                    },
                    _ => return Err("Invalid field type".into()),
                };

                let floor_z = match fields[5] {
                    PlutusData::BigInt(alonzo::BigInt::Int(v)) => match i64::try_from(v.0) {
                        Ok(v) => v,
                        Err(_) => return Err("Invalid floor_z value".into()),
                    },
                    _ => return Err("Invalid field type".into()),
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
            _ => Err("Invalid PlutusData type".into()),
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
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => match constr.tag {
                121 => Ok(PlayerState::LIVE),
                122 => Ok(PlayerState::DEAD),
                123 => Ok(PlayerState::REBORN),
                _ => Err("Invalid tag for PlayerState".into()),
            },
            _ => Err("Invalid PlutusData for PlayerState".into()),
        }
    }
}
