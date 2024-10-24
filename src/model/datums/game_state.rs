use anyhow::{anyhow, bail, Context};
use pallas::ledger::primitives::{
    alonzo,
    conway::{Constr, PlutusData},
};

pub struct PaymentCredential([u8; 28]);

pub enum State {
    LOBBY,
    RUNNING,
    CHEATED,
    FINISHED,
}
pub struct GameState {
    referee: PaymentCredential,
    players: Vec<PaymentCredential>,
    state: State,
    winner: Option<PaymentCredential>,
    cheater: Option<PaymentCredential>,
}

impl From<GameState> for PlutusData {
    fn from(value: GameState) -> Self {
        let players: PlutusData = PlutusData::Array(alonzo::MaybeIndefArray::Indef(
            value
                .players
                .into_iter()
                .map(|x| x.into())
                .collect::<Vec<_>>(),
        ));

        let winner: PlutusData = match value.winner {
            Some(winner) => PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Indef(vec![winner.into()]),
            }),
            None => PlutusData::Constr(Constr {
                tag: 122,
                any_constructor: None,
                fields: alonzo::MaybeIndefArray::Indef(vec![]),
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
                fields: alonzo::MaybeIndefArray::Indef(vec![]),
            }),
        };

        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: alonzo::MaybeIndefArray::Indef(vec![
                value.referee.into(),
                players,
                value.state.into(),
                winner,
                cheater,
            ]),
        })
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

                if constr.fields.len() != 5 {
                    bail!("Invalid number of fields for GameState.");
                }

                let referee: PaymentCredential =
                    constr.fields[0].clone().try_into().context("referee")?;

                let players: Vec<PaymentCredential> = match constr.fields[1].clone() {
                    PlutusData::Array(array) => {
                        let mut players = Vec::new();
                        for player in array.to_vec() {
                            players.push(player.try_into().context("players")?);
                        }

                        players
                    }
                    _ => bail!("Invalid data type for players"),
                };

                let state: State = constr.fields[2].clone().try_into().context("state")?;

                let winner: Option<PaymentCredential> = match constr.fields[3].clone() {
                    PlutusData::Constr(constr) => {
                        if constr.tag == 121 {
                            Some(
                                PaymentCredential::try_from(PlutusData::Constr(constr))
                                    .context("winner")?,
                            )
                        } else if constr.tag == 122 {
                            None
                        } else {
                            bail!("Invalid constructor tag for winner");
                        }
                    }
                    _ => bail!("Invalid data type for winner"),
                };

                let cheater: Option<PaymentCredential> = match constr.fields[4].clone() {
                    PlutusData::Constr(constr) => {
                        if constr.tag == 121 {
                            Some(
                                PaymentCredential::try_from(PlutusData::Constr(constr))
                                    .context("cheater")?,
                            )
                        } else if constr.tag == 122 {
                            None
                        } else {
                            bail!("Invalid constructor tag for cheater");
                        }
                    }
                    _ => bail!("Invalid data type for cheater"),
                };

                Ok(GameState {
                    referee,
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
            State::LOBBY => Constr {
                tag: 121,
                any_constructor: Some(0),
                fields: alonzo::MaybeIndefArray::Indef(vec![]),
            },
            State::RUNNING => Constr {
                tag: 121,
                any_constructor: Some(1),
                fields: alonzo::MaybeIndefArray::Indef(vec![]),
            },
            State::CHEATED => Constr {
                tag: 121,
                any_constructor: Some(2),
                fields: alonzo::MaybeIndefArray::Indef(vec![]),
            },
            State::FINISHED => Constr {
                tag: 121,
                any_constructor: Some(4),
                fields: alonzo::MaybeIndefArray::Indef(vec![]),
            },
        })
    }
}

impl TryFrom<PlutusData> for State {
    type Error = anyhow::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(constr) => match constr.tag {
                121 => Ok(State::LOBBY),
                122 => Ok(State::RUNNING),
                123 => Ok(State::CHEATED),
                124 => Ok(State::FINISHED),
                _ => bail!("Invalid constructor tag for State."),
            },
            _ => bail!("Invalid data type for State."),
        }
    }
}
