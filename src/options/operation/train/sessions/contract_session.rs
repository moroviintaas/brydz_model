/*
use brydz_core::sztorm::comm::{ContractAgentSyncComm, ContractEnvSyncComm};
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractEnvStateComplete, ContractState};
use sztorm::agent::AgentGenT;
use sztorm::comm::EnvCommEndpoint;
use sztorm_rl::actor_critic::ActorCriticPolicy;

type AgentA2C<S> = AgentGenT<ContractDP, ActorCriticPolicy<DP, S, >>

pub struct ContractA2CSession<S: ContractState>{
    environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
    declarer: AgentGenT<
        ContractDP,
        ActorCriticPolicy<DP, InfoSet, StateConverter>,
        ContractAgentSyncComm>
}
*/