use anyhow::{anyhow, bail, Context};
use pallas::crypto::hash::Hash;
use pallas::ledger::{
    addresses::PaymentKeyHash,
    primitives::{
        alonzo,
        conway::{Constr, PlutusData},
    },
};

use crate::model::game::player::Player;
use crate::model::hydra::utxo::Datum;

#[derive(Debug, PartialEq, Eq)]
pub struct PaymentCredential([u8; 28]);

#[derive(Debug)]
pub enum State {
    Lobby,
    Running,
    Cheated,
    Finished,
    Aborted,
}
#[derive(Debug)]
pub struct GameState {
    referee: PaymentCredential,
    player_count: u64,
    bot_count: u64,
    pub players: Vec<PaymentCredential>,
    state: State,
    winner: Option<PaymentCredential>,
    cheater: Option<PaymentCredential>,
}

impl GameState {
    pub fn new(referee: PaymentCredential, player_count: u64, bot_count: u64) -> Self {
        Self {
            referee,
            player_count,
            bot_count,
            players: Vec::new(),
            state: State::Lobby,
            winner: None,
            cheater: None,
        }
    }

    pub fn add_player(mut self, player: PaymentCredential) -> Self {
        self.players.push(player);

        self
    }

    pub fn set_winner(mut self, winner: PaymentCredential) -> Self {
        self.winner = Some(winner);

        self
    }

    pub fn set_cheater(mut self, cheater: PaymentCredential) -> Self {
        self.cheater = Some(cheater);

        self
    }

    pub fn set_state(mut self, state: State) -> Self {
        self.state = state;

        self
    }
}

impl From<GameState> for PlutusData {
    fn from(value: GameState) -> Self {
        let x = value
            .players
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<_>>();

        let players: PlutusData = PlutusData::Array(alonzo::MaybeIndefArray::Indef(x));

        let winner: PlutusData = match value.winner {
            Some(winner) => PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Indef(vec![winner.into()]),
            }),
            None => PlutusData::Constr(Constr {
                tag: 122,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Def(vec![]),
            }),
        };

        let cheater: PlutusData = match value.cheater {
            Some(cheater) => PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Indef(vec![cheater.into()]),
            }),
            None => PlutusData::Constr(Constr {
                tag: 122,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Def(vec![]),
            }),
        };

        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: alonzo::MaybeIndefArray::Indef(vec![
                value.referee.into(),
                PlutusData::BigInt(alonzo::BigInt::Int((value.player_count as i64).into())),
                PlutusData::BigInt(alonzo::BigInt::Int((value.bot_count as i64).into())),
                players,
                value.state.into(),
                winner,
                cheater,
            ]),
        })
    }
}

impl TryFrom<Datum> for GameState {
    type Error = anyhow::Error;

    fn try_from(value: Datum) -> Result<Self, Self::Error> {
        match value {
            Datum::Inline(data) => data.try_into(),
            _ => bail!("invalid datum type"),
        }
    }
}

impl TryFrom<PlutusData> for GameState {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                if constr.tag != 121 {
                    bail!("Invalid constructor tag for GameState.");
                }

                if constr.fields.len() != 7 {
                    bail!("Invalid number of fields for GameState.");
                }

                let referee: PaymentCredential =
                    constr.fields[0].clone().try_into().context("referee")?;

                let player_count = match constr.fields[1].clone() {
                    PlutusData::BigInt(alonzo::BigInt::Int(int)) => u64::try_from(int.0)?,
                    _ => bail!("invalid player_count"),
                };

                let bot_count = match constr.fields[2].clone() {
                    PlutusData::BigInt(alonzo::BigInt::Int(int)) => u64::try_from(int.0)?,
                    _ => bail!("invalid bot_count"),
                };

                let players: Vec<PaymentCredential> = match constr.fields[3].clone() {
                    PlutusData::Array(array) => {
                        let mut players = Vec::new();
                        for player in array.to_vec() {
                            players.push(player.try_into().context("players")?);
                        }

                        players
                    }
                    _ => bail!("Invalid data type for players"),
                };

                let state: State = constr.fields[4].clone().try_into().context("state")?;

