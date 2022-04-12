mod utils;

use crate::utils::{DummySignatureManger, SolanaClock, SolanaHashMap};
use anchor_lang::{prelude::borsh::maybestd::collections::HashMap, prelude::*};
use api3_common::{derive_beacon_id, ensure, process_beacon_update, Bytes32, DataPoint, Uint};

declare_id!("FRoo7m8Sf6ZAirGgnn3KopQymDtujWx818kcnRxzi23b");

// a bunch of error codes
const ERROR_INVALID_BEACON_ID_KEY: u64 = 1u64;
const ERROR_INVALID_SYSVAR_INSTRUCTIONS_KEY: u64 = 2u64;
const ERROR_SIGNATURES_NOT_VALIDATED: u64 = 3u64;
const ERROR_SIGNATURES_MORE_THAN_DATA: u64 = 4u64;
const ERROR_NOT_ENOUGH_ACCOUNT: u64 = 5u64;

fn map_error(e: api3_common::Error) -> anchor_lang::error::Error {
    anchor_lang::error::Error::from(ProgramError::Custom(e.into()))
}

#[program]
pub mod beacon_server {
    use super::*;

    /// Update a new beacon data point with signed data. The beacon id is used as
    /// the seed to generate pda for the Beacon data account.
    pub fn update_beacon_with_signed_data(
        ctx: Context<DataPointAccount>,
        datapoint_key: [u8; 32],
        template_id: [u8; 32],
        timestamp: [u8; 32],
        data: Vec<u8>,
    ) -> Result<()> {
        let airnode = ctx.accounts.user.key.to_bytes().to_vec();
        let beacon_id = derive_beacon_id(airnode, template_id);
        ensure!(
            beacon_id == datapoint_key,
            Error::from(ProgramError::from(ERROR_INVALID_BEACON_ID_KEY))
        )?;

        let timestamp = Uint::from(&timestamp);
        let mut s = SolanaHashMap::new(
            vec![(beacon_id, &mut ctx.accounts.datapoint)],
            HashMap::new(),
        );
        process_beacon_update(&mut s, beacon_id, timestamp, data).map_err(map_error)?;

        Ok(())
    }

    /// Update a new beacon data point with signed data.
    /// The beacon id is used as the seed to generate pda for the Beacon data account.
    pub fn update_dapi_with_beacons(
        ctx: Context<DataPointAccount>,
        datapoint_key: [u8; 32],
        beacon_ids: Vec<[u8; 32]>,
    ) -> Result<()> {
        assert!(
            !ctx.remaining_accounts.is_empty(),
            "must provide beacon accounts"
        );

        let beacon_id_tuples = ctx
            .remaining_accounts
            .iter()
            .map(|item| -> Result<(Pubkey, Account<WrappedDataPoint>)> {
                Account::try_from_unchecked(item).map(|i| (*item.key, i))
            })
            .collect::<Result<Vec<(Pubkey, Account<WrappedDataPoint>)>>>()?;

        utils::check_beacon_ids(&beacon_ids, &beacon_id_tuples)?;
        utils::check_dapi_id(&datapoint_key, &beacon_ids)?;

        // Step 2. Prepare the accounts
        let mut idx = 0;
        let write = vec![(datapoint_key, &mut ctx.accounts.datapoint)];
        let mut read = HashMap::new();
        for (_, wrapped) in beacon_id_tuples {
            let datapoint =
                DataPoint::from(wrapped.raw_datapoint.clone()).expect("cannot parse datapoint");
            read.insert(beacon_ids[idx], datapoint);
            idx += 1;
        }

        ensure!(
            idx == beacon_ids.len(),
            Error::from(ProgramError::from(ERROR_NOT_ENOUGH_ACCOUNT))
        )?;

        let mut s = SolanaHashMap::new(write, read);
        api3_common::update_dapi_with_beacons(&mut s, &beacon_ids).map_err(map_error)?;
        Ok(())
    }

