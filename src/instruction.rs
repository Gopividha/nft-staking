#![allow(clippy::too_many_arguments)]

use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::mem;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum NftInstruction {
    // Init
    InitializePlatform { amount: u64 },

    StakeNft,

    //Unstake
    UnStakeNft,

    Harvest,
}

impl NftInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidAccountData)?;

        Ok(match tag {
            0 => Self::InitializePlatform {
                amount: Self::unpack_amount(rest)?,
            },
            2 => Self::StakeNft,
            3 => Self::UnStakeNft,
            4 => Self::Harvest,

            _ => return Err(ProgramError::InvalidAccountData),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(mem::size_of::<Self>());
        match &*self {
            Self::InitializePlatform { amount } => {
                buf.push(0);
                buf.extend_from_slice(&amount.to_le_bytes());
            }

            _ => todo!(),
        }
        buf
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidAccountData)?;
        Ok(amount)
    }
}
