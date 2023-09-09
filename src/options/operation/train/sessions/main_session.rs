use std::marker::PhantomData;
use brydz_core::sztorm::comm::{ContractAgentSyncComm, ContractEnvSyncComm};
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractDummyState, ContractEnvStateComplete};
use sztorm::agent::{AgentGen, AgentTrajectory, Policy, RandomPolicy};
use sztorm::protocol::DomainParameters;
use sztorm::state::agent::ScoringInformationSet;
use sztorm_rl::error::SztormRLError;
use sztorm_rl::tensor_repr::WayToTensor;
use crate::options::operation::sessions::{AgentType, ContractInfoSetForLearning, SessionAgentTrait};
use crate::options::operation::TrainOptions;




pub struct TrainingSession<
    DISW2T: WayToTensor,
    WISW2T: WayToTensor,
    OISW2T: WayToTensor,
    DIS: ContractInfoSetForLearning<DISW2T> + Clone,
    WIS: ContractInfoSetForLearning<WISW2T> + Clone,
    OIS: ContractInfoSetForLearning<OISW2T> + Clone,
    PolicyD: Policy<ContractDP, StateType=DIS>,
    PolicyW: Policy<ContractDP, StateType=WIS>,
    PolicyO: Policy<ContractDP, StateType=OIS>,
    PolicyDTest: Policy<ContractDP, StateType=DIS>,
    PolicyWTest: Policy<ContractDP, StateType=WIS>,
    PolicyOTest: Policy<ContractDP, StateType=OIS>,
    AgentD: SessionAgentTrait<DISW2T, PolicyD>,
    AgentW: SessionAgentTrait<WISW2T, PolicyW>,
    AgentO: SessionAgentTrait<OISW2T, PolicyO>,
    AgentDTest: SessionAgentTrait<DISW2T, PolicyDTest>,
    AgentWTest: SessionAgentTrait<WISW2T, PolicyWTest>,
    AgentOTest: SessionAgentTrait<OISW2T, PolicyOTest>,

>{
    environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
    declarer: AgentD,
    whist: AgentW,
    offside: AgentO,
    dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,

    declarer_tester: AgentDTest,
    whist_tester: AgentWTest,
    offside_tester: AgentOTest,

    declarer_trajectories: Vec<AgentTrajectory<ContractDP, DIS>>,
    whist_trajectories: Vec<AgentTrajectory<ContractDP, WIS>>,
    offside_trajectories: Vec<AgentTrajectory<ContractDP, OIS>>,
    declarer_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    whist_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    offside_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    _disw2t: PhantomData<DISW2T>,
    _wisw2t: PhantomData<WISW2T>,
    _oisw2t: PhantomData<OISW2T>,
    _policy_d: PhantomData<PolicyD>,
    _policy_w: PhantomData<PolicyW>,
    _policy_o: PhantomData<PolicyO>,
    _policy_d_test: PhantomData<PolicyDTest>,
    _policy_w_test: PhantomData<PolicyWTest>,
    _policy_o_test: PhantomData<PolicyOTest>,


}

impl <
    DISW2T: WayToTensor,
    WISW2T: WayToTensor,
    OISW2T: WayToTensor,
    DIS: ContractInfoSetForLearning<DISW2T> + Clone,
    WIS: ContractInfoSetForLearning<WISW2T> + Clone,
    OIS: ContractInfoSetForLearning<OISW2T> + Clone,
    PolicyD: Policy<ContractDP, StateType=DIS>,
    PolicyW: Policy<ContractDP, StateType=WIS>,
    PolicyO: Policy<ContractDP, StateType=OIS>,
    PolicyDTest: Policy<ContractDP, StateType=DIS>,
    PolicyWTest: Policy<ContractDP, StateType=WIS>,
    PolicyOTest: Policy<ContractDP, StateType=OIS>,
    AgentD: SessionAgentTrait<DISW2T, PolicyD>,
    AgentW: SessionAgentTrait<WISW2T, PolicyW>,
    AgentO: SessionAgentTrait<OISW2T, PolicyO>,
    AgentDTest: SessionAgentTrait<DISW2T, PolicyDTest>,
    AgentWTest: SessionAgentTrait<WISW2T, PolicyWTest>,
    AgentOTest: SessionAgentTrait<OISW2T, PolicyOTest>,

> TrainingSession<
    DISW2T,
    WISW2T,
    OISW2T,
    DIS,
    WIS,
    OIS,
    PolicyD,
    PolicyW,
    PolicyO,
    PolicyDTest,
    PolicyWTest,
    PolicyOTest,
    AgentD,
    AgentW,
    AgentO,
    AgentDTest,
    AgentWTest,
    AgentOTest
>{
    /*
    pub(crate) fn new_(
        environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
        declarer: AgentD,
        whist: AgentW,
        offside: AgentO,
        dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,
        declarer_tester: AgentDTest,
        whist_tester: AgentWTest,
        offside_tester: AgentOTest,

    ) -> Self{
        Self{
            environment, declarer, whist, offside, dummy,
            declarer_tester, whist_tester, offside_tester,
            declarer_trajectories: Vec::new(),
            whist_trajectories: Vec::new(),
            offside_trajectories: Vec::new(),
            declarer_rewards: Vec::new(),
            whist_rewards: Vec::new(),
            offside_rewards: Vec::new(),
            _disw2t: Default::default(),
            _wisw2t: Default::default(),
            _oisw2t: Default::default(),
            _policy_d: Default::default(),
            _policy_w: Default::default(),
            _policy_o: Default::default(),
            _policy_d_test: Default::default(),
            _policy_w_test: Default::default(),
            _policy_o_test: Default::default(),

        }
    }

     */


}



pub fn run_train_session(main_agent_policy_type: &AgentType,  options: TrainOptions) -> SztormRLError<ContractDP>{

    //let declarer = AgentD

    todo!()
}