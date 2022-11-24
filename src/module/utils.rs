use crate::module::rpc_client::RpcClient;
use crate::GENESIS_VALIDATORS_ROOT;
use lighthouse_bls::Hash256;
use lighthouse_ssz::Encode;
use lighthouse_types::{
    BeaconBlockHeader, ChainSpec, Domain, EthSpec, MainnetEthSpec, SigningData, SyncAggregate,
    SyncCommittee,
};
use ruc::*;
use sha2::digest::Digest;
use sha2::Sha256;
use std::sync::Arc;

pub async fn compute_period_at_slot<T: EthSpec>(
    slot: u64,
    rpc_client: Arc<RpcClient>,
    spec: &ChainSpec,
) -> Result<u64> {
    let block = rpc_client
        .beacon_get_block_by_slot::<T>(slot)
        .await
        .c(d!())?;

    let period = block
        .slot()
        .epoch(MainnetEthSpec::slots_per_epoch())
        .sync_committee_period(spec)
        .map_err(|e| eg!("{:?}", e))
        .c(d!())?;

    Ok(period)
}

pub fn is_valid_merkle_branch(
    leaf: Hash256,
    branch: &[Hash256],
    depth: u32,
    index: u64,
    root: Hash256,
) -> Result<bool> {
    if branch.len() != depth as usize {
        return Err(eg!("Merkle proof branch length doesn't match depth."));
    }

    if leaf.as_bytes().len() < 32 {
        return Err(eg!("Merkle proof leaf not 32 bytes."));
    }

    let mut value = leaf;

    for i in 0..depth {
        let idx = i as usize;

        if branch[idx].0.len() < 32 {
            return Err(eg!("Merkle proof branch not 32 bytes."));
        }

        let pow = 2_u64.checked_pow(i).ok_or(eg!("pow overflow"))?;
        let res = index / pow % 2;

        let mut input = [0u8; 64];

        if res != 0 {
            input[0..32].copy_from_slice(&branch[idx].0);
            input[32..64].copy_from_slice(&value.0);
        } else {
            input[0..32].copy_from_slice(&value.0);
            input[32..64].copy_from_slice(&branch[idx].0);
        }

        value = sha256(&input);
    }

    Ok(value == root)
}

pub fn get_subtree_index(generalized_index: usize) -> Result<u64> {
    let pow = generalized_index.ilog2();
    let result = generalized_index
        % (2_usize
            .checked_pow(pow)
            .ok_or(eg!("checked pow overflow"))?);
    Ok(result as u64)
}

pub fn sha256(data: &[u8]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let h = Hash256::from_slice(&result);
    h
}

pub fn verify_signature<T: EthSpec>(
    sync_aggregate: &SyncAggregate<T>,
    current_sync_committee: &SyncCommittee<T>,
    spec: &ChainSpec,
    signed_header: &BeaconBlockHeader,
) -> Result<bool> {
    let mut pubkeys = vec![];

    for pubkey in current_sync_committee.pubkeys.iter() {
        let pk = pubkey.decompress().map_err(|e| eg!("{:?}", e)).c(d!())?;
        pubkeys.push(pk);
    }
    let pubkey_refs = pubkeys.iter().collect::<Vec<_>>();

    let fork_version = spec.genesis_fork_version;

    let domain = spec.compute_domain(
        Domain::SyncCommittee,
        fork_version,
        GENESIS_VALIDATORS_ROOT.get().unwrap().clone(),
    );

    let signing_root: Vec<u8> = SigningData {
        object_root: signed_header.canonical_root(),
        domain,
    }
    .as_ssz_bytes();

    let verify = sync_aggregate
        .sync_committee_signature
        .fast_aggregate_verify(Hash256::from_slice(&signing_root), &pubkey_refs);

    Ok(verify)
}