    /// Updates a dAPI using data signed by the respective Airnodes
    /// without requiring a request or subscription. The beacons for which the
    /// signature is omitted will be read from the storage.
    pub fn update_dapi_with_signed_data<'b>(
        ctx: Context<'_, '_, '_, 'b, DataPointAccount<'b>>,
        datapoint_key: [u8; 32],
        airnodes: Vec<Vec<u8>>,
        beacon_ids: Vec<[u8; 32]>,
        template_ids: Vec<[u8; 32]>,
        timestamps: Vec<[u8; 32]>,
        data: Vec<Vec<u8>>,
    ) -> Result<()> {
        // Step 1. Check signature
        let account_iter = &mut ctx.remaining_accounts.iter();
        let instruction_acc = account_iter
            .next()
            .ok_or_else(|| Error::from(ProgramError::from(ERROR_NOT_ENOUGH_ACCOUNT)))?;
        let sig_count = ensure_batch_signed(instruction_acc, &data)?;

        let mut idx = 0usize;
        let beacon_id_tuples = account_iter
            .clone()
            .map(|item| -> (Pubkey, Bytes32) {
                let r = (*item.key, template_ids[idx]);
                idx += 1;
                r
            })
            .collect::<Vec<(Pubkey, Bytes32)>>();
        utils::check_beacon_ids_with_templates(&beacon_ids, &beacon_id_tuples)?;
        utils::check_dapi_id(&datapoint_key, &beacon_ids)?;

        let sig_checker = DummySignatureManger::new(sig_count);

        // Step 2. Prepare the accounts
        let mut idx = 0;
        let write = vec![(datapoint_key, &mut ctx.accounts.datapoint)];
        let mut read = HashMap::new();
        for account in account_iter {
            let beacon_offset = idx + sig_count;
            let wrapped: Account<WrappedDataPoint> = Account::try_from(account)?;
            let datapoint =
                DataPoint::from(wrapped.raw_datapoint.clone()).expect("cannot parse datapoint");
            read.insert(beacon_ids[beacon_offset], datapoint);
            idx += 1;
        }
        idx += sig_count;

        ensure!(
            idx == beacon_ids.len(),
            Error::from(ProgramError::from(ERROR_NOT_ENOUGH_ACCOUNT))
        )?;

        let mut s = SolanaHashMap::new(write, read);
        let clock = SolanaClock::new(Clock::get().unwrap().unix_timestamp as u32);

        api3_common::update_dapi_with_signed_data(
            &mut s,
            &sig_checker,
            &clock,
            airnodes,
            template_ids,
            timestamps,
            data,
            (0..idx).into_iter().map(|_| vec![]).collect(),
        )
        .map_err(map_error)?;
        Ok(())
    }

    /// Sets the data point ID the name points to
    /// While a data point ID refers to a specific Beacon or dAPI, names
    /// provide a more abstract interface for convenience. This means a name
    /// that was pointing at a Beacon can be pointed to a dAPI, then another
    /// dAPI, etc.
    pub fn set_name(
        _ctx: Context<DataPointIdAccount>,
        datapoint_id_key: [u8; 32],
        _name: [u8; 32],
        _data_point_id: [u8; 32],
    ) -> Result<()> {
        msg!(
            "delete this in actual implementation: {:?}",
            datapoint_id_key
        );
        Ok(())
    }

    pub fn read_with_data_point_id(
        _ctx: Context<DataPointAccount>,
        datapoint_key: [u8; 32],
    ) -> Result<()> {
        msg!("delete this in actual implementation: {:?}", datapoint_key);
        Ok(())
    }

    /// Reads the data point with name
    /// The read data point may belong to a Beacon or dAPI. The reader
    /// must be whitelisted for the hash of the data point name.
    pub fn read_with_name(
        _ctx: Context<DataPointAccount>,
        datapoint_key: [u8; 32],
        _name: [u8; 32],
    ) -> Result<(u128, u32)> {
        msg!("delete this in actual implementation: {:?}", datapoint_key);
        Ok((0, 0))
    }

    /// Returns if a reader can read the data point
    pub fn reader_can_read_data_point(
        _ctx: Context<DataPointAccount>,
        datapoint_key: [u8; 32],
        _name: [u8; 32],
        _reader: [u8; 32],
    ) -> Result<bool> {
        msg!("delete this in actual implementation: {:?}", datapoint_key);
        Ok(false)
    }
}

