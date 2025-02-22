use std::ops::Deref;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::stake::state::StakeState;

#[account]
#[derive(Debug)]
pub struct StakeWrapper {
    pub inner: StakeState,
}

// impl anchor_lang::AccountDeserialize for StakeWrapper {
//     fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
//         Self::try_deserialize_unchecked(buf)
//     }

//     fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
//         let result = Self {
//             inner: bincode::deserialize(buf).map_err(|_| ProgramError::InvalidAccountData)?,
//         };
//         *buf = &buf[std::mem::size_of::<StakeState>()..];
//         Ok(result)
//     }
// }

impl Deref for StakeWrapper {
    type Target = StakeState;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
