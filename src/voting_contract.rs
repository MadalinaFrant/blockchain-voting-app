#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait VotingContract {

    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[view(getCandidates)]
    #[storage_mapper("candidates")]
    fn candidates(&self) -> VecMapper<ManagedBuffer>;

    #[view(getVoters)]
    #[storage_mapper("voters")]
    fn voters(&self) -> MapMapper<ManagedAddress, bool>;

    #[view(getVotes)]
    #[storage_mapper("votes")]
    fn votes(&self) -> MapMapper<ManagedBuffer, u64>;

    #[only_owner]
    #[endpoint]
    fn add_candidate(&self, candidate: ManagedBuffer) {
        require!(
            !candidate.is_empty(), 
            "Candidate name cannot be empty."
        );

        require!(
            !self.candidates().iter().any(|c| c == candidate), 
            "Candidate is already registered."
        );

        self.candidates().push(&candidate);
        self.votes().insert(candidate.clone(), 0);

        self.candidate_added(candidate);
    }

    #[only_owner]
    #[endpoint]
    fn register_voter(&self, voter: ManagedAddress) {
        require!(
            !self.voters().contains_key(&voter), 
            "Voter is already registered."
        );

        self.voters().insert(voter.clone(), false);

        self.voter_registered(voter);
    }

    #[endpoint]
    fn vote(&self, candidate: ManagedBuffer) {
        let caller = self.blockchain().get_caller();

        require!(
            self.candidates().iter().any(|c| c == candidate), 
            "Candidate doesn't exist."
        );

        require!(
            self.voters().contains_key(&caller), 
            "Voter is not registered."
        );

        require!(
            !self.voters().get(&caller).unwrap(), 
            "Voter has already voted."
        );

        let votes = self.votes().get(&candidate).unwrap_or(0);
        self.votes().insert(candidate.clone(), votes + 1);
        self.voters().insert(caller.clone(), true);

        self.vote_cast(caller, candidate);
    }

    #[event("candidateAdded")]
    fn candidate_added(
        &self, 
        #[indexed] candidate: ManagedBuffer
    );

    #[event("voterRegistered")]
    fn voter_registered(
        &self, 
        #[indexed] voter: ManagedAddress
    );

    #[event("voteCast")]
    fn vote_cast(
        &self, 
        #[indexed] voter: ManagedAddress, 
        #[indexed] candidate: ManagedBuffer
    );

}
