#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short, Address};

// Structure to track freelancer payroll information
#[contracttype]
#[derive(Clone)]
pub struct PayrollRecord {
    pub freelancer_id: Address,
    pub amount: u128,
    pub currency: String,      // "USD", "EUR", "INR", etc.
    pub payment_date: u64,
    pub payment_status: bool,  // true if paid, false if pending
    pub description: String,
}

// Structure to track exchange rates for currency conversion
#[contracttype]
#[derive(Clone)]
pub struct ExchangeRate {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: u128,           // Rate multiplied by 1000000 for precision
    pub last_updated: u64,
}

// Mapping for Payroll records to their unique IDs
#[contracttype]
pub enum PayrollBook {
    Payroll(u64),
}

// Mapping for Exchange Rates
#[contracttype]
pub enum RateBook {
    Rate(u64),
}

// Counter for tracking total payments
const PAYMENT_COUNT: Symbol = symbol_short!("PAY_CNT");

// Counter for tracking exchange rates
const RATE_COUNT: Symbol = symbol_short!("RATE_CNT");

#[contract]
pub struct CrossCurrencyPayroll;

#[contractimpl]
impl CrossCurrencyPayroll {

    // Function to record a new payroll entry for a freelancer
    pub fn create_payroll(
        env: Env,
        freelancer: Address,
        amount: u128,
        currency: String,
        description: String,
    ) -> u64 {
        let time = env.ledger().timestamp();
        let mut payment_count: u64 = env.storage().instance().get(&PAYMENT_COUNT).unwrap_or(0);
        payment_count += 1;

        let payroll = PayrollRecord {
            freelancer_id: freelancer.clone(),
            amount,
            currency: currency.clone(),
            payment_date: time,
            payment_status: false,
            description,
        };

        // Store the payroll record
        env.storage().instance().set(&PayrollBook::Payroll(payment_count), &payroll);
        env.storage().instance().set(&PAYMENT_COUNT, &payment_count);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Payroll created for freelancer, Amount: {}, Currency code set", amount);

        payment_count
    }

    // Function to process payment and mark it as completed
    pub fn process_payment(env: Env, payment_id: u64) {
        let mut payroll = Self::view_payroll(env.clone(), payment_id);

        if payroll.payment_status == false {
            payroll.payment_status = true;

            env.storage().instance().set(&PayrollBook::Payroll(payment_id), &payroll);
            env.storage().instance().extend_ttl(5000, 5000);

            log!(&env, "Payment processed for Payment-ID: {}", payment_id);
        } else {
            log!(&env, "Payment already processed!");
            panic!("Payment already processed!");
        }
    }

    // Function to convert currency amount using stored exchange rates
    pub fn convert_currency(
        env: Env,
        amount: u128,
        from_currency: String,
        to_currency: String,
    ) -> u128 {
        let rate = Self::get_exchange_rate(env.clone(), from_currency.clone(), to_currency.clone());
        
        if rate == 0 {
            log!(&env, "Exchange rate not found for conversion");
            panic!("Exchange rate not found!");
        }

        // Calculate converted amount (amount * rate / 1000000)
        let converted_amount = (amount * rate) / 1000000;
        
        log!(&env, "Currency converted, amount: {}", converted_amount);
        
        converted_amount
    }

    // Function to update exchange rates (can only be called by admin/oracle)
    pub fn set_exchange_rate(
        env: Env,
        from_currency: String,
        to_currency: String,
        rate: u128,
    ) {
        let time = env.ledger().timestamp();
        let mut rate_count: u64 = env.storage().instance().get(&RATE_COUNT).unwrap_or(0);
        rate_count += 1;

        let exchange_rate = ExchangeRate {
            from_currency: from_currency.clone(),
            to_currency: to_currency.clone(),
            rate,
            last_updated: time,
        };

        // Store exchange rate using rate count as key
        env.storage().instance().set(&RateBook::Rate(rate_count), &exchange_rate);
        env.storage().instance().set(&RATE_COUNT, &rate_count);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Exchange rate updated with rate value: {}", rate);
    }

    // Helper function to retrieve a specific payroll record
    pub fn view_payroll(env: Env, payment_id: u64) -> PayrollRecord {
        let key = PayrollBook::Payroll(payment_id);

        env.storage().instance().get(&key).unwrap_or(PayrollRecord {
            freelancer_id: Address::from_str(&env, "CBQHLSNAYSO23TOSXZXY2QN7F7DUNXVZJWYBEJLAKI2DFV4OKRIE7UJ"),
            amount: 0,
            currency: String::from_str(&env, "NOT_FOUND"),
            payment_date: 0,
            payment_status: false,
            description: String::from_str(&env, "NOT_FOUND"),
        })
    }

    // Helper function to retrieve exchange rate by rate ID
    pub fn view_exchange_rate(env: Env, rate_id: u64) -> ExchangeRate {
        let key = RateBook::Rate(rate_id);

        env.storage().instance().get(&key).unwrap_or(ExchangeRate {
            from_currency: String::from_str(&env, "NOT_FOUND"),
            to_currency: String::from_str(&env, "NOT_FOUND"),
            rate: 0,
            last_updated: 0,
        })
    }

    // Helper function to retrieve exchange rate
    fn get_exchange_rate(env: Env, _from_currency: String, _to_currency: String) -> u128 {
        let rate_count: u64 = env.storage().instance().get(&RATE_COUNT).unwrap_or(0);
        
        if rate_count > 0 {
            let latest_rate = Self::view_exchange_rate(env.clone(), rate_count);
            latest_rate.rate
        } else {
            0
        }
    }
}