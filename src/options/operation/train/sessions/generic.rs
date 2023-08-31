use rand::prelude::Distribution;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use tch::nn::Optimizer;
use brydz_core::contract::ContractRandomizer;
use brydz_core::deal::{DealDistribution, DescriptionDeckDeal};
use brydz_core::player::side::{Side, SideMap};
use brydz_core::sztorm::comm::{ContractAgentSyncComm, ContractEnvSyncComm};
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractDummyState, ContractEnvStateComplete};
use sztorm::agent::{AgentGen, AgentGenT, AgentTrajectory, RandomPolicy, ResetAgent};
use sztorm::protocol::DomainParameters;
use sztorm::state::ConstructedState;
use sztorm_rl::actor_critic::ActorCriticPolicy;
use sztorm_rl::tensor_repr::WayToTensor;
use crate::options::operation::sessions::{ContractA2CAgentLocalGen, ContractInfoSetForLearning};

pub type ContractA2CLocalAgent<ISW, S> = AgentGenT<
    ContractDP,
    ActorCriticPolicy<
        ContractDP,
        S,
        ISW>,
    ContractAgentSyncComm>;




pub struct GenericContractA2CSession<
    DISW2T: WayToTensor,
    WISW2T: WayToTensor,
    OISW2T: WayToTensor,
    DIS: ContractInfoSetForLearning<DISW2T> + Clone,
    WIS: ContractInfoSetForLearning<WISW2T> + Clone,
    OIS: ContractInfoSetForLearning<OISW2T> + Clone,
>{
    environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
    declarer: ContractA2CLocalAgent<DISW2T, DIS>,
    whist: ContractA2CLocalAgent<WISW2T, WIS>,
    offside: ContractA2CLocalAgent<OISW2T, OIS>,
    dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,

    declarer_trajectories: Vec<AgentTrajectory<ContractDP, DIS>>,
    whist_trajectories: Vec<AgentTrajectory<ContractDP, WIS>>,
    offside_trajectories: Vec<AgentTrajectory<ContractDP, OIS>>,
    declarer_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    whist_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    offside_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,





}



impl<
    DISW2T: WayToTensor,
    WISW2T: WayToTensor,
    OISW2T: WayToTensor,
    DIS: ContractInfoSetForLearning<DISW2T> + Clone,
    WIS: ContractInfoSetForLearning<WISW2T> + Clone,
    OIS: ContractInfoSetForLearning<OISW2T> + Clone,
> GenericContractA2CSession<
    DISW2T,
    WISW2T,
    OISW2T,
    DIS,
    WIS,
    OIS>{

    pub fn new_rand_init(
        declarer_policy: ActorCriticPolicy<ContractDP, DIS, DISW2T>,
        whist_policy: ActorCriticPolicy<ContractDP, WIS, WISW2T>,
        offside_policy: ActorCriticPolicy<ContractDP, OIS, OISW2T>
        ) -> Self{
        let mut rng = thread_rng();
        let contract_params = ContractRandomizer::default().sample(&mut rng);
        let deal_description = DescriptionDeckDeal{
            probabilities: DealDistribution::Fair,
            cards: DealDistribution::Fair.sample(&mut rng)
        };
        let (comm_env_decl, comm_decl_env) = ContractEnvSyncComm::new_pair();
        let (comm_env_whist, comm_whist_env) = ContractEnvSyncComm::new_pair();
        let (comm_env_dummy, comm_dummy_env) = ContractEnvSyncComm::new_pair();
        let (comm_env_offside, comm_offside_env) = ContractEnvSyncComm::new_pair();


        let declarer = ContractA2CLocalAgent::new(
            contract_params.declarer(),
            DIS::construct_from((&contract_params.declarer(), &contract_params, &deal_description)),
            comm_decl_env, declarer_policy);

        let whist = ContractA2CLocalAgent::new(
            contract_params.declarer().next_i(1),
            WIS::construct_from((&contract_params.declarer().next_i(1), &contract_params, &deal_description)),
            comm_whist_env, whist_policy);

        let offside = ContractA2CLocalAgent::new(
            contract_params.declarer().next_i(3),
            OIS::construct_from((&contract_params.declarer().next_i(3), &contract_params, &deal_description)),
            comm_offside_env, offside_policy);

        let dummy = AgentGen::new(
            contract_params.declarer().next_i(2),
            ContractDummyState::construct_from((&contract_params.declarer().next_i(2), &contract_params, &deal_description)), comm_dummy_env, RandomPolicy::new(), );

        let (north_comm, east_comm, south_comm, west_comm) = match contract_params.declarer() {
            Side::East => (comm_env_offside, comm_env_decl, comm_env_whist, comm_env_dummy),
            Side::South => (comm_env_dummy, comm_env_offside, comm_env_decl, comm_env_whist),
            Side::West => (comm_env_whist, comm_env_dummy, comm_env_offside, comm_env_decl),
            Side::North => ( comm_env_decl, comm_env_whist, comm_env_dummy, comm_env_offside),
        };
        let environment = ContractEnv::new(
            ContractEnvStateComplete::construct_from((&contract_params, &deal_description)),
            SideMap::new(north_comm, east_comm, south_comm, west_comm));



        Self{
            environment,
            dummy,
            declarer,
            whist,
            offside,
            declarer_trajectories: Vec::new(),
            whist_trajectories: Vec::new(),
            offside_trajectories: Vec::new(),
            declarer_rewards: Vec::new(),
            whist_rewards: Vec::new(),
            offside_rewards: Vec::new()

        }
    }
/*
    fn prepare_game(&mut self, rng: &mut ThreadRng, distribution: &DealDistribution, contract_randomizer: &ContractRandomizer ){
        let deal = distribution.sample(rng);
        let deal_description = DescriptionDeckDeal{
            probabilities: distribution.clone(),
            cards: deal,
        };

        let contract = contract_randomizer.sample(rng);
        self.stash_trajectories();
        self.environment.replace_state(ContractEnvStateComplete::construct_from((&contract, &deal_description)));
        self.declarer.0.reset(S::construct_from((&contract.declarer(), &contract, &deal_description)));
        self.whist.0.reset(S::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
        self.dummy.reset(ContractDummyState::construct_from((&contract.declarer().next_i(2), &contract, &deal_description)));
        self.offside.0.reset(S::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));

        self.declarer.change_id(contract.declarer());
        self.whist.change_id(contract.whist());
        self.dummy.change_id(contract.dummy());
        self.offside.change_id(contract.offside());
    }

*/
}