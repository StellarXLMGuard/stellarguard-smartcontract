#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, Env,
    IntoVal, String, Symbol, Vec,
};

mod test;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    NextInvoiceId,
    AllowedAsset(Address),
    Invoice(u64),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvoiceState {
    Created,
    Paid,
    Fulfilled,
    Refunded,
    Canceled,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invoice {
    pub id: u64,
    pub merchant: Address,
    pub recipient: Address,
    pub asset: Address,
    pub amount: i128,
    pub expires_at: u64,
    pub escrowed: bool,
    pub payer: Address,
    pub state: InvoiceState,
    pub created_at: u64,
    pub paid_at: u64,
    pub fulfilled_at: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    AssetNotAllowed = 4,
    InvoiceNotFound = 5,
    InvoiceNotCreated = 6,
    InvoiceExpired = 7,
    AlreadyPaid = 8,
    CannotCancelPaid = 9,
    PayerRequired = 10,
    NotEscrowed = 11,
    NotPaid = 12,
    AlreadyFulfilled = 13,
    AlreadyRefunded = 14,
    InvalidAmount = 15,
    ExpiryMustBeFuture = 16,
    EscrowRefundNotPermitted = 17,
    PayerRefundNotPermitted = 18,
    TransferError = 19,
}

const DAY_IN_LEDGERS: u32 = 17280;

#[contractevent]
pub struct InvoiceCreated {
    #[topic]
    pub id: u64,
    #[topic]
    pub merchant: Address,
    pub recipient: Address,
    pub asset: Address,
    pub amount: i128,
    pub expires_at: u64,
    pub escrowed: bool,
}

#[contractevent]
pub struct InvoicePaid {
    #[topic]
    pub id: u64,
    #[topic]
    pub payer: Address,
    pub amount: i128,
}

#[contractevent]
pub struct InvoiceFulfilled {
    #[topic]
    pub id: u64,
}

#[contractevent]
pub struct InvoiceRefunded {
    #[topic]
    pub id: u64,
}

#[contractevent]
pub struct InvoiceCanceled {
    #[topic]
    pub id: u64,
}

#[contractevent]
pub struct InvoiceExpired {
    #[topic]
    pub id: u64,
}

fn require_admin(env: &Env) {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized));
    admin.require_auth();
}

fn load_invoice(env: &Env, invoice_id: u64) -> Invoice {
    env.storage()
        .persistent()
        .get(&DataKey::Invoice(invoice_id))
        .unwrap_or_else(|| panic_with_error!(env, Error::InvoiceNotFound))
}

fn save_invoice(env: &Env, invoice: &Invoice) {
    env.storage()
        .persistent()
        .set(&DataKey::Invoice(invoice.id), invoice);
}

fn is_expired(expires_at: u64, env: &Env) -> bool {
    env.ledger().timestamp() >= expires_at
}

fn transfer_asset(env: &Env, from: &Address, to: &Address, asset: &Address, amount: &i128) {
    let token_client = token::Client::new(env, asset);
    token_client.transfer(from, to, amount);
}

fn transfer_asset_from_contract(env: &Env, to: &Address, asset: &Address, amount: &i128) {
    let token_client = token::Client::new(env, asset);
    let contract_address = env.current_contract_address();
    token_client.transfer(&contract_address, to, amount);
}

#[contract]
pub struct PayLinkContract;

