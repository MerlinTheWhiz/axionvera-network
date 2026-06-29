#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

use axionvera_accounting::{self as accounting, AccountingCategory, AccountingOperation, AccountingEntry, OperationResources};
use axionvera_events as events;
use axionvera_interfaces::{FeeConfig, FeeError, FeeReceipt, FeeTotals, FeeType, TREASURY_BPS_DENOMINATOR};
use axionvera_storage as storage;

#[contract]
pub struct FeeContract;

#[contractimpl]
impl FeeContract {
    pub fn version() -> u32 {
        1
    }
}

/// Validate that a fee configuration uses legal basis-point values.
pub fn validate_fee_config(config: &FeeConfig) -> Result<(), FeeError> {
    validate_rate(config.deposit_fee_bps)?;
    validate_rate(config.withdrawal_fee_bps)?;
    validate_rate(config.reward_fee_bps)?;
    Ok(())
}

/// Return the basis-point rate for the requested fee kind.
pub fn fee_rate(config: &FeeConfig, fee_type: FeeType) -> u32 {
    config.rate_for(fee_type)
}

/// Compute the gross fee, net amount, and treasury allocation for a fee event.
pub fn build_fee_receipt(
    fee_type: FeeType,
    actor: Address,
    treasury: Address,
    asset: Option<Address>,
    gross_amount: i128,
    fee_bps: u32,
    timestamp: u64,
) -> Result<FeeReceipt, FeeError> {
    let fee_amount = calculate_fee_amount(gross_amount, fee_bps)?;
    let net_amount = gross_amount
        .checked_sub(fee_amount)
        .ok_or(FeeError::MathOverflow)?;

    Ok(FeeReceipt {
        fee_type,
        actor,
        treasury,
        asset,
        gross_amount,
        fee_bps,
        fee_amount,
        net_amount,
        treasury_amount: fee_amount,
        timestamp,
    })
}

/// Calculate the fee amount for a gross amount and basis-point rate.
pub fn calculate_fee_amount(amount: i128, fee_bps: u32) -> Result<i128, FeeError> {
    validate_amount(amount)?;
    validate_rate(fee_bps)?;

    amount
        .checked_mul(fee_bps as i128)
        .ok_or(FeeError::MathOverflow)?
        .checked_div(TREASURY_BPS_DENOMINATOR as i128)
        .ok_or(FeeError::MathOverflow)
}

/// Record a fee receipt in storage, accounting, and the event stream.
pub fn record_fee_collection(
    e: &Env,
    receipt: &FeeReceipt,
    resources: OperationResources,
) -> Result<FeeTotals, FeeError> {
    if receipt.fee_amount == 0 {
        return Ok(storage::get_fee_totals(e, receipt.fee_type));
    }

    let totals = storage::record_fee_totals(
        e,
        receipt.fee_type,
        receipt.fee_amount,
        receipt.treasury_amount,
    )
    .map_err(|_| FeeError::MathOverflow)?;

    let operation = fee_operation(receipt.fee_type);
    accounting::record_operation(
        e,
        AccountingEntry {
            category: AccountingCategory::Treasury,
            operation,
            actor: Some(receipt.actor.clone()),
            asset: receipt.asset.clone(),
            amount_in: receipt.fee_amount,
            amount_out: 0,
            amount_processed: receipt.fee_amount,
            resources,
        },
    )
    .map_err(|_| FeeError::MathOverflow)?;

    events::emit_fee_collected(e, receipt);
    events::emit_fee_treasury_allocated(
        e,
        receipt.fee_type,
        receipt.treasury.clone(),
        receipt.treasury_amount,
        totals.treasury_amount,
    );

    Ok(totals)
}

/// Emit a fee configuration event after storing the new config.
pub fn emit_fee_configured(e: &Env, admin: Address, config: &FeeConfig) {
    events::emit_fee_configured(e, admin, config);
}

/// Build and record a fee receipt from a configuration record.
pub fn collect_fee(
    e: &Env,
    fee_type: FeeType,
    actor: Address,
    asset: Option<Address>,
    gross_amount: i128,
    config: &FeeConfig,
    resources: OperationResources,
) -> Result<FeeReceipt, FeeError> {
    let receipt = build_fee_receipt(
        fee_type,
        actor,
        config.treasury.clone(),
        asset,
        gross_amount,
        fee_rate(config, fee_type),
        e.ledger().timestamp(),
    )?;
    let _ = record_fee_collection(e, &receipt, resources)?;
    Ok(receipt)
}

fn validate_amount(amount: i128) -> Result<(), FeeError> {
    if amount <= 0 {
        return Err(FeeError::InvalidAmount);
    }
    Ok(())
}

fn validate_rate(rate_bps: u32) -> Result<(), FeeError> {
    if rate_bps > TREASURY_BPS_DENOMINATOR {
        return Err(FeeError::InvalidFeeRate);
    }
    Ok(())
}

fn fee_operation(fee_type: FeeType) -> AccountingOperation {
    match fee_type {
        FeeType::Deposit => AccountingOperation::TreasuryDepositFee,
        FeeType::Withdrawal => AccountingOperation::TreasuryWithdrawalFee,
        FeeType::Reward => AccountingOperation::TreasuryRewardFee,
    }
}

#[cfg(test)]
mod tests;
