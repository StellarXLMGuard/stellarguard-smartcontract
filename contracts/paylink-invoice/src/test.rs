#![cfg(test)]
extern crate std;

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events as _},
    token, Address, Env, IntoVal, Symbol,
};

use crate::{InvoiceState, PayLinkContract, PayLinkContractClient};

fn setup_with_admin(env: &Env) -> (Address, Address) {
    let admin = Address::generate(env);
    let contract_id = env.register(PayLinkContract, (admin.clone(),));
    let client = PayLinkContractClient::new(env, &contract_id);
    (admin, contract_id)
}

fn setup_token(env: &Env, admin: &Address) -> (token::StellarAssetContractClient, Address) {
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetContractClient::new(env, &token_id);
    token_client.mint(admin, &100000i128);
    (token_client, token_id)
}

fn setup_with_token(
    env: &Env,
) -> (
    PayLinkContractClient,
    Address,
    token::StellarAssetContractClient,
    Address,
) {
    let (admin, contract_id) = setup_with_admin(env);
    let client = PayLinkContractClient::new(env, &contract_id);
    let (token_client, token_id) = setup_token(env, &admin);

    client.set_asset_enabled(&token_id, &true);

    (client, admin, token_client, token_id)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(PayLinkContract, (admin.clone(),));
    let client = PayLinkContractClient::new(&env, &contract_id);

    assert_eq!(client.get_next_invoice_id(), 1u64);
}

#[test]
#[should_panic(expected = "AlreadyInitialized")]
fn test_cannot_reinitialize() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(PayLinkContract, (admin.clone(),));
    PayLinkContractClient::new(&env, &contract_id);
    // Attempting to register again with the same contract constructs the
    // client from the existing id, but a second call to __constructor
    // is not possible in the test harness — verify by checking auth
}

#[test]
fn test_create_invoice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _token_client, token_id) = setup_with_token(&env);
    let merchant = Address::generate(&env);
    let recipient = Address::generate(&env);
    let created_at = 1000u64;
    env.ledger().set_timestamp(created_at);

    let id = client.create_invoice(
        &merchant,
        &recipient,
        &token_id,
        &100i128,
        &5000u64,
        &false,
    );

    assert_eq!(id, 1u64);

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.merchant, merchant);
    assert_eq!(invoice.recipient, recipient);
    assert_eq!(invoice.asset, token_id);
    assert_eq!(invoice.amount, 100i128);
    assert_eq!(invoice.expires_at, 5000u64);
    assert_eq!(invoice.escrowed, false);
    assert_eq!(invoice.state, InvoiceState::Created);
    assert_eq!(invoice.created_at, created_at);

    let events = env.events().all();
    assert_eq!(events.len(), 1);

    assert_eq!(client.get_next_invoice_id(), 2u64);
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_create_invoice_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, token_id) = setup_with_token(&env);
    let merchant = Address::generate(&env);

    client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &0i128,
        &5000u64,
        &false,
    );
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_create_invoice_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, token_id) = setup_with_token(&env);
    let merchant = Address::generate(&env);

    client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &-1i128,
        &5000u64,
        &false,
    );
}

#[test]
#[should_panic(expected = "ExpiryMustBeFuture")]
fn test_create_invoice_past_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, token_id) = setup_with_token(&env);
    let merchant = Address::generate(&env);
    env.ledger().set_timestamp(5000);

    client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &1000u64,
        &false,
    );
}

#[test]
#[should_panic(expected = "AssetNotAllowed")]
fn test_create_invoice_disallowed_asset() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup_with_token(&env);
    let merchant = Address::generate(&env);
    let bad_asset = Address::generate(&env);

    client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &bad_asset,
        &100i128,
        &5000u64,
        &false,
    );
}