#[contractimpl]
impl PayLinkContract {
    pub fn __constructor(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextInvoiceId, &1u64);
        env.storage()
            .instance()
            .extend_ttl(120 * DAY_IN_LEDGERS, 180 * DAY_IN_LEDGERS);
    }

    pub fn set_asset_enabled(env: Env, asset: Address, enabled: bool) {
        require_admin(&env);
        if enabled {
            env.storage()
                .persistent()
                .set(&DataKey::AllowedAsset(asset.clone()), &true);
        } else {
            env.storage().persistent().remove(&DataKey::AllowedAsset(asset));
        }
    }

    pub fn is_asset_enabled(env: Env, asset: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::AllowedAsset(asset))
            .unwrap_or(false)
    }

    pub fn create_invoice(
        env: Env,
        merchant: Address,
        recipient: Address,
        asset: Address,
        amount: i128,
        expires_at: u64,
        escrowed: bool,
    ) -> u64 {
        merchant.require_auth();

        if amount <= 0 {
            panic_with_error!(&env, Error::InvalidAmount);
        }

        if expires_at <= env.ledger().timestamp() {
            panic_with_error!(&env, Error::ExpiryMustBeFuture);
        }

        if !Self::is_asset_enabled(env.clone(), asset.clone()) {
            panic_with_error!(&env, Error::AssetNotAllowed);
        }

        let invoice_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextInvoiceId)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));

        let invoice = Invoice {
            id: invoice_id,
            merchant: merchant.clone(),
            recipient,
            asset: asset.clone(),
            amount,
            expires_at,
            escrowed,
            payer: merchant.clone(),
            state: InvoiceState::Created,
            created_at: env.ledger().timestamp(),
            paid_at: 0,
            fulfilled_at: 0,
        };

        save_invoice(&env, &invoice);

        env.storage()
            .instance()
            .set(&DataKey::NextInvoiceId, &(invoice_id + 1));

        env.storage().instance().extend_ttl(120 * DAY_IN_LEDGERS, 180 * DAY_IN_LEDGERS);

        InvoiceCreated {
            id: invoice_id,
            merchant,
            asset,
            amount,
            expires_at,
            escrowed,
            recipient: invoice.payer.clone(),
        }
        .publish(&env);

        invoice_id
    }

    pub fn pay_invoice(env: Env, payer: Address, invoice_id: u64) {
        payer.require_auth();

        let mut invoice = load_invoice(&env, invoice_id);

        if invoice.state != InvoiceState::Created {
            panic_with_error!(&env, Error::AlreadyPaid);
        }

        if is_expired(invoice.expires_at, &env) {
            panic_with_error!(&env, Error::InvoiceExpired);
        }

        let asset = invoice.asset.clone();
        let amount = invoice.amount;
        let is_escrowed = invoice.escrowed;

        let transfer_destination = if is_escrowed {
            env.current_contract_address()
        } else {
            invoice.recipient.clone()
        };

        transfer_asset(&env, &payer, &transfer_destination, &asset, &amount);

        invoice.state = InvoiceState::Paid;
        invoice.payer = payer.clone();
        invoice.paid_at = env.ledger().timestamp();
        save_invoice(&env, &invoice);

        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Invoice(invoice_id), 120 * DAY_IN_LEDGERS, 180 * DAY_IN_LEDGERS);

        InvoicePaid {
            id: invoice_id,
            payer,
            amount,
        }
        .publish(&env);
    }

    pub fn mark_fulfilled(env: Env, merchant: Address, invoice_id: u64) {
        merchant.require_auth();

        let mut invoice = load_invoice(&env, invoice_id);

        if invoice.merchant != merchant {
            panic_with_error!(&env, Error::NotAuthorized);
        }

        if invoice.state != InvoiceState::Paid {
            panic_with_error!(&env, Error::NotPaid);
        }

        if !invoice.escrowed {
            invoice.state = InvoiceState::Fulfilled;
            invoice.fulfilled_at = env.ledger().timestamp();
            save_invoice(&env, &invoice);

            InvoiceFulfilled { id: invoice_id }.publish(&env);
            return;
        }

        transfer_asset_from_contract(
            &env,
            &invoice.recipient,
            &invoice.asset,
            &invoice.amount,
        );

        invoice.state = InvoiceState::Fulfilled;
        invoice.fulfilled_at = env.ledger().timestamp();
        save_invoice(&env, &invoice);

        InvoiceFulfilled { id: invoice_id }.publish(&env);
    }

    pub fn refund_invoice(env: Env, caller: Address, invoice_id: u64) {
        caller.require_auth();

        let mut invoice = load_invoice(&env, invoice_id);

        if invoice.state != InvoiceState::Paid {
            panic_with_error!(&env, Error::NotPaid);
        }

        if !invoice.escrowed {
            panic_with_error!(&env, Error::NotEscrowed);
        }

        let is_merchant = invoice.merchant == caller;
        let is_payer = invoice.payer == caller;

        if !is_merchant && !is_payer {
            panic_with_error!(&env, Error::NotAuthorized);
        }

        if is_merchant && is_expired(invoice.expires_at, &env) {
            panic_with_error!(&env, Error::EscrowRefundNotPermitted);
        }

        if is_payer && !is_expired(invoice.expires_at, &env) {
            panic_with_error!(&env, Error::PayerRefundNotPermitted);
        }

        transfer_asset_from_contract(&env, &invoice.payer, &invoice.asset, &invoice.amount);

        invoice.state = InvoiceState::Refunded;
        save_invoice(&env, &invoice);

        InvoiceRefunded { id: invoice_id }.publish(&env);
    }

    pub fn cancel_invoice(env: Env, merchant: Address, invoice_id: u64) {
        merchant.require_auth();

        let mut invoice = load_invoice(&env, invoice_id);

        if invoice.merchant != merchant {
            panic_with_error!(&env, Error::NotAuthorized);
        }

        if invoice.state == InvoiceState::Paid
            || invoice.state == InvoiceState::Fulfilled
            || invoice.state == InvoiceState::Refunded
        {
            panic_with_error!(&env, Error::CannotCancelPaid);
        }

        invoice.state = InvoiceState::Canceled;
        save_invoice(&env, &invoice);

        InvoiceCanceled { id: invoice_id }.publish(&env);
    }

    pub fn expire_invoice(env: Env, invoice_id: u64) {
        let mut invoice = load_invoice(&env, invoice_id);

        if invoice.state != InvoiceState::Created {
            return;
        }

        if !is_expired(invoice.expires_at, &env) {
            return;
        }

        invoice.state = InvoiceState::Expired;
        save_invoice(&env, &invoice);

        InvoiceExpired { id: invoice_id }.publish(&env);
    }

    pub fn get_invoice(env: Env, invoice_id: u64) -> Invoice {
        load_invoice(&env, invoice_id)
    }

    pub fn get_next_invoice_id(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextInvoiceId)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized))
    }
}
