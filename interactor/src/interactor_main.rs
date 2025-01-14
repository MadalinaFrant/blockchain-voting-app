#![allow(non_snake_case)]

mod proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};


const GATEWAY: &str = sdk::gateway::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";


#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "upgrade" => interact.upgrade().await,
        "getCandidates" => interact.candidates().await,
        "getVoters" => interact.voters().await,
        "getVotes" => interact.votes().await,
        "add_candidate" => interact.add_candidate().await,
        "register_voter" => interact.register_voter().await,
        "vote" => interact.vote().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}


#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    contract_address: Option<Bech32Address>
}

impl State {
        // Deserializes state from file
        pub fn load_state() -> Self {
            if Path::new(STATE_FILE).exists() {
                let mut file = std::fs::File::open(STATE_FILE).unwrap();
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                toml::from_str(&content).unwrap()
            } else {
                Self::default()
            }
        }
    
        /// Sets the contract address
        pub fn set_address(&mut self, address: Bech32Address) {
            self.contract_address = Some(address);
        }
    
        /// Returns the contract address
        pub fn current_address(&self) -> &Bech32Address {
            self.contract_address
                .as_ref()
                .expect("no known contract, deploy first")
        }
    }
    
    impl Drop for State {
        // Serializes state to file
        fn drop(&mut self) {
            let mut file = std::fs::File::create(STATE_FILE).unwrap();
            file.write_all(toml::to_string(self).unwrap().as_bytes())
                .unwrap();
        }
    }

struct ContractInteract {
    interactor: Interactor,
    wallet_address: Address,
    contract_code: BytesValue,
    state: State
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::alice());

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/voting-contract.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state()
        }
    }

    async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::VotingContractProxy)
            .init()
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_address(Bech32Address::from_bech32_string(new_address_bech32.clone()));

        println!("new address: {new_address_bech32}");
    }

    async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::VotingContractProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn candidates(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::VotingContractProxy)
            .candidates()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        for candidate in result_value.iter() {
            println!("{:?}", 
                String::from_utf8(candidate.to_vec()).unwrap());
        }

    }

    async fn voters(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::VotingContractProxy)
            .voters()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        for MultiValue2((voter, has_voted)) in result_value.iter() {
            println!("Voter: {:?} - has voted: {:?}", 
                bech32::encode(voter), has_voted);
        }
    }

    async fn votes(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::VotingContractProxy)
            .votes()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        for MultiValue2((candidate, votes)) in result_value.iter() {
            println!("Candidate: {:?} - Votes: {:?}", 
                String::from_utf8(candidate.to_vec()).unwrap(), votes);
        }
    }

    async fn add_candidate(&mut self) {
        let candidate = ManagedBuffer::new_from_bytes(&b"Alice"[..]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::VotingContractProxy)
            .add_candidate(candidate)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn register_voter(&mut self) {

        let mut voterAddr = String::new();

        match test_wallets::alice().address().to_bech32_string() {
            Ok(v) => {
                voterAddr = v;
            },
            Err(e) => {
                println!("Error: {e}");
            }
        }

        let voter = bech32::decode(&voterAddr);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::VotingContractProxy)
            .register_voter(voter)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn vote(&mut self) {
        let candidate = ManagedBuffer::new_from_bytes(&b"Alice"[..]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::VotingContractProxy)
            .vote(candidate)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

}