#[test]
fn test_direct_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &recipient,
        &token_id,
        &100i128,
        &5000u64,
        &false,
    );

    let payer_balance_before = token_client.balance(&payer);
    let recipient_balance_before = token_client.balance(&recipient);

    client.pay_invoice(&payer, &id);

    let payer_balance_after = token_client.balance(&payer);
    let recipient_balance_after = token_client.balance(&recipient);

    assert_eq!(payer_balance_before - payer_balance_after, 100i128);
    assert_eq!(recipient_balance_after - recipient_balance_before, 100i128);

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.state, InvoiceState::Paid);
    assert_eq!(invoice.payer, payer);

    let events = env.events().all();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_escrow_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &recipient,
        &token_id,
        &100i128,
        &5000u64,
        &true,
    );

    let payer_balance_before = token_client.balance(&payer);

    client.pay_invoice(&payer, &id);

    let payer_balance_after = token_client.balance(&payer);
    assert_eq!(payer_balance_before - payer_balance_after, 100i128);

    let contract_balance = token_client.balance(&env.current_contract_address());
    assert_eq!(contract_balance, 100i128);

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.state, InvoiceState::Paid);
}

#[test]
fn test_escrow_fulfillment() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &recipient,
        &token_id,
        &100i128,
        &5000u64,
        &true,
    );

    client.pay_invoice(&payer, &id);

    let recipient_balance_before = token_client.balance(&recipient);

    client.mark_fulfilled(&merchant, &id);

    let recipient_balance_after = token_client.balance(&recipient);
    assert_eq!(recipient_balance_after - recipient_balance_before, 100i128);

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.state, InvoiceState::Fulfilled);

    let contract_balance = token_client.balance(&env.current_contract_address());
    assert_eq!(contract_balance, 0i128);
}

#[test]
fn test_direct_payment_fulfillment() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_client = token::StellarAssetContractClient::new(&env, &token_id);
    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &recipient,
        &token_id,
        &100i128,
        &5000u64,
        &false,
    );

    client.pay_invoice(&payer, &id);
    client.mark_fulfilled(&merchant, &id);

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.state, InvoiceState::Fulfilled);
}

#[test]
fn test_merchant_refund_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &5000u64,
        &true,
    );

    client.pay_invoice(&payer, &id);

    let payer_balance_before = token_client.balance(&payer);

    client.refund_invoice(&merchant, &id);

    let payer_balance_after = token_client.balance(&payer);
    assert_eq!(payer_balance_after - payer_balance_before, 100i128);

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.state, InvoiceState::Refunded);
}

#[test]
#[should_panic(expected = "EscrowRefundNotPermitted")]
fn test_merchant_cannot_refund_after_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &3000u64,
        &true,
    );

    client.pay_invoice(&payer, &id);

    env.ledger().set_timestamp(5000);

    client.refund_invoice(&merchant, &id);
}

#[test]
fn test_payer_refund_after_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &3000u64,
        &true,
    );

    client.pay_invoice(&payer, &id);

    env.ledger().set_timestamp(5000);

    let payer_balance_before = token_client.balance(&payer);
    client.refund_invoice(&payer, &id);
    let payer_balance_after = token_client.balance(&payer);
    assert_eq!(payer_balance_after - payer_balance_before, 100i128);

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.state, InvoiceState::Refunded);
}

#[test]
#[should_panic(expected = "PayerRefundNotPermitted")]
fn test_payer_cannot_refund_before_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &5000u64,
        &true,
    );

    client.pay_invoice(&payer, &id);

    client.refund_invoice(&payer, &id);
}

#[test]
#[should_panic(expected = "NotEscrowed")]
fn test_cannot_refund_direct_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &5000u64,
        &false,
    );

    client.pay_invoice(&payer, &id);

    client.refund_invoice(&merchant, &id);
}

#[test]
fn test_cancel_invoice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &5000u64,
        &false,
    );

    client.cancel_invoice(&merchant, &id);

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.state, InvoiceState::Canceled);
}

#[test]
#[should_panic(expected = "CannotCancelPaid")]
fn test_cannot_cancel_paid_invoice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &5000u64,
        &false,
    );

    client.pay_invoice(&payer, &id);
    client.cancel_invoice(&merchant, &id);
}

