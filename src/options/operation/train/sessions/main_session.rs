use std::marker::PhantomData;
use brydz_core::sztorm::comm::{ContractAgentSyncComm, ContractEnvSyncComm};
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractAgentInfoSetAllKnowing, ContractDummyState, ContractEnvStateComplete};
use sztorm::agent::{AgentGen, AgentGenT, AgentTrajectory, EnvRewardedAgent, Policy, PolicyAgent, RandomPolicy, StatefulAgent, TracingAgent};
use sztorm::protocol::DomainParameters;
use sztorm::state::agent::ScoringInformationSet;
use sztorm_rl::agent::{NetworkLearningAgent, TestingAgent};
use sztorm_rl::error::SztormRLError;
use sztorm_rl::LearningNetworkPolicy;
use sztorm_rl::tensor_repr::WayToTensor;
use crate::options::operation::sessions::{AgentType, ContractInfoSetForLearning, SessionAgentTrait, Team};
use crate::options::operation::TrainOptions;

/*
    <AgentD as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP,
    <AgentW as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>,

    <AgentW as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP,
    <AgentO as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>,
    <AgentO as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,
 */
pub struct TrainingSession<
    AgentD: NetworkLearningAgent<ContractDP>,
    AgentW: NetworkLearningAgent<ContractDP>,
    AgentO: NetworkLearningAgent<ContractDP>,
    AgentDTest: TestingAgent<ContractDP>,
    AgentWTest: TestingAgent<ContractDP>,
    AgentOTest: TestingAgent<ContractDP>,
>
where
    <AgentD as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>,
    <AgentD as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,
    <AgentW as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>,
    <AgentW as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,
    <AgentO as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>,
    <AgentO as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,

     <AgentDTest as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,
     <AgentWTest as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,
     <AgentOTest as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>


{
    environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
    declarer: AgentD,
    whist: AgentW,
    offside: AgentO,
    dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,

    test_declarer: AgentDTest,
    test_whist: AgentWTest,
    test_offside: AgentOTest,

    declarer_trajectories: Vec<AgentTrajectory<ContractDP, <AgentD as StatefulAgent<ContractDP>>::State>>,
    whist_trajectories: Vec<AgentTrajectory<ContractDP, <AgentW as StatefulAgent<ContractDP>>::State>>,
    offside_trajectories: Vec<AgentTrajectory<ContractDP, <AgentO as StatefulAgent<ContractDP>>::State>>,
    declarer_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    whist_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    offside_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
}
impl <
    AgentD: NetworkLearningAgent<ContractDP> + EnvRewardedAgent<ContractDP>,
    AgentW: NetworkLearningAgent<ContractDP> + EnvRewardedAgent<ContractDP>,
    AgentO: NetworkLearningAgent<ContractDP> + EnvRewardedAgent<ContractDP>,
    AgentDTest: TestingAgent<ContractDP>  + EnvRewardedAgent<ContractDP>,
    AgentWTest: TestingAgent<ContractDP>  + EnvRewardedAgent<ContractDP>,
    AgentOTest: TestingAgent<ContractDP> +  EnvRewardedAgent<ContractDP>,
> TrainingSession<
    AgentD, AgentW, AgentO, AgentDTest, AgentWTest, AgentOTest
>
where
    <AgentD as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>,
    <AgentD as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,
    <AgentW as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>,
    <AgentW as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,
    <AgentO as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>,
    <AgentO as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,

     <AgentDTest as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,
     <AgentWTest as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>,
     <AgentOTest as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP>


{
    pub(crate) fn _new(
        environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
        declarer: AgentD,
        whist: AgentW,
        offside: AgentO,
        dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,

        test_declarer: AgentDTest,
        test_whist: AgentWTest,
        test_offside: AgentOTest,
    ) -> Self{
        Self{
            environment,
            declarer,
            whist,
            offside,
            dummy,
            test_declarer,
            test_whist,
            test_offside,
            declarer_trajectories: Default::default(),
            whist_trajectories: Default::default(),
            offside_trajectories: Default::default(),
            declarer_rewards: Default::default(),
            whist_rewards: Default::default(),
            offside_rewards: Default::default(),
        }
    }

    fn clear_trajectories(&mut self){
        self.declarer.take_trajectory();
        self.whist.take_trajectory();
        self.offside.take_trajectory();
        self.offside_trajectories.clear();
        self.whist_trajectories.clear();
        self.declarer_trajectories.clear();
    }

    fn store_single_game_results_in_test(&mut self, team: &Team)
    {
        match team{
            Team::Contractors => {
                self.declarer_rewards.push(self.declarer.current_universal_score());
                self.whist_rewards.push(self.test_whist.current_universal_score());
                self.offside_rewards.push(self.test_offside.current_universal_score());
            }
            Team::Defenders => {
                self.declarer_rewards.push(self.test_declarer.current_universal_score());
                self.whist_rewards.push(self.whist.current_universal_score());
                self.offside_rewards.push(self.offside.current_universal_score());
            }
        }
    }



}

