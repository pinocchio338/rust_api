mod utils;

use anchor_lang::prelude::*;

declare_id!("FRoo7m8Sf6ZAirGgnn3KopQymDtujWx818kcnRxzi23b");

#[program]
pub mod beacon_server {
    use super::*;

    /// Update a new beacon data point with signed data. The beacon id is used as
    /// the seed to generate pda for the Beacon data account.
    pub fn update_beacon_with_signed_data(
        ctx: Context<DataPointAccount>,
        datapoint_key: [u8; 32],
        _template_id: [u8; 32],
        _timestamp: [u8; 32],
        data: Vec<u8>,
        _signature: Vec<u8>,
    ) -> Result<()> {
        // TOOD: perform signature check

        msg!("delete this in actual implementation: {:?}", datapoint_key);

        utils::update_beacon_data(&mut ctx.accounts.datapoint, data)?;

        Ok(())
    }

    /// Update a new beacon data point with signed data.
    /// The beacon id is used as the seed to generate pda for the Beacon data account.
    pub fn update_dapi_with_beacons(
        ctx: Context<DataPointAccount>,
        datapoint_key: [u8; 32],
        beacon_ids: Vec<[u8; 32]>,
    ) -> Result<()> {
        assert!(!ctx.remaining_accounts.is_empty(), "must provide beacon accounts");

        let beacon_id_tuples = ctx
            .remaining_accounts
            .iter()
            .map(|item| -> Result<(Pubkey, Account<WrappedDataPoint>)> {
                Account::try_from_unchecked(item).map(|i| (*item.key, i))
            })
            .collect::<Result<Vec<(Pubkey, Account<WrappedDataPoint>)>>>()?;

        utils::check_beacon_ids(&beacon_ids, &beacon_id_tuples)?;
        utils::check_dapi_id(&datapoint_key, &beacon_ids)?;

        let account = &mut ctx.accounts.datapoint;
        account.raw_datapoint = vec![1];

        Ok(())
    }

    /// Updates a dAPI using data signed by the respective Airnodes
    /// without requiring a request or subscription. The beacons for which the
    /// signature is omitted will be read from the storage.
    pub fn update_dapi_with_signed_data(
        _ctx: Context<DataPointAccount>,
        datapoint_key: [u8; 32],
        _beacon_ids: Vec<[u8; 32]>,
        _template_ids: Vec<[u8; 32]>,
        _timestamps: Vec<[u8; 32]>,
        _datas: Vec<Vec<u8>>,
        _signatures: Vec<Vec<u8>>,
    ) -> Result<()> {
        // TOOD: perform signature check

        msg!("delete this in actual implementation: {:?}", datapoint_key);

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
        msg!("delete this in actual implementation: {:?}", datapoint_id_key);
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
        space = 8 + 37,
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