#[derive(Accounts)]
#[instruction(datapoint_id_key: [u8; 32])]
pub struct DataPointIdAccount<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32,
        seeds = [b"hashed-name", datapoint_id_key.as_ref()],
        bump
    )]
    pub datapoint_id: Account<'info, WrappedDataPointId>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(datapoint_key: [u8; 32])]
pub struct DataPointAccount<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 41,
        seeds = [b"datapoint", datapoint_key.as_ref()],
        bump
    )]
    pub datapoint: Account<'info, WrappedDataPoint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct WrappedDataPoint {
    raw_datapoint: Vec<u8>,
    bump: u8,
}

#[account]
pub struct WrappedDataPointId {
    datapoint_id: [u8; 32],
    bump: u8,
}

fn ensure_batch_signed(instruction_acc: &AccountInfo, data: &[Vec<u8>]) -> Result<usize> {
    ensure!(
        *instruction_acc.key == anchor_lang::solana_program::sysvar::instructions::id(),
        Error::from(ProgramError::from(ERROR_INVALID_SYSVAR_INSTRUCTIONS_KEY))
    )?;

    let r = anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked(
        0,
        instruction_acc,
    )?;
    ensure!(
        r.program_id == anchor_lang::solana_program::ed25519_program::id(),
        Error::from(ProgramError::from(ERROR_SIGNATURES_NOT_VALIDATED))
    )?;

    let sig_count = r.data[0] as usize;
    ensure!(
        sig_count <= data.len(),
        Error::from(ProgramError::from(ERROR_SIGNATURES_MORE_THAN_DATA))
    )?;

    Ok(sig_count)
}

// /// Create the beacon accounts. We only create up till `sig_count`, as only `sig_count` number
// /// of signatures are provided
// fn check_or_create_account<'a>(
//     account_iter: &mut Iter<AccountInfo<'a>>,
//     beacon_ids: &Vec<Bytes32>,
//     payer_info: &AccountInfo<'a>,
//     program_id: &Pubkey,
//     system_info: &AccountInfo<'a>,
//     sig_count: usize
// ) -> Result<()> {
//     let mut idx = 0;
//     for a in account_iter {
//         if a.data_is_empty() {
//             create_and_serialize_account_signed(
//                 payer_info,
//                 &a,
//                 &[DATAPOINT_SEED.as_bytes(), &beacon_ids[idx]],
//                 program_id,
//                 system_info,
//                 &Rent::get()?,
//                 DATAPOINT_ACCOUNT_SIZE,
//             )?;
//         }
//         idx += 1;
//         if idx >= sig_count {
//             break;
//         }
//     }
//     Ok(())
// }
//
// pub fn create_and_serialize_account_signed<'a>(
//     payer_info: &AccountInfo<'a>,
//     account_info: &AccountInfo<'a>,
//     account_address_seeds: &[&[u8]],
//     program_id: &Pubkey,
//     system_info: &AccountInfo<'a>,
//     rent: &Rent,
//     account_size: usize,
// ) -> Result<()> {
//     // Get PDA and assert it's the same as the requested account address
//     let (account_address, bump_seed) =
//         Pubkey::find_program_address(account_address_seeds, program_id);
//
//     if account_address != *account_info.key {
//         msg!(
//             "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
//             account_info.key,
//             account_address
//         );
//         return Err(Error::from(ProgramError::from(ERROR_INVALID_ACCOUNT_SEED)));
//     }
//
//     let create_account_instruction =
//         anchor_lang::solana_program::system_instruction::create_account(
//             payer_info.key,
//             account_info.key,
//             rent.minimum_balance(account_size),
//             account_size as u64,
//             program_id,
//         );
//
//     let mut signers_seeds = account_address_seeds.to_vec();
//     let bump = &[bump_seed];
//     signers_seeds.push(bump);
//
//     anchor_lang::solana_program::program::invoke_signed(
//         &create_account_instruction,
//         &[
//             payer_info.clone(),
//             account_info.clone(),
//             system_info.clone(),
//         ],
//         &[&signers_seeds[..]],
//     )?;
//
//     Ok(())
// }
