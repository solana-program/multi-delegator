use pinocchio::ProgramResult;

use crate::MultiDelegatorError;

pub fn validate_fixed_transfer(
    transfer_amount: u64,
    remaining: u64,
    expiry_ts: i64,
    current_ts: i64,
) -> ProgramResult {
    if transfer_amount == 0 {
        return Err(MultiDelegatorError::InvalidAmount.into());
    }
    if expiry_ts != 0 && current_ts > expiry_ts {
        return Err(MultiDelegatorError::DelegationExpired.into());
    }
    if transfer_amount > remaining {
        return Err(MultiDelegatorError::AmountExceedsLimit.into());
    }
    Ok(())
}

pub fn validate_recurring_transfer(
    transfer_amount: u64,
    amount_per_period: u64,
    period_length_s: u64,
    current_period_start_ts: &mut i64,
    amount_pulled_in_period: &mut u64,
    expiry_ts: i64,
    current_ts: i64,
) -> ProgramResult {
    if transfer_amount == 0 {
        return Err(MultiDelegatorError::InvalidAmount.into());
    }
    if expiry_ts != 0 && current_ts > expiry_ts {
        return Err(MultiDelegatorError::DelegationExpired.into());
    }

    let period_length =
        i64::try_from(period_length_s).map_err(|_| MultiDelegatorError::InvalidPeriodLength)?;
    if period_length == 0 {
        return Err(MultiDelegatorError::InvalidPeriodLength.into());
    }

    let time_since_start = current_ts.saturating_sub(*current_period_start_ts);

    if time_since_start >= period_length {
        let periods_passed = time_since_start / period_length;
        let increment = periods_passed
            .checked_mul(period_length)
            .ok_or(MultiDelegatorError::ArithmeticOverflow)?;
        *current_period_start_ts = current_period_start_ts
            .checked_add(increment)
            .ok_or(MultiDelegatorError::ArithmeticOverflow)?;
        *amount_pulled_in_period = 0;
    }

    let available = amount_per_period
        .checked_sub(*amount_pulled_in_period)
        .ok_or(MultiDelegatorError::ArithmeticUnderflow)?;
    if transfer_amount > available {
        return Err(MultiDelegatorError::AmountExceedsPeriodLimit.into());
    }

    *amount_pulled_in_period = amount_pulled_in_period
        .checked_add(transfer_amount)
        .ok_or(MultiDelegatorError::ArithmeticOverflow)?;

    Ok(())
}