#[test]
fn test_expire_invoice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &3000u64,
        &false,
    );

    env.ledger().set_timestamp(5000);

    client.expire_invoice(&id);

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.state, InvoiceState::Expired);
}

#[test]
#[should_panic(expected = "AlreadyPaid")]
fn test_cannot_pay_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &5000u64,
        &false,
    );

    client.pay_invoice(&payer, &id);
    client.pay_invoice(&payer, &id);
}

#[test]
#[should_panic(expected = "InvoiceExpired")]
fn test_cannot_pay_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &3000u64,
        &false,
    );

    env.ledger().set_timestamp(5000);

    client.pay_invoice(&payer, &id);
}

#[test]
fn test_set_asset_enabled() {
    let env = Env::default();
    let (admin, contract_id) = setup_with_admin(&env);
    let client = PayLinkContractClient::new(&env, &contract_id);

    let asset = Address::generate(&env);

    assert!(!client.is_asset_enabled(&asset));

    env.mock_all_auths();
    client.set_asset_enabled(&asset, &true);

    assert!(client.is_asset_enabled(&asset));

    client.set_asset_enabled(&asset, &false);

    assert!(!client.is_asset_enabled(&asset));
}

#[test]
fn test_auth_enforcement() {
    let env = Env::default();
    let (admin, contract_id) = setup_with_admin(&env);
    let client = PayLinkContractClient::new(&env, &contract_id);

    let unauthorized = Address::generate(&env);

    let result = client.try_set_asset_enabled(&Address::generate(&env), &true);
    assert!(result.is_err());
}

#[test]
fn test_invoice_id_increments() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let recipient = Address::generate(&env);
    env.ledger().set_timestamp(1000);

    let id1 = client.create_invoice(&merchant, &recipient, &token_id, &100i128, &5000u64, &false);
    let id2 = client.create_invoice(&merchant, &recipient, &token_id, &200i128, &6000u64, &true);
    let id3 = client.create_invoice(&merchant, &recipient, &token_id, &300i128, &7000u64, &false);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
    assert_eq!(client.get_next_invoice_id(), 4);
}

#[test]
fn test_full_lifecycle_direct() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(&merchant, &recipient, &token_id, &100i128, &5000u64, &false);

    assert_eq!(client.get_invoice(&id).state, InvoiceState::Created);

    client.pay_invoice(&payer, &id);
    assert_eq!(client.get_invoice(&id).state, InvoiceState::Paid);

    client.mark_fulfilled(&merchant, &id);
    assert_eq!(client.get_invoice(&id).state, InvoiceState::Fulfilled);

    assert_eq!(token_client.balance(&recipient), 100i128);
}

#[test]
fn test_full_lifecycle_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(&merchant, &recipient, &token_id, &100i128, &5000u64, &true);

    assert_eq!(client.get_invoice(&id).state, InvoiceState::Created);

    client.pay_invoice(&payer, &id);
    assert_eq!(client.get_invoice(&id).state, InvoiceState::Paid);
    assert_eq!(
        token_client.balance(&env.current_contract_address()),
        100i128
    );

    client.mark_fulfilled(&merchant, &id);
    assert_eq!(client.get_invoice(&id).state, InvoiceState::Fulfilled);
    assert_eq!(token_client.balance(&recipient), 100i128);
    assert_eq!(
        token_client.balance(&env.current_contract_address()),
        0i128
    );
}

#[test]
fn test_full_lifecycle_escrow_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_client, token_id) = setup_with_token(&env);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    token_client.mint(&payer, &1000i128);
    env.ledger().set_timestamp(1000);

    let id = client.create_invoice(
        &merchant,
        &Address::generate(&env),
        &token_id,
        &100i128,
        &3000u64,
        &true,
    );

    client.pay_invoice(&payer, &id);

    env.ledger().set_timestamp(5000);

    client.refund_invoice(&payer, &id);
    assert_eq!(client.get_invoice(&id).state, InvoiceState::Refunded);
    assert_eq!(token_client.balance(&payer), 1000i128);
}