                let winner: Option<PaymentCredential> = match constr.fields[5].clone() {
                    PlutusData::Constr(constr) => {
                        if Some(0) == constr.any_constructor {
                            if constr.fields.len() != 1 {
                                bail!("invalid length for Just type");
                            }

                            match constr.fields[0].clone() {
                                PlutusData::Constr(constr) => Some(
                                    PaymentCredential::try_from(PlutusData::Constr(constr))
                                        .context("failed to get PaymentCredential for winner")?,
                                ),
                                _ => bail!("invalid inner type for Just<PaymentCredential>"),
                            }
                        } else if Some(1) == constr.any_constructor {
                            None
                        } else {
                            bail!("Invalid constructor for winner");
                        }
                    }
                    _ => bail!("Invalid data type for winner"),
                };

                let cheater: Option<PaymentCredential> = match constr.fields[6].clone() {
                    PlutusData::Constr(constr) => {
                        if Some(0) == constr.any_constructor {
                            if constr.fields.len() != 1 {
                                bail!("invalid length for Just type");
                            }

                            match constr.fields[0].clone() {
                                PlutusData::Constr(constr) => Some(
                                    PaymentCredential::try_from(PlutusData::Constr(constr))
                                        .context("failed to get PaymentCredential for cheater")?,
                                ),
                                _ => bail!("invalid inner type for Just<PaymentCredential>"),
                            }
                        } else if Some(1) == constr.any_constructor {
                            None
                        } else {
                            bail!("Invalid constructor tag for cheater");
                        }
                    }
                    _ => bail!("Invalid data type for cheater"),
                };

                Ok(GameState {
                    referee,
                    player_count,
                    bot_count,
                    players,
                    state,
                    winner,
                    cheater,
                })
            }
            _ => bail!("Invalid data type for GameState"),
        }
    }
}

impl From<PaymentKeyHash> for PaymentCredential {
    fn from(value: PaymentKeyHash) -> Self {
        // We can do this unsafe, because we we know PaymentKeyHash is 28 bytes long
        let ptr = value.as_ref().as_ptr() as *const [u8; 28];
        unsafe { PaymentCredential(*ptr) }
    }
}

impl From<PaymentCredential> for Hash<28> {
    fn from(value: PaymentCredential) -> Hash<28> {
        value.0.into()
    }
}

impl From<Player> for PaymentCredential {
    fn from(value: Player) -> Self {
        value.signing_key.into()
    }
}

impl From<PaymentCredential> for PlutusData {
    fn from(value: PaymentCredential) -> Self {
        let bytes: alonzo::BoundedBytes = value.0.to_vec().into();
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: alonzo::MaybeIndefArray::Indef(vec![PlutusData::BoundedBytes(bytes)]),
        })
    }
}

impl TryFrom<PlutusData> for PaymentCredential {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => {
                if constr.tag != 121 {
                    bail!("Invalid constructor tag for PaymentCredential.");
                }
                if constr.fields.len() != 1 {
                    bail!("Invalid number of fields for PaymentCredential.");
                }
                match constr.fields[0].clone() {
                    PlutusData::BoundedBytes(bytes) => {
                        let bytes: Vec<u8> = bytes.into();
                        if bytes.len() != 28 {
                            bail!("Invalid length for PaymentCredential.");
                        }

                        let credential: [u8; 28] = bytes
                            .try_into()
                            .map_err(|_| anyhow!("Failed to convert Vec<u8> to [u8; 28]"))?;

                        Ok(PaymentCredential(credential))
                    }
                    _ => bail!("Invalid field type for PaymentCredential."),
                }
            }
            _ => bail!("Invalid data type for PaymentCredential."),
        }
    }
}

impl From<State> for PlutusData {
    fn from(value: State) -> Self {
        PlutusData::Constr(match value {
            State::Lobby => Constr {
                tag: 121,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Def(vec![]),
            },
            State::Running => Constr {
                tag: 122,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Def(vec![]),
            },
            State::Cheated => Constr {
                tag: 123,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Def(vec![]),
            },
            State::Finished => Constr {
                tag: 124,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Def(vec![]),
            },
            State::Aborted => Constr {
                tag: 125,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Def(vec![]),
            },
        })
    }
}

impl TryFrom<PlutusData> for State {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => match constr.tag {
                121 => Ok(State::Lobby),
                122 => Ok(State::Running),
                123 => Ok(State::Cheated),
                124 => Ok(State::Finished),
                125 => Ok(State::Aborted),
                _ => bail!("Invalid constructor tag for State."),
            },
            _ => bail!("Invalid data type for State."),
        }
    }
}