/*
pub struct TrainingSessionOld<
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
    AgentD: NSessionAgentTrait<DISW2T, PolicyD>,
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

> TrainingSessionOld<
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




}

 */

/*
pub struct ContractLearningSessionSymmetric<
    P: LearningNetworkPolicy<ContractDP>,

>
*/
/*
#[derive(Default)]
pub struct TrainingSessionBuilder<
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
    environment: Option<ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>>,
    declarer: Option<AgentD>,
    whist: Option<AgentW>,
    offside: Option<AgentO>,
    dummy: Option<AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>>,

    declarer_tester: Option<AgentDTest>,
    whist_tester: Option<AgentWTest>,
    offside_tester: Option<AgentOTest>,

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

> TrainingSessionBuilder <
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
    pub fn environment(&mut self, environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>){
        self.environment = Some(environment);
    }

    pub fn whist(&mut self, agent_whist: AgentW){
        self.whist = Some(agent_whist)
    }
    //pub fn declarer(&mut self, )
}


 */
/*
pub trait ContractLearningSessionTrait
where <<Self as ContractLearningSessionTrait>::AgentDeclarer as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>{
    type AgentDeclarer: NetworkLearningAgent<ContractDP>;
    type AgentWhist;
    type AgentOffside;
    type AgentDummy;

    fn declarer(&self) -> &Self::AgentDeclarer;
    fn whist(&self) -> &Self::AgentWhist;
    fn offside(&self) -> &Self::AgentOffside;
    fn dummy(&self) -> &Self::AgentDummy;

    fn declarer_mut(&mut self) -> &mut Self::AgentDeclarer;
    fn whist_mut(&mut self) -> &mut Self::AgentWhist;
    fn offside_mut(&mut self) -> &mut Self::AgentOffside;
    fn dummy_mut(&mut self) -> &mut Self::AgentDummy;

    fn environment(&self) -> &ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>;
    fn environment_mut(&mut self) -> &mut ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>;

    fn clear_trajectories(&mut self){
        self.declarer_mut().take_trajectory()
    }



}


 */


/*
pub trait ContractLearningSessionTrait
where <<Self as ContractLearningSessionTrait>::DeclarerPolicy as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>,
<<Self as ContractLearningSessionTrait>::WhistPolicy as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>,
<<Self as ContractLearningSessionTrait>::OffsidePolicy as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>{
    type DeclarerPolicy: LearningNetworkPolicy<ContractDP>;
    type WhistPolicy: LearningNetworkPolicy<ContractDP>;
    type OffsidePolicy: LearningNetworkPolicy<ContractDP>;


    fn declarer(&self) -> &AgentGenT<ContractDP, Self::DeclarerPolicy, ContractAgentSyncComm>;
    fn declarer_mut(&mut self) -> &mut AgentGenT<ContractDP, Self::DeclarerPolicy, ContractAgentSyncComm>;
    fn whist(&self) -> &AgentGenT<ContractDP, Self::DeclarerPolicy, ContractAgentSyncComm>;
    fn whist_mut(&mut self) -> &mut AgentGenT<ContractDP, Self::DeclarerPolicy, ContractAgentSyncComm>;
    fn offside(&self) -> &AgentGenT<ContractDP, Self::DeclarerPolicy, ContractAgentSyncComm>;
    fn offside_mut(&mut self) -> &mut AgentGenT<ContractDP, Self::DeclarerPolicy, ContractAgentSyncComm>;

    fn clear_trajectories(&mut self){
        self.declarer_mut().take_trajectory();
        self.whist_mut().take_trajectory();
        self.offside_mut().take_trajectory();

    }

}

 */

