#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token::TokenClient, Address, BytesN, Env, Vec,
};

use axionvera_events::{
    self, TreasuryDistributionEvent, TreasuryFeeRecordedEvent, TreasuryInitializedEvent,
    TreasuryStrategyConfiguredEvent, ACT_TREASURY_DISTRIBUTE, ACT_TREASURY_FEE, ACT_TREASURY_INIT,
    ACT_TREASURY_STRATEGY, EVENT_VERSION, PROTOCOL,
};
use axionvera_interfaces::{
    AllocationRule, AllocationStrategy, AllocationTransfer, TreasuryAllocator,
    TreasuryDistributionReceipt, TreasuryError, TreasuryFeeRecord, TREASURY_BPS_DENOMINATOR,
};

const MAX_ALLOCATION_RULES: u32 = 16;
const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Initialized,
    Admin,
    Asset,
    Strategy(BytesN<32>),
    Distribution(BytesN<32>),
    Fee(BytesN<32>),
    TotalFeesRecorded,
    TotalDistributed,
    RecipientDistributed(Address),
}

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {
    pub fn version() -> u32 {
        1
    }
}

#[contractimpl]
impl TreasuryAllocator for TreasuryContract {
    fn initialize(e: Env, admin: Address, asset: Address) -> Result<(), TreasuryError> {
        if is_initialized(&e) {
            return Err(TreasuryError::AlreadyInitialized);
        }

        admin.require_auth();
        e.storage().instance().set(&DataKey::Initialized, &true);
        e.storage().instance().set(&DataKey::Admin, &admin);
        e.storage().instance().set(&DataKey::Asset, &asset);
        e.storage()
            .instance()
            .set(&DataKey::TotalFeesRecorded, &0_i128);
        e.storage()
            .instance()
            .set(&DataKey::TotalDistributed, &0_i128);
        bump_instance_ttl(&e);
        emit_initialized(&e, admin, asset);
        Ok(())
    }

    fn configure_strategy(
        e: Env,
        admin: Address,
        strategy: AllocationStrategy,
    ) -> Result<(), TreasuryError> {
        require_admin(&e, &admin)?;
        validate_strategy(&e, &strategy)?;

        e.storage()
            .persistent()
            .set(&DataKey::Strategy(strategy.id.clone()), &strategy);
        bump_instance_ttl(&e);
        emit_strategy_configured(&e, &strategy);
        Ok(())
    }

    fn record_fee(
        e: Env,
        caller: Address,
        fee_id: BytesN<32>,
        payer: Address,
        amount: i128,
    ) -> Result<TreasuryFeeRecord, TreasuryError> {
        require_admin(&e, &caller)?;
        if amount <= 0 {
            return Err(TreasuryError::InvalidAmount);
        }
        if e.storage().persistent().has(&DataKey::Fee(fee_id.clone())) {
            return Err(TreasuryError::DuplicateFee);
        }

        payer.require_auth();
        let asset = get_asset(&e)?;
        let treasury_address = e.current_contract_address();
        let token = TokenClient::new(&e, &asset);
        token.transfer(&payer, &treasury_address, &amount);
        let treasury_balance = token.balance(&treasury_address);
        record_total_fees(&e, amount)?;

        let record = TreasuryFeeRecord {
            fee_id: fee_id.clone(),
            payer,
            asset,
            amount,
            treasury_balance,
            timestamp: axionvera_events::ledger_timestamp(&e),
        };
        e.storage().persistent().set(&DataKey::Fee(fee_id), &record);
        bump_instance_ttl(&e);
        emit_fee_recorded(&e, &record);
        Ok(record)
    }

    fn distribute(
        e: Env,
        admin: Address,
        distribution_id: BytesN<32>,
        strategy_id: BytesN<32>,
        amount: i128,
    ) -> Result<TreasuryDistributionReceipt, TreasuryError> {
        require_admin(&e, &admin)?;
        if amount <= 0 {
            return Err(TreasuryError::InvalidAmount);
        }
        if e.storage()
            .persistent()
            .has(&DataKey::Distribution(distribution_id.clone()))
        {
            return Err(TreasuryError::DuplicateDistribution);
        }

        let strategy = get_strategy(&e, &strategy_id)?;
        let asset = get_asset(&e)?;
        let treasury_address = e.current_contract_address();
        let token = TokenClient::new(&e, &asset);
        if token.balance(&treasury_address) < amount {
            return Err(TreasuryError::InsufficientBalance);
        }

        let transfers = calculate_transfers(&e, &strategy, amount)?;
        for transfer in transfers.iter() {
            token.transfer(&treasury_address, &transfer.recipient, &transfer.amount);
            record_recipient_distribution(&e, &transfer.recipient, transfer.amount)?;
        }
        record_total_distribution(&e, amount)?;

        let receipt = TreasuryDistributionReceipt {
            distribution_id: distribution_id.clone(),
            strategy_id,
            asset: asset.clone(),
            total_amount: amount,
            transfers,
            timestamp: axionvera_events::ledger_timestamp(&e),
        };
        e.storage()
            .persistent()
            .set(&DataKey::Distribution(distribution_id), &receipt);
        bump_instance_ttl(&e);
        emit_distribution(&e, &receipt);
        Ok(receipt)
    }

