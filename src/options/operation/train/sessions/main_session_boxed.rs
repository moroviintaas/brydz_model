use rand::distributions::Distribution;
use rand::thread_rng;
use brydz_core::contract::{ContractParameters, ContractRandomizer};
use brydz_core::deal::{DealDistribution, DescriptionDeckDeal};
use brydz_core::sztorm::comm::{ContractAgentSyncComm, ContractEnvSyncComm};
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractDummyState, ContractEnvStateComplete};
use sztorm::agent::{AgentGen, AgentGenT, AgentTrajectory, Policy, RandomPolicy};
use sztorm::protocol::DomainParameters;
use sztorm_rl::tensor_repr::WayToTensor;
use crate::options::operation::sessions::{AgentType, ContractInfoSetForLearning, SessionAgentTrait, SessionAgentTraitDyn};
use crate::options::operation::TrainOptions;

pub struct TrainingSessionBoxed<
    DISW2T: WayToTensor,
    WISW2T: WayToTensor,
    OISW2T: WayToTensor,
    DIS: ContractInfoSetForLearning<DISW2T> + Clone,
    WIS: ContractInfoSetForLearning<WISW2T> + Clone,
    OIS: ContractInfoSetForLearning<OISW2T> + Clone,
    PolicyD: Policy<ContractDP, StateType=DIS>,
    PolicyW: Policy<ContractDP, StateType=WIS>,
    PolicyO: Policy<ContractDP, StateType=OIS>,
>{

    environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
    declarer: Box<dyn SessionAgentTraitDyn<DISW2T, PolicyD>>,
    whist: Box<dyn SessionAgentTraitDyn<WISW2T, PolicyW>>,
    offside: Box<dyn SessionAgentTraitDyn<OISW2T, PolicyO>>,
    dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,

    declarer_test: Box<dyn SessionAgentTraitDyn<DISW2T, DIS>>,
    whist_test: Box<dyn SessionAgentTraitDyn<WISW2T, WIS>>,
    offside_test: Box<dyn SessionAgentTraitDyn<OISW2T, WIS>>,

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
    PolicyD: Policy<ContractDP, StateType=DIS>,
    PolicyW: Policy<ContractDP, StateType=WIS>,
    PolicyO: Policy<ContractDP, StateType=OIS>,
> TrainingSessionBoxed<
    DISW2T, WISW2T, OISW2T,
    DIS, WIS, OIS,
    PolicyD, PolicyW, PolicyO
>{

    fn _create_agent<ISW2T: WayToTensor, P: Policy<ContractDP, StateType=ISW2T>>(
        policy_learn_type: &AgentType,
        &contract_params: ContractParameters,
        deal_description: DescriptionDeckDeal
    ) -> Box<dyn SessionAgentTraitDyn<ISW2T, P>>{
        
    }

    pub fn new_from_training_config(policy_learn_type: &AgentType) -> Self{
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

        let (_, comm_decl_env) = ContractEnvSyncComm::new_pair();
        let (_, comm_whist_env) = ContractEnvSyncComm::new_pair();
        let (_, comm_offside_env) = ContractEnvSyncComm::new_pair();
        todo!()
        /*
        let (declarer, whist, offside) = match policy_learn_type{
            AgentType::ActorCritic(options) => {
                let declarer_policy =
                let d = AgentGenT::new(
                    contract_params.declarer(),
                    DIS
                )
            }
            AgentType::Q(options) => {}
        }

         */


    }
}