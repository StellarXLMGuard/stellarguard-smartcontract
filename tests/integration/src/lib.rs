#[cfg(test)]
mod workflows {
    use soroban_sdk::testutils::{Address as _, Events as _, Ledger as _};
    use soroban_sdk::token::StellarAssetClient;
    use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, String, Val, Vec};
    use stellar_guard_access_control::{AccessControlContract, AccessControlContractClient, Role};
    use stellar_guard_governance::{
        GovernanceContract, GovernanceContractClient, ProposalAction, ProposalStatus,
    };
    use stellar_guard_token_vault::{TokenVaultContract, TokenVaultContractClient};
    use stellar_guard_treasury::{TreasuryContract, TreasuryContractClient};

    struct Contracts {
        env: Env,
        treasury_id: Address,
        treasury: TreasuryContractClient<'static>,
        governance_id: Address,
        governance: GovernanceContractClient<'static>,
        vault_id: Address,
        vault: TokenVaultContractClient<'static>,
        access: AccessControlContractClient<'static>,
    }

    fn setup() -> Contracts {
        let env = Env::default();
        env.mock_all_auths();

        let treasury_id = env.register_contract(None, TreasuryContract);
        let governance_id = env.register_contract(None, GovernanceContract);
        let vault_id = env.register_contract(None, TokenVaultContract);
        let access_id = env.register_contract(None, AccessControlContract);

        Contracts {
            treasury: TreasuryContractClient::new(&env, &treasury_id),
            governance: GovernanceContractClient::new(&env, &governance_id),
            vault: TokenVaultContractClient::new(&env, &vault_id),
            access: AccessControlContractClient::new(&env, &access_id),
            env,
            treasury_id,
            governance_id,
            vault_id,
        }
    }

    fn text(env: &Env, value: &str) -> String {
        String::from_str(env, value)
    }

    fn assert_last_event(env: &Env, contract_id: &Address, namespace: Val, action: Val) {
        let events = env.events().all();
        let event = events.get(events.len() - 1).unwrap();
        assert_eq!(event.0, *contract_id);
        assert_eq!(event.1, vec![env, namespace, action]);
    }

    #[test]
    fn full_treasury_workflow_init_deposit_propose_approve_execute() {
        let contracts = setup();
        let env = &contracts.env;

        let admin = Address::generate(env);
        let signer1 = Address::generate(env);
        let signer2 = Address::generate(env);
        let token_admin = Address::generate(env);
        let asset = env
            .register_stellar_asset_contract_v2(token_admin)
            .address();
        StellarAssetClient::new(env, &asset).mint(&signer1, &5_000_000);
        let recipient = Address::generate(env);
        let signers = Vec::from_array(env, [signer1.clone(), signer2.clone()]);

        contracts.treasury.initialize(&admin, &2, &signers, &asset);
        contracts.treasury.deposit(&signer1, &5_000_000);
        let tx_id = contracts.treasury.propose_withdrawal(
            &signer1,
            &recipient,
            &2_000_000,
            &text(env, "ops"),
        );
        contracts.treasury.approve(&signer2, &tx_id);
        contracts.treasury.execute(&signer1, &tx_id);

        let tx = contracts.treasury.get_transaction(&tx_id);
        assert!(tx.executed);
        assert_eq!(contracts.treasury.get_balance(), 3_000_000);
        assert_last_event(
            env,
            &contracts.treasury_id,
            symbol_short!("treasury").into_val(env),
            symbol_short!("execute").into_val(env),
        );
    }

    #[test]
    fn full_governance_workflow_init_propose_vote_finalize_execute() {
        let contracts = setup();
        let env = &contracts.env;

        let owner = Address::generate(env);
        let member1 = Address::generate(env);
        let member2 = Address::generate(env);
        let new_member = Address::generate(env);
        let members = Vec::from_array(env, [member1.clone(), member2.clone()]);

        contracts.access.initialize(&owner);
        contracts
            .access
            .assign_role(&owner, &member1, &Role::Member);
        contracts
            .access
            .assign_role(&owner, &member2, &Role::Member);
        assert!(contracts.access.is_member_or_above(&member1));

        contracts.governance.initialize(&owner, &members, &50, &10);
        let proposal_id = contracts.governance.create_proposal(
            &member1,
            &text(env, "Add member"),
            &text(env, "Add another governance member"),
            &ProposalAction::AddMember,
            &0,
            &new_member,
        );
        contracts.governance.vote(&member1, &proposal_id, &true);
        contracts.governance.vote(&member2, &proposal_id, &true);

        env.ledger().set_sequence_number(100);
        let status = contracts.governance.finalize(&member1, &proposal_id);
        assert_eq!(status, ProposalStatus::Passed);
        contracts.governance.execute_proposal(&owner, &proposal_id);

        let members_after = contracts.governance.get_members();
        assert!(members_after.contains(new_member));
        assert_last_event(
            env,
            &contracts.governance_id,
            symbol_short!("gov").into_val(env),
            symbol_short!("exec").into_val(env),
        );
    }

    #[test]
    fn full_vault_workflow_lock_claim_vest_cliff_claim() {
        let contracts = setup();
        let env = &contracts.env;

        let admin = Address::generate(env);
        let signer = Address::generate(env);
        let owner = Address::generate(env);
        let beneficiary = Address::generate(env);
        contracts
            .vault
            .initialize(&admin, &Vec::from_array(env, [signer.clone()]), &1);

        env.ledger().set_timestamp(1_000);
        let lock_id = contracts
            .vault
            .lock_tokens(&owner, &600_000, &60, &symbol_short!("lock"));
        env.ledger().set_timestamp(1_060);
        assert_eq!(contracts.vault.claim(&owner, &lock_id), 600_000);

        env.ledger().set_timestamp(2_000);
        let vesting_id = contracts.vault.create_vesting(
            &admin,
            &beneficiary,
            &1_200_000,
            &120,
            &30,
            &symbol_short!("vest"),
        );
        env.ledger().set_timestamp(2_029);
        assert!(contracts
            .vault
            .try_claim_vested(&beneficiary, &vesting_id)
            .is_err());

        env.ledger().set_timestamp(2_060);
        assert_eq!(
            contracts.vault.claim_vested(&beneficiary, &vesting_id),
            600_000
        );
        env.ledger().set_timestamp(2_120);
        assert_eq!(
            contracts.vault.claim_vested(&beneficiary, &vesting_id),
            600_000
        );

        assert_eq!(contracts.vault.get_stats().total_locked, 0);
        assert_last_event(
            env,
            &contracts.vault_id,
            symbol_short!("vault").into_val(env),
            symbol_short!("v_claim").into_val(env),
        );
    }

    #[test]
    fn emergency_unlock_workflow_approves_threshold_then_unlocks() {
        let contracts = setup();
        let env = &contracts.env;

        let admin = Address::generate(env);
        let signer1 = Address::generate(env);
        let signer2 = Address::generate(env);
        let owner = Address::generate(env);
        let signers = Vec::from_array(env, [signer1.clone(), signer2.clone()]);
        contracts.vault.initialize(&admin, &signers, &2);

        let lock_id = contracts
            .vault
            .lock_tokens(&owner, &700_000, &3_600, &symbol_short!("safe"));
        contracts.vault.approve_emergency(&signer1, &lock_id);
        assert!(contracts
            .vault
            .try_emergency_unlock(&owner, &lock_id)
            .is_err());

        contracts.vault.approve_emergency(&signer2, &lock_id);
        assert_eq!(contracts.vault.emergency_unlock(&owner, &lock_id), 700_000);
        assert!(contracts.vault.get_lock(&lock_id).claimed);
        assert_last_event(
            env,
            &contracts.vault_id,
            symbol_short!("vault").into_val(env),
            symbol_short!("emrg_ex").into_val(env),
        );
    }

    #[test]
    fn edge_case_single_signer_treasury() {
        let contracts = setup();
        let env = &contracts.env;

        let admin = Address::generate(env);
        let signer = Address::generate(env);
        let token_admin = Address::generate(env);
        let asset = env
            .register_stellar_asset_contract_v2(token_admin)
            .address();
        StellarAssetClient::new(env, &asset).mint(&signer, &1_000_000);
        let recipient = Address::generate(env);
        let signers = Vec::from_array(env, [signer.clone()]);

        contracts.treasury.initialize(&admin, &1, &signers, &asset);
        contracts.treasury.deposit(&signer, &1_000_000);

        let tx_id = contracts.treasury.propose_withdrawal(
            &signer,
            &recipient,
            &500_000,
            &text(env, "single signer withdrawal"),
        );

        contracts.treasury.execute(&signer, &tx_id);
        assert!(contracts.treasury.get_transaction(&tx_id).executed);
        assert_eq!(contracts.treasury.get_balance(), 500_000);
    }

    #[test]
    fn edge_case_governance_100_percent_quorum() {
        let contracts = setup();
        let env = &contracts.env;

        let owner = Address::generate(env);
        let member1 = Address::generate(env);
        let member2 = Address::generate(env);
        let members = Vec::from_array(env, [member1.clone(), member2.clone()]);

        contracts.governance.initialize(&owner, &members, &100, &100);

        let proposal_id = contracts.governance.create_proposal(
            &member1,
            &text(env, "100% Quorum Proposal"),
            &text(env, "All members must vote"),
            &ProposalAction::General,
            &0,
            &Address::generate(env),
        );

        contracts.governance.vote(&member1, &proposal_id, &true);
        contracts.governance.vote(&member2, &proposal_id, &true);

        env.ledger().set_timestamp(200);
        contracts.governance.finalize(&member1, &proposal_id);

        let proposal = contracts.governance.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Passed);
    }

    #[test]
    fn edge_case_vault_zero_cliff_period() {
        let contracts = setup();
        let env = &contracts.env;

        let admin = Address::generate(env);
        let beneficiary = Address::generate(env);
        let token_admin = Address::generate(env);
        let asset = env
            .register_stellar_asset_contract_v2(token_admin)
            .address();
        StellarAssetClient::new(env, &asset).mint(&admin, &10_000_000);

        let signer1 = Address::generate(env);
        let signer2 = Address::generate(env);
        let signers = Vec::from_array(env, [signer1, signer2]);

        contracts.vault.initialize(&admin, &signers, &2, &asset);

        let vesting_id = contracts.vault.create_vesting(
            &admin,
            &beneficiary,
            &10_000_000,
            &365 * 86400,
            &0,
            &text(env, "zero cliff vesting"),
        );

        env.ledger().set_timestamp(1);
        let claimable = contracts.vault.get_vesting(&vesting_id).claimable_amount;
        assert!(claimable > 0);
    }

    #[test]
    fn edge_case_vesting_duration_equals_cliff() {
        let contracts = setup();
        let env = &contracts.env;

        let admin = Address::generate(env);
        let beneficiary = Address::generate(env);
        let token_admin = Address::generate(env);
        let asset = env
            .register_stellar_asset_contract_v2(token_admin)
            .address();
        StellarAssetClient::new(env, &asset).mint(&admin, &10_000_000);

        let signer1 = Address::generate(env);
        let signer2 = Address::generate(env);
        let signers = Vec::from_array(env, [signer1, signer2]);

        contracts.vault.initialize(&admin, &signers, &2, &asset);

        let duration = 365 * 86400;
        let vesting_id = contracts.vault.create_vesting(
            &admin,
            &beneficiary,
            &10_000_000,
            &duration,
            &duration,
            &text(env, "duration equals cliff"),
        );

        env.ledger().set_timestamp(duration + 1);
        let claimable = contracts.vault.get_vesting(&vesting_id).claimable_amount;
        assert_eq!(claimable, 10_000_000);
    }

    #[test]
    fn edge_case_treasury_balance_exact_withdrawal_amount() {
        let contracts = setup();
        let env = &contracts.env;

        let admin = Address::generate(env);
        let signer = Address::generate(env);
        let token_admin = Address::generate(env);
        let asset = env
            .register_stellar_asset_contract_v2(token_admin)
            .address();
        StellarAssetClient::new(env, &asset).mint(&signer, &5_000_000);
        let recipient = Address::generate(env);
        let signers = Vec::from_array(env, [signer.clone()]);

        contracts.treasury.initialize(&admin, &1, &signers, &asset);
        contracts.treasury.deposit(&signer, &5_000_000);

        let tx_id = contracts.treasury.propose_withdrawal(
            &signer,
            &recipient,
            &5_000_000,
            &text(env, "withdraw entire balance"),
        );

        contracts.treasury.execute(&signer, &tx_id);
        assert_eq!(contracts.treasury.get_balance(), 0);
    }

    #[test]
    fn edge_case_governance_proposal_at_exact_voting_deadline() {
        let contracts = setup();
        let env = &contracts.env;

        let owner = Address::generate(env);
        let member1 = Address::generate(env);
        let member2 = Address::generate(env);
        let members = Vec::from_array(env, [member1.clone(), member2.clone()]);

        contracts.governance.initialize(&owner, &members, &50, &100);

        let proposal_id = contracts.governance.create_proposal(
            &member1,
            &text(env, "Deadline Test"),
            &text(env, "Vote at exact deadline"),
            &ProposalAction::General,
            &0,
            &Address::generate(env),
        );

        contracts.governance.vote(&member1, &proposal_id, &true);

        env.ledger().set_timestamp(100);
        contracts.governance.vote(&member2, &proposal_id, &true);

        contracts.governance.finalize(&member1, &proposal_id);
        let proposal = contracts.governance.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Passed);
    }
}