    fn strategy(e: Env, strategy_id: BytesN<32>) -> Option<AllocationStrategy> {
        e.storage()
            .persistent()
            .get(&DataKey::Strategy(strategy_id))
    }

    fn fee_record(e: Env, fee_id: BytesN<32>) -> Option<TreasuryFeeRecord> {
        e.storage().persistent().get(&DataKey::Fee(fee_id))
    }

    fn distribution_receipt(
        e: Env,
        distribution_id: BytesN<32>,
    ) -> Option<TreasuryDistributionReceipt> {
        e.storage()
            .persistent()
            .get(&DataKey::Distribution(distribution_id))
    }

    fn recipient_distributed(e: Env, recipient: Address) -> i128 {
        e.storage()
            .persistent()
            .get(&DataKey::RecipientDistributed(recipient))
            .unwrap_or(0)
    }

    fn total_fees_recorded(e: Env) -> i128 {
        e.storage()
            .instance()
            .get(&DataKey::TotalFeesRecorded)
            .unwrap_or(0)
    }

    fn total_distributed(e: Env) -> i128 {
        e.storage()
            .instance()
            .get(&DataKey::TotalDistributed)
            .unwrap_or(0)
    }

    fn asset_balance(e: Env) -> i128 {
        let Ok(asset) = get_asset(&e) else {
            return 0;
        };
        TokenClient::new(&e, &asset).balance(&e.current_contract_address())
    }
}

fn is_initialized(e: &Env) -> bool {
    e.storage()
        .instance()
        .get::<_, bool>(&DataKey::Initialized)
        .unwrap_or(false)
}

fn require_admin(e: &Env, admin: &Address) -> Result<(), TreasuryError> {
    if !is_initialized(e) {
        return Err(TreasuryError::NotInitialized);
    }

    let stored_admin = e
        .storage()
        .instance()
        .get::<_, Address>(&DataKey::Admin)
        .ok_or(TreasuryError::NotInitialized)?;
    if &stored_admin != admin {
        return Err(TreasuryError::Unauthorized);
    }
    admin.require_auth();
    Ok(())
}

fn get_asset(e: &Env) -> Result<Address, TreasuryError> {
    e.storage()
        .instance()
        .get(&DataKey::Asset)
        .ok_or(TreasuryError::NotInitialized)
}

fn get_strategy(e: &Env, strategy_id: &BytesN<32>) -> Result<AllocationStrategy, TreasuryError> {
    e.storage()
        .persistent()
        .get(&DataKey::Strategy(strategy_id.clone()))
        .ok_or(TreasuryError::StrategyNotFound)
}

fn validate_strategy(e: &Env, strategy: &AllocationStrategy) -> Result<(), TreasuryError> {
    let rule_count = strategy.rules.len();
    if rule_count == 0 {
        return Err(TreasuryError::EmptyStrategy);
    }
    if rule_count > MAX_ALLOCATION_RULES {
        return Err(TreasuryError::TooManyRules);
    }

    let mut total_bps = 0_u32;
    let mut recipients = Vec::new(e);
    for rule in strategy.rules.iter() {
        validate_rule(&rule)?;
        if contains_address(&recipients, &rule.recipient) {
            return Err(TreasuryError::DuplicateRecipient);
        }
        recipients.push_back(rule.recipient);
        total_bps = total_bps
            .checked_add(rule.share_bps)
            .ok_or(TreasuryError::InvalidShareTotal)?;
    }

    if total_bps != TREASURY_BPS_DENOMINATOR {
        return Err(TreasuryError::InvalidShareTotal);
    }
    Ok(())
}

