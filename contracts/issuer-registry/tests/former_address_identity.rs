//! Issuer identity survives address rotation, for dispute conflict checks.

use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

fn hash(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

#[test]
fn current_and_former_addresses_resolve_to_the_same_issuer_id() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let client = IssuerRegistryContractClient::new(&env, &env.register(IssuerRegistryContract, ()));
    client.initialize(&admin);
    let issuer = Address::generate(&env);
    let issuer_id = hash(&env, 1);
    client.register_issuer(&issuer_id, &issuer, &hash(&env, 2), &hash(&env, 3));

    let stranger = Address::generate(&env);
    assert_eq!(
        client.get_issuer_id_for_address(&issuer),
        Some(issuer_id.clone())
    );
    assert_eq!(client.get_issuer_id_for_address(&stranger), None);

    let rotated = Address::generate(&env);
    client.rotate_issuer_address(&issuer_id, &rotated);
    client.accept_issuer_address_rotation(&issuer_id);

    assert_eq!(
        client.get_issuer_id_for_address(&rotated),
        Some(issuer_id.clone())
    );
    // The old address is no longer active, but it still names the same issuer.
    assert!(!client.is_active_address(&issuer));
    assert_eq!(client.get_issuer_id_for_address(&issuer), Some(issuer_id));
    assert_eq!(client.get_issuer_id_for_address(&stranger), None);
}
