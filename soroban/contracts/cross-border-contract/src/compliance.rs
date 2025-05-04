use crate::types::*;
use soroban_sdk::{Address, BytesN, Env, Symbol};

pub fn verify_compliance(env: Env, user: Address, documents: BytesN<32>) {
    user.require_auth();

    // Simulate compliance check (integrate with external KYC/AML service)
    let compliance = ComplianceData {
        kyc_verified: true,
        aml_verified: true,
        verification_documents: documents.clone(),
    };

    env.storage()
        .instance()
        .set(&DataKey::Compliance(user.clone()), &compliance);

    env.events().publish(
        (Symbol::new(&env, "ComplianceVerified"),),
        (
            user,
            compliance.kyc_verified,
            compliance.aml_verified,
            documents,
        ),
    );
}

pub fn is_compliant(env: &Env, user: &Address) -> bool {
    let compliance: ComplianceData = env
        .storage()
        .instance()
        .get(&DataKey::Compliance(user.clone()))
        .unwrap_or(ComplianceData {
            kyc_verified: false,
            aml_verified: false,
            verification_documents: BytesN::from_array(&env, &[0; 32]),
        });
    compliance.kyc_verified && compliance.aml_verified
}
