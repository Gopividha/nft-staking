use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PlatForm {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub total_staked_nft: u64,
}
impl Sealed for PlatForm {}
impl IsInitialized for PlatForm {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for PlatForm {
    const LEN: usize = 41;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, PlatForm::LEN];
        let (is_initialized, owner, total_staked_nft) = array_refs![src, 1, 32, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(PlatForm {
            is_initialized,
            owner: Pubkey::new_from_array(*owner),
            total_staked_nft: u64::from_le_bytes(*total_staked_nft),
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, PlatForm::LEN];
        let (is_initialized_dst, owner_dst, total_staked_nft_dst) = mut_array_refs![dst, 1, 32, 8];
        let PlatForm {
            is_initialized,
            owner,
            total_staked_nft,
        } = self;
        is_initialized_dst[0] = *is_initialized as u8;
        owner_dst.copy_from_slice(owner.as_ref());
        *total_staked_nft_dst = total_staked_nft.to_le_bytes();
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct UserState {
    pub is_initialized: bool,
    pub user: Pubkey,
    pub total_staked_nft: u64,
    pub last_staked_time: u64,
}
impl Sealed for UserState {}
impl IsInitialized for UserState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for UserState {
    const LEN: usize = 49;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, UserState::LEN];
        let (is_initialized, user, total_staked_nft, last_staked_time) =
            array_refs![src, 1, 32, 8, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(UserState {
            is_initialized,
            user: Pubkey::new_from_array(*user),
            total_staked_nft: u64::from_le_bytes(*total_staked_nft),
            last_staked_time: u64::from_le_bytes(*last_staked_time),
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, UserState::LEN];
        let (is_initialized_dst, user_dst, total_staked_nft_dst, last_staked_time_dst) =
            mut_array_refs![dst, 1, 32, 8, 8];
        let UserState {
            is_initialized,
            user,
            total_staked_nft,
            last_staked_time,
        } = self;
        is_initialized_dst[0] = *is_initialized as u8;
        user_dst.copy_from_slice(user.as_ref());
        *total_staked_nft_dst = total_staked_nft.to_le_bytes();
        *last_staked_time_dst = last_staked_time.to_le_bytes();
    }
}