fn validate_rule(rule: &AllocationRule) -> Result<(), TreasuryError> {
    if rule.share_bps == 0 || rule.share_bps > TREASURY_BPS_DENOMINATOR {
        return Err(TreasuryError::InvalidShare);
    }
    Ok(())
}

fn calculate_transfers(
    e: &Env,
    strategy: &AllocationStrategy,
    amount: i128,
) -> Result<Vec<AllocationTransfer>, TreasuryError> {
    let mut transfers = Vec::new(e);
    let mut allocated = 0_i128;
    let last_index = (strategy.rules.len() - 1) as usize;

    for (index, rule) in strategy.rules.iter().enumerate() {
        let transfer_amount = if index == last_index {
            amount
                .checked_sub(allocated)
                .ok_or(TreasuryError::InvalidAmount)?
        } else {
            amount
                .checked_mul(rule.share_bps as i128)
                .ok_or(TreasuryError::InvalidAmount)?
                .checked_div(TREASURY_BPS_DENOMINATOR as i128)
                .ok_or(TreasuryError::InvalidAmount)?
        };

        allocated = allocated
            .checked_add(transfer_amount)
            .ok_or(TreasuryError::InvalidAmount)?;
        transfers.push_back(AllocationTransfer {
            recipient: rule.recipient,
            amount: transfer_amount,
        });
    }

    Ok(transfers)
}

fn record_recipient_distribution(
    e: &Env,
    recipient: &Address,
    amount: i128,
) -> Result<(), TreasuryError> {
    let key = DataKey::RecipientDistributed(recipient.clone());
    let current = e.storage().persistent().get(&key).unwrap_or(0_i128);
    let updated = current
        .checked_add(amount)
        .ok_or(TreasuryError::InvalidAmount)?;
    e.storage().persistent().set(&key, &updated);
    Ok(())
}

fn record_total_fees(e: &Env, amount: i128) -> Result<(), TreasuryError> {
    let current = e
        .storage()
        .instance()
        .get(&DataKey::TotalFeesRecorded)
        .unwrap_or(0_i128);
    let updated = current
        .checked_add(amount)
        .ok_or(TreasuryError::InvalidAmount)?;
    e.storage()
        .instance()
        .set(&DataKey::TotalFeesRecorded, &updated);
    Ok(())
}

fn record_total_distribution(e: &Env, amount: i128) -> Result<(), TreasuryError> {
    let current = e
        .storage()
        .instance()
        .get(&DataKey::TotalDistributed)
        .unwrap_or(0_i128);
    let updated = current
        .checked_add(amount)
        .ok_or(TreasuryError::InvalidAmount)?;
    e.storage()
        .instance()
        .set(&DataKey::TotalDistributed, &updated);
    Ok(())
}

fn contains_address(values: &Vec<Address>, needle: &Address) -> bool {
    for value in values.iter() {
        if &value == needle {
            return true;
        }
    }
    false
}

fn bump_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

fn emit_initialized(e: &Env, admin: Address, asset: Address) {
    e.events().publish(
        (PROTOCOL, ACT_TREASURY_INIT),
        TreasuryInitializedEvent {
            event_version: EVENT_VERSION,
            admin,
            asset,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

fn emit_strategy_configured(e: &Env, strategy: &AllocationStrategy) {
    e.events().publish(
        (PROTOCOL, ACT_TREASURY_STRATEGY),
        TreasuryStrategyConfiguredEvent {
            event_version: EVENT_VERSION,
            strategy_id: strategy.id.clone(),
            rule_count: strategy.rules.len(),
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

fn emit_fee_recorded(e: &Env, record: &TreasuryFeeRecord) {
    e.events().publish(
        (PROTOCOL, ACT_TREASURY_FEE),
        TreasuryFeeRecordedEvent {
            event_version: EVENT_VERSION,
            fee_id: record.fee_id.clone(),
            payer: record.payer.clone(),
            asset: record.asset.clone(),
            amount: record.amount,
            treasury_balance: record.treasury_balance,
            timestamp: record.timestamp,
        },
    );
}

fn emit_distribution(e: &Env, receipt: &TreasuryDistributionReceipt) {
    e.events().publish(
        (PROTOCOL, ACT_TREASURY_DISTRIBUTE),
        TreasuryDistributionEvent {
            event_version: EVENT_VERSION,
            distribution_id: receipt.distribution_id.clone(),
            strategy_id: receipt.strategy_id.clone(),
            asset: receipt.asset.clone(),
            total_amount: receipt.total_amount,
            timestamp: receipt.timestamp,
        },
    );
}

#[cfg(test)]
mod test;
