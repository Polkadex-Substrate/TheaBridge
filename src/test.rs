use ethers::prelude::LocalWallet;
use ethers::signers::Signer;
use k256::ecdsa;
use sp_core::{H256, Pair};
use sp_core::ecdsa::Signature;

#[test]
fn test_thea_sig() {
    let seed = String::from("c05c6ae125754dd17f36bcc5318498ce5c6c2f0e9e1116c68b77889a8be2ff02");
    let wallet: LocalWallet = seed.as_str().parse().unwrap();
    let wallet = wallet.with_chain_id(11155111u64);
    println!("Public key {:?}", wallet.address());
    let data = [1;32];
    println!("Message hex {:?}", hex::encode(data.clone()));
    let signature = wallet.sign_hash(H256(data)).unwrap();
    println!("Signature {:?}", signature);
}