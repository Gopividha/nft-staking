use num_traits::CheckedDiv;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    system_instruction::create_account,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
};

use crate::{
    error::FarmError,
    instruction::NftInstruction,
    state::{PlatForm, UserState},
};
use spl_associated_token_account;
use spl_token::{instruction::transfer, state::Account as TokenAccount};
use std::cell::RefCell;
use std::str::FromStr;
pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = NftInstruction::unpack(instruction_data)?;
        match instruction {
            NftInstruction::InitializePlatform { amount } => {
                msg!("Instruction:INIT PLATFORM");
                return Self::process_init_platform(accounts, program_id, amount);
            }
            NftInstruction::StakeNft {} => {
                msg!("Instruction:STAKE NFT!!!!!");
                return Self::process_stake_nft(accounts, program_id);
            }
            NftInstruction::UnStakeNft {} => {
                msg!("Instruction: UNSTAKE NFT");
                return Self::process_unstake_nft(accounts, program_id);
            }
            NftInstruction::Harvest {} => {
                msg!("Instruction: CLAIM REWARD");
                return Self::process_harvest_reward(accounts, program_id);
            }
        }
    }

    pub fn process_init_platform(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        alloc_point_new: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let platform_state_account = next_account_info(account_info_iter)?;
        let owner_account = next_account_info(account_info_iter)?;
        let admin_reward_token_account = next_account_info(account_info_iter)?;
        let pda_reward_token_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let system_program_id = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        invoke(
            &create_account(
                owner_account.key,
                platform_state_account.key,
                Rent::default().minimum_balance(PlatForm::LEN),
                PlatForm::LEN as u64,
                program_id,
            ),
            &[
                owner_account.clone(),
                platform_state_account.clone(),
                system_program_id.clone(),
            ],
        )?;
        msg!("Platfom_state_account {}", platform_state_account.key);

        //pda to store staked tokens
        let pda_prefix = "rappid-paltform";

        let pda_seed = &[pda_prefix.as_bytes(), platform_state_account.key.as_ref()];

        let (pda, nonce) = Pubkey::find_program_address(pda_seed, program_id);

        msg!("pda {}", pda);

        let mut platform_data =
            PlatForm::unpack_unchecked(&platform_state_account.try_borrow_data()?)?;
        msg!("after unpack");

        if platform_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        platform_data.is_initialized = true;
        platform_data.owner = *owner_account.key;
        platform_data.total_staked_nft = 0;

        let transfer_token = transfer(
            token_program.key,
            admin_reward_token_account.key,
            pda_reward_token_account.key,
            owner_account.key,
            &[],
            alloc_point_new.clone(),
        )?;
        msg!("Calling the token program to transfer LP tokens pdatoken account...");
        invoke(
            &transfer_token,
            &[
                admin_reward_token_account.clone(),
                pda_reward_token_account.clone(),
                owner_account.clone(),
                token_program.clone(),
            ],
        )?;

        PlatForm::pack(
            platform_data,
            &mut platform_state_account.try_borrow_mut_data()?,
        )?;

        Ok(())
    }

    pub fn process_user_init(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        msg!("entry ********************************");
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        let user_state_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;

        let pda_seed = &[(user.key).as_ref(), (mint.key).as_ref()];
        let (pda, nonce) = Pubkey::find_program_address(pda_seed, program_id);
        msg!("pda {}", pda);

        if pda != *user_state_account.key {
            msg!("pda wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        invoke_signed(
            &create_account(
                user.key,
                user_state_account.key,
                Rent::default().minimum_balance(UserState::LEN),
                UserState::LEN as u64,
                program_id,
            ),
            &[
                user.clone(),
                user_state_account.clone(),
                system_program.clone(),
            ],
            &[&[&(user.key).as_ref(), &(mint.key).as_ref()[..], &[nonce]]],
        )?;

        let mut user_data = UserState::unpack_unchecked(&user_state_account.try_borrow_data()?)?;

        if user_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        msg!("after unpacks ******************");

        user_data.is_initialized = true;
        user_data.user = *user.key;
        user_data.total_staked_nft = 0;
        user_data.last_staked_time = 0;

        UserState::pack(user_data, &mut user_state_account.try_borrow_mut_data()?)?;

        Ok(())
    }

    pub fn process_stake_nft(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        msg!("entered ******************");
        let user = next_account_info(account_info_iter)?;
        let user_state_account = next_account_info(account_info_iter)?;
        let platform_state = next_account_info(account_info_iter)?;

        let token_account = next_account_info(account_info_iter)?;
        let mint_key = next_account_info(account_info_iter)?;

        let pda_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        let system_program = next_account_info(account_info_iter)?;

        let user_pda_seed = &[(user.key).as_ref(), (mint_key.key).as_ref()];
        let (user_state, nonce) = Pubkey::find_program_address(user_pda_seed, program_id);
        msg!("pda {}", user_state_account.key);

        if user_state != *user_state_account.key {
            msg!("user_state_acc wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        if user_state_account.owner != program_id {
            let user_init_accounts = &[
                user.clone(),
                user_state_account.clone(),
                system_program.clone(),
                mint_key.clone(),
            ];

            Self::process_user_init(user_init_accounts, program_id);
        };

        let mut user_data = UserState::unpack_unchecked(&user_state_account.try_borrow_data()?)?;

        //pda to store staked tokens
        let pda_prefix = "rappid-paltform";
        let pda_seed = &[pda_prefix.as_bytes(), (platform_state.key).as_ref()];

        let (pda, nonce) = Pubkey::find_program_address(pda_seed, program_id);
        msg!("pda {}", pda_account.key);

        if pda != *pda_account.key {
            msg!("wrong pda");
            return Err(ProgramError::InvalidAccountData);
        }
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            user.key,
            &[],
        )?;

        invoke(
            &owner_change_ix,
            &[token_account.clone(), user.clone(), token_program.clone()],
        )?;

        //set up clock
        let system_clock = Clock::get()?;

        let mut platform_state_info =
            PlatForm::unpack_unchecked(&platform_state.try_borrow_data()?)?;

        msg!("unpacked!!!!!!!!!!!!!!");

        user_data.last_staked_time = system_clock.unix_timestamp.clone() as u64;
        msg!("current time{}",user_data.last_staked_time);

        platform_state_info.total_staked_nft = platform_state_info
            .total_staked_nft
            .checked_add(1)
            .ok_or(ProgramError::AccountDataTooSmall)?;

        UserState::pack(user_data, &mut user_state_account.try_borrow_mut_data()?)?;
        PlatForm::pack(
            platform_state_info,
            &mut platform_state.try_borrow_mut_data()?,
        )?;
        msg!(" packed!!!!1111");

        Ok(())
    }

    pub fn process_unstake_nft(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        msg!("entered ******************");
        let user = next_account_info(account_info_iter)?;
        let user_state_account = next_account_info(account_info_iter)?;
        let platform_state = next_account_info(account_info_iter)?;

        let pda_token_account = next_account_info(account_info_iter)?;
        let mint_key = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        let token_program = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let user_reward_account = next_account_info(account_info_iter)?;
        let pda_reward_token_account = next_account_info(account_info_iter)?;


        let user_pda_seed = &[(user.key).as_ref(), (mint_key.key).as_ref()];

        
        let (user_state, nonce) = Pubkey::find_program_address(user_pda_seed, program_id);
        msg!("pda {}", user_state);

        if user_state != *user_state_account.key {
            msg!("user_state_acc wrong");
            return Err(ProgramError::InvalidAccountData);
        }

        //pda to store staked tokens
        let pda_prefix = "rappid-paltform";
        let pda_seed = &[pda_prefix.as_bytes(), (platform_state.key).as_ref()];

        let (pda, nonce) = Pubkey::find_program_address(pda_seed, program_id);
        msg!("user {}", user_state_account.key);

        msg!("pda {}", pda);

        if pda != *pda_account.key {
            msg!("error with farm pda");
            return Err(ProgramError::InvalidAccountData);
        }

        let transfer_nft = spl_token::instruction::set_authority(
            token_program.key,
            pda_token_account.key,
            Some(user.key),
            spl_token::instruction::AuthorityType::AccountOwner,
            &pda,
            &[],
        )?;
        invoke_signed(
            &transfer_nft,
            &[
                pda_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[
                pda_prefix.as_bytes(),
                &(platform_state.key).as_ref()[..],
                &[nonce],
            ]],
        )?;

        //set up clock
        let system_clock = Clock::get()?;
        msg!("111");

        let user_data = UserState::unpack_unchecked(&user_state_account.try_borrow_data()?)?;

        msg!("222");

        let mut platform_state_info =
            PlatForm::unpack_unchecked(&platform_state.try_borrow_data()?)?;

        platform_state_info.total_staked_nft = platform_state_info
            .total_staked_nft
            .checked_sub(1)
            .ok_or(ProgramError::AccountDataTooSmall)?;

        msg!(
            "total value staked {}",
            platform_state_info.total_staked_nft
        );
        msg!{"lst updated {}",user_data.last_staked_time};

        let total_staked_duration =
        system_clock.unix_timestamp.clone() as u64 - user_data.last_staked_time;
        msg!("total_duration{}",total_staked_duration);



        if total_staked_duration > 86400 {
            let harvest_accounts = &[
                user.clone(),
                user_state_account.clone(),
                platform_state.clone(),
                user_reward_account.clone(),
                pda_reward_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
                system_program.clone(),
                mint_key.clone(),
            ];

            Self::process_harvest_reward(harvest_accounts, program_id);
        }

        UserState::pack(user_data, &mut user_state_account.try_borrow_mut_data()?)?;
        PlatForm::pack(
            platform_state_info,
            &mut platform_state.try_borrow_mut_data()?,
        )?;

        Ok(())
    }

    pub fn process_harvest_reward(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        msg!("claim reward!!!!!!");
        let user = next_account_info(account_info_iter)?;
        let user_state_account = next_account_info(account_info_iter)?;
        let platform_state = next_account_info(account_info_iter)?;

        let user_reward_account = next_account_info(account_info_iter)?;
        let pda_reward_token_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        let token_program = next_account_info(account_info_iter)?;

        //pda with rewarder auth
        msg!("1111111");


        let pda_prefix = "rappid-paltform";
        let pda_seed = &[pda_prefix.as_bytes(), (platform_state.key).as_ref()];

        let (pda, nonce) = Pubkey::find_program_address(pda_seed, program_id);

        let system_clock = Clock::get()?;

        let mut user_data = UserState::unpack_unchecked(&user_state_account.try_borrow_data()?)?;

        let total_staked_duration =
        system_clock.unix_timestamp.clone() as u64 - user_data.last_staked_time;
        let reward_per_day:u64 = 10;
        let rewra_per_sec = ((reward_per_day as f32) / 86400.00)*100000.00;

        

        msg!("rewra_per_sec {}",rewra_per_sec);
        msg!("total staked duration {}",total_staked_duration);


        let total_reward = ((total_staked_duration * rewra_per_sec as u64) / 1000);
        msg!("total reward {}",total_reward);

        if total_staked_duration > 86400 {
            let transfer_token = transfer(
                token_program.key,
                pda_reward_token_account.key,
                user_reward_account.key,
                &pda,
                &[],
                total_reward.clone(),
            )?;
            msg!("Calling the token program to transfer LP tokens pdatoken account...");
            invoke_signed(
                &transfer_token,
                &[
                    pda_reward_token_account.clone(),
                    user_reward_account.clone(),
                    pda_account.clone(),
                    token_program.clone(),
                ],
                &[&[
                    pda_prefix.as_bytes(),
                    &(platform_state.key).as_ref()[..],
                    &[nonce],
                ]],
            )?;
    
        }

       
        UserState::pack(user_data, &mut user_state_account.try_borrow_mut_data()?)?;
        msg!("user packed!!!!");

        Ok(())
    }
}
