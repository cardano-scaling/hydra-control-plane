use pallas::ledger::primitives::{
    alonzo,
    conway::{Constr, PlutusData},
};

pub struct GameData {
    is_over: bool,
    admin: Vec<u8>,
    player: Player,
    monsters: Vec<MapObject>,
}

pub struct Player {
    player_state: PlayerState,
    map_object: MapObject,
    kill_count: u32,
}

pub struct MapObject {
    position: Position,
    health: u32,
}

pub struct Position {
    momentum_x: i64,
    momentum_y: i64,
    momentum_z: i64,
    angle: i64,
    z: i64,
    floor_z: i64,
}

pub enum PlayerState {
    LIVE,
    DEAD,
    REBORN,
}

impl Into<PlutusData> for GameData {
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

impl GameData {
    pub fn new(admin: Vec<u8>) -> GameData {
        GameData {
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
        let kill_count: i64 = self.kill_count.into();
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
        let health: i64 = self.health.into();
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
