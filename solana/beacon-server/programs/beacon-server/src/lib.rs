mod access;
mod utils;
mod whitelist;

use crate::utils::{DatapointHashMap, DummySignatureManger, SolanaClock};
use anchor_lang::{prelude::borsh::maybestd::collections::HashMap, prelude::*};
use api3_common::{derive_beacon_id, ensure, process_beacon_update, DataPoint, Uint};

declare_id!("FRoo7m8Sf6ZAirGgnn3KopQymDtujWx818kcnRxzi23b");

// a bunch of error codes
const ERROR_INVALID_BEACON_ID_KEY: u64 = 1u64;
const ERROR_INVALID_SYSVAR_INSTRUCTIONS_KEY: u64 = 2u64;
const ERROR_SIGNATURES_NOT_VALIDATED: u64 = 3u64;
const ERROR_SIGNATURES_MORE_THAN_DATA: u64 = 4u64;
const ERROR_NOT_ENOUGH_ACCOUNT: u64 = 5u64;
const ERROR_INVALID_NAME_HASH: u64 = 6u64;
const ERROR_DATA_LENGTH_NOT_MATCH: u64 = 7u64;
const ERROR_INVALID_DERIVED_DAPI_ID_KEY: u64 = 8u64;
const ERROR_INVALID_SYSTEM_PROGRAM_ID: u64 = 9u64;

fn map_error(e: api3_common::Error) -> anchor_lang::error::Error {
    anchor_lang::error::Error::from(ProgramError::Custom(e.into()))
}

#[program]
pub mod beacon_server {
    use super::*;
    use crate::access::DummyAccessControl;
    use crate::utils::NameHashHashMap;

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
        utils::check_sys_program(ctx.accounts.system_program.key)?;

        let timestamp = Uint::from(&timestamp);
        let mut s = DatapointHashMap::new(
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
        utils::check_sys_program(ctx.accounts.system_program.key)?;

        let beacon_id_tuples = ctx
            .remaining_accounts
            .iter()
            .map(|item| -> Result<(Pubkey, Account<WrappedDataPoint>)> {
                Account::try_from_unchecked(item).map(|i| (*item.key, i))
            })
            .collect::<Result<Vec<(Pubkey, Account<WrappedDataPoint>)>>>()?;

        let keys = beacon_id_tuples.iter().map(|a| a.0).collect::<Vec<_>>();
        utils::check_beacon_ids(&beacon_ids, &keys, ctx.program_id)?;
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

        let mut s = DatapointHashMap::new(write, read);
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
        let sig_checker = DummySignatureManger::new(sig_count);

        utils::check_sys_program(ctx.accounts.system_program.key)?;

        // Step 2. Check the beacon id accounts are correct
        let mut idx = 0usize;
        let keys = account_iter
            .clone()
            .map(|item| -> Pubkey {
                idx += 1;
                *item.key
            })
            .collect::<Vec<Pubkey>>();

        utils::check_beacon_ids(&beacon_ids, &keys, ctx.program_id)?;
        utils::check_dapi_id(&datapoint_key, &beacon_ids)?;

        // Step 3. Extract and prepare the data for beacon ids from storage
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

        // Step 4. Execute update_dapi_with_signed_data process
        let mut s = DatapointHashMap::new(write, read);
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
        ctx: Context<DataPointIdAccount>,
        name_hash: [u8; 32],
        name: [u8; 32],
        datapoint_id: [u8; 32],
    ) -> Result<()> {
        let access = DummyAccessControl::default();
        let msg_sender = ctx.accounts.user.key.to_bytes();

        utils::check_sys_program(ctx.accounts.system_program.key)?;
        utils::check_name_hash(&name, &name_hash)?;

        let mut storage = NameHashHashMap::new(vec![(name_hash, &mut ctx.accounts.hash)]);
        api3_common::set_name(
            name,
            datapoint_id,
            &msg_sender,
            &access,
            &mut storage
        ).map_err(map_error)
    }

    // pub fn read_with_data_point_id(
    //     ctx: Context<DataPointAccount>,
    //     datapoint_key: [u8; 32],
    // ) -> Result<(Int, u32)> {
    //     let whitelist = DummyWhitelist::default();
    //     let access = DummyAccessControl::default();
    //     let msg_sender = ctx.accounts.user.key.to_bytes();
    //
    //     let write = vec![(datapoint_key, &mut ctx.accounts.datapoint)];
    //     let mut s = DatapointHashMap::new(write, HashMap::new());
    //     let r = api3_common::read_with_data_point_id(
    //         &datapoint_key,
    //         &msg_sender,
    //         &mut s,
    //         &access,
    //         &whitelist,
    //     ).map_err(map_error)?;
    //     Ok(r)
    // }
    //
    // /// Reads the data point with name
    // /// The read data point may belong to a Beacon or dAPI. The reader
    // /// must be whitelisted for the hash of the data point name.
    // pub fn read_with_name(
    //     ctx: Context<DataWithDataPointIdAccount>,
    //     datapoint_key: [u8; 32],
    //     name: [u8; 32],
    //     name_hash: [u8; 32],
    // ) -> Result<(Int, u32)> {
    //     ensure!(
    //         keccak_packed(&[Token::FixedBytes(name.to_vec())]) == name_hash,
    //         Error::from(ProgramError::from(ERROR_INVALID_NAME_HASH))
    //     )?;
    //     let whitelist = DummyWhitelist::default();
    //     let access = DummyAccessControl::default();
    //     let msg_sender = ctx.accounts.user.key.to_bytes();
    //
    //     let datapoint_s = DatapointHashMap::new(
    //         vec![(datapoint_key, &mut ctx.accounts.datapoint)],
    //         HashMap::new(),
    //     );
    //     let name_hash_s = NameHashHashMap::new(vec![(datapoint_key, &mut ctx.accounts.hash)]);
    //
    //     let r = api3_common::read_with_name(
    //         name,
    //         &msg_sender,
    //         &datapoint_s,
    //         &name_hash_s,
    //         &access,
    //         &whitelist,
    //     ).map_err(map_error)?;
    //     Ok(r)
    // }
    //
    // /// Returns if a reader can read the data point
    // pub fn reader_can_read_data_point(
    //     ctx: Context<DataPointAccount>,
    //     datapoint_key: [u8; 32],
    // ) -> Result<bool> {
    //     let msg_sender = ctx.accounts.user.key.to_bytes();
    //     let whitelist = DummyWhitelist::default();
    //     let access = DummyAccessControl::default();
    //     Ok(api3_common::reader_can_read_data_point(
    //         &datapoint_key,
    //         &msg_sender,
    //         &access,
    //         &whitelist,
    //     ))
    // }
}

#[derive(Accounts)]
#[instruction(name_hash: [u8; 32])]
pub struct DataPointIdAccount<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 33,
        seeds = [b"hashed-name", name_hash.as_ref()],
        bump
    )]
    pub hash: Account<'info, WrappedDataPointId>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(datapoint_key: [u8; 32], name_hash: [u8; 32])]
pub struct DataWithDataPointIdAccount<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 33,
        seeds = [b"hashed-name", name_hash.as_ref()],
        bump
    )]
    pub hash: Account<'info, WrappedDataPointId>,
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
    pub raw_datapoint: Vec<u8>,
    pub bump: u8,
}

#[account]
pub struct WrappedDataPointId {
    pub datapoint_id: [u8; 32],
    pub bump: u8,
}

#[account]
pub struct WrappedUserWithRole {
    pub has_role: bool,
    pub bump: u8,
}

#[account]
pub struct WrappedRoleDetail {
    pub admin_role: [u8; 32],
    pub bump: u8,
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
