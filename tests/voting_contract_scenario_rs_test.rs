use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("mxsc:output/voting-contract.mxsc.json", voting_contract::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/voting_contract.scen.json");
}
