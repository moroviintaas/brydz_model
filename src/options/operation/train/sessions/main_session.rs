use std::marker::PhantomData;
use std::thread;
use log::{debug, info};
use rand::distributions::Distribution;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use brydz_core::contract::{ContractMechanics, ContractParametersGen, ContractRandomizer};
use brydz_core::deal::{DealDistribution, DescriptionDeckDeal};
use brydz_core::player::side::Side;
use brydz_core::sztorm::comm::{ContractAgentSyncComm, ContractEnvSyncComm};
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractAgentInfoSetAllKnowing, ContractDummyState, ContractEnvStateComplete, ContractState};
use karty::suits::Suit;
use sztorm::agent::{AgentGen, AgentGenT, AgentTrajectory, AutomaticAgent, AutomaticAgentRewarded, EnvRewardedAgent, Policy, PolicyAgent, RandomPolicy, ResetAgent, StatefulAgent, TracingAgent};
use sztorm::env::{RoundRobinPenalisingUniversalEnvironment, StatefulEnvironment};
use sztorm::error::SztormError;
use sztorm::protocol::DomainParameters;
use sztorm::state::agent::{ConstructedInfoSet, ScoringInformationSet};
use sztorm::state::ConstructedState;
use sztorm_rl::agent::{NetworkLearningAgent, TestingAgent};
use sztorm_rl::error::SztormRLError;
use sztorm_rl::{LearningNetworkPolicy, TrainConfig};
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

pub type ContractInfoSetSeed<'a> = (&'a Side, &'a ContractParametersGen<Suit>, &'a DescriptionDeckDeal);
/*
pub struct TrainingSession<
    AgentD: NetworkLearningAgent<ContractDP>,
    AgentW: NetworkLearningAgent<ContractDP>,
    AgentO: NetworkLearningAgent<ContractDP>,
    AgentDTest: TestingAgent<ContractDP>,
    AgentWTest: TestingAgent<ContractDP>,
    AgentOTest: TestingAgent<ContractDP>,
    DIS2T: WayToTensor,
    WIS2T: WayToTensor,
    OIS2T: WayToTensor,
    DISTest2T: WayToTensor,
    WISTest2T: WayToTensor,
    OISTest2T: WayToTensor,

>
where
    <AgentD as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    <AgentD as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP> + ContractInfoSetForLearning<DIS2T>,
    <AgentW as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    <AgentW as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP> + ContractInfoSetForLearning<WIS2T>,
    <AgentO as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    <AgentO as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP> + ContractInfoSetForLearning<OIS2T>,

     <AgentDTest as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP> + ContractInfoSetForLearning<DISTest2T>,
     <AgentWTest as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP> + ContractInfoSetForLearning<WISTest2T>,
     <AgentOTest as StatefulAgent<ContractDP>>::State: ScoringInformationSet<ContractDP> + ContractInfoSetForLearning<OISTest2T>


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

    _dis2t: PhantomData<DIS2T>,
    _wis2t: PhantomData<WIS2T>,
    _ois2t: PhantomData<OIS2T>,
    _dis_test2t: PhantomData<DISTest2T>,
    _wis_test2t: PhantomData<WISTest2T>,
    _ois_test2t: PhantomData<OISTest2T>,
}
impl <
    AgentD: NetworkLearningAgent<ContractDP> + EnvRewardedAgent<ContractDP> + ResetAgent<ContractDP> + Send,
    AgentW: NetworkLearningAgent<ContractDP> + EnvRewardedAgent<ContractDP> + ResetAgent<ContractDP> + Send,
    AgentO: NetworkLearningAgent<ContractDP> + EnvRewardedAgent<ContractDP> + ResetAgent<ContractDP> + Send,
    AgentDTest: TestingAgent<ContractDP>  + EnvRewardedAgent<ContractDP> + ResetAgent<ContractDP> + Send,
    AgentWTest: TestingAgent<ContractDP>  + EnvRewardedAgent<ContractDP> + ResetAgent<ContractDP> + Send,
    AgentOTest: TestingAgent<ContractDP> +  EnvRewardedAgent<ContractDP> + ResetAgent<ContractDP> + Send,
    DIS2T: WayToTensor,
    WIS2T: WayToTensor,
    OIS2T: WayToTensor,
    DISTest2T: WayToTensor,
    WISTest2T: WayToTensor,
    OISTest2T: WayToTensor,
> TrainingSession<
    AgentD, AgentW, AgentO, AgentDTest, AgentWTest, AgentOTest,
    DIS2T,
    WIS2T,
    OIS2T,
    DISTest2T,
    WISTest2T,
    OISTest2T,

>
where
    <AgentD as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    <AgentD as StatefulAgent<ContractDP>>::State:  ContractInfoSetForLearning<DIS2T>,
    <AgentW as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    <AgentW as StatefulAgent<ContractDP>>::State: ContractInfoSetForLearning<WIS2T>,
    <AgentO as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    <AgentO as StatefulAgent<ContractDP>>::State: ContractInfoSetForLearning<OIS2T>,

     <AgentDTest as StatefulAgent<ContractDP>>::State: ContractInfoSetForLearning<DISTest2T>,
     <AgentWTest as StatefulAgent<ContractDP>>::State: ContractInfoSetForLearning<WISTest2T>,
     <AgentOTest as StatefulAgent<ContractDP>>::State: ContractInfoSetForLearning<OISTest2T>


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
        train_config: TrainConfig,
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
            _dis2t: Default::default(),
            _wis2t: Default::default(),
            _ois2t: Default::default(),
            _dis_test2t: Default::default(),
            _wis_test2t: Default::default(),
            _ois_test2t: Default::default(),
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

    fn prepare_game(&mut self, rng: &mut ThreadRng, distribution: &DealDistribution, contract_randomizer: &ContractRandomizer ){
        let deal = distribution.sample(rng);
        let deal_description = DescriptionDeckDeal{
            probabilities: distribution.clone(),
            cards: deal,
        };

        let contract = contract_randomizer.sample(rng);
        let old_declarer_side = self.environment.state().contract_data().declarer();
        self.environment.replace_state(ContractEnvStateComplete::construct_from((&contract, &deal_description)));
        self.declarer.reset(<AgentD as StatefulAgent<ContractDP>>::State::construct_from((&contract.declarer(), &contract, &deal_description)));
        self.whist.reset(<AgentW as StatefulAgent<ContractDP>>::State::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
        self.dummy.reset(ContractDummyState::construct_from((&contract.declarer().next_i(2), &contract, &deal_description)));
        self.offside.reset(<AgentO as StatefulAgent<ContractDP>>::State::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));

        self.declarer.change_id(contract.declarer());
        self.whist.change_id(contract.whist());
        self.dummy.change_id(contract.dummy());
        self.offside.change_id(contract.offside());
        self.environment.comms_mut().rotate(old_declarer_side, contract.declarer());

        debug!("Preparing game, trump: {}", &contract.bid().trump());
        debug!("Preparing game, declarer's side: {}", &contract.declarer());
    }

    fn prepare_test_game
    (
        &mut self,
        rng: &mut ThreadRng,
        distribution: &DealDistribution,
        contract_randomizer: &ContractRandomizer,
        tested_team: Team) {

        debug!("Preparing test game for team: {tested_team:?}");
        let deal = distribution.sample(rng);
        let deal_description = DescriptionDeckDeal{
            probabilities: distribution.clone(),
            cards: deal,
        };

        let contract = contract_randomizer.sample(rng);
        let old_declarer_side = self.environment.state().contract_data().declarer();
        self.environment.replace_state(ContractEnvStateComplete::construct_from((&contract, &deal_description)));
        self.dummy.reset(ContractDummyState::construct_from((&contract.declarer().next_i(2), &contract, &deal_description)));
        match tested_team{
            Team::Contractors => {
                self.declarer.reset(<AgentD as StatefulAgent<ContractDP>>::State::construct_from((&contract.declarer(), &contract, &deal_description)));
                self.test_whist.reset(<AgentWTest as StatefulAgent<ContractDP>>::State::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
                self.test_offside.reset(<AgentOTest as StatefulAgent<ContractDP>>::State::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));
            }
            Team::Defenders => {
                self.test_declarer.reset(<AgentDTest as StatefulAgent<ContractDP>>::State::construct_from((&contract.declarer(), &contract, &deal_description)));
                self.whist.reset(<AgentW as StatefulAgent<ContractDP>>::State::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
                self.offside.reset(<AgentO as StatefulAgent<ContractDP>>::State::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));
                debug!("Whist's , commited score: {}", self.whist.current_universal_score());
            }
        }

        self.declarer.change_id(contract.declarer());
        self.dummy.change_id(contract.dummy());
        self.offside.change_id(contract.offside());
        self.whist.change_id(contract.whist());
        self.environment.comms_mut().rotate(old_declarer_side, contract.declarer());
        self.test_whist.change_id(contract.whist());
        self.test_offside.change_id(contract.offside());
        self.test_declarer.change_id(contract.declarer());

        debug!("Preparing game, trump: {}", &contract.bid().trump());
        debug!("Preparing game, declarer's side: {}", &contract.declarer());
        debug!("Declarer ({}) cards: {:#}", &contract.declarer(), deal_description.cards[&contract.declarer()]);
        debug!("Whist ({}) cards: {:#}", &contract.whist(), deal_description.cards[&contract.whist()]);
        debug!("Dummy ({}) cards: {:#}", &contract.dummy(), deal_description.cards[&contract.dummy()]);
        debug!("Offside ({}) cards: {:#}", &contract.offside(), deal_description.cards[&contract.offside()]);


    }

    fn play_game(&mut self) -> Result<(), SztormRLError<ContractDP>>{
        thread::scope(|s|{
            s.spawn(||{
                self.environment.run_round_robin_uni_rewards_penalise(-100);
            });
            s.spawn(||{
                self.declarer.run_rewarded();
            });

            s.spawn(||{
                self.whist.run_rewarded();
            });

            s.spawn(||{
                self.dummy.run_rewarded();
            });

            s.spawn(||{
                self.offside.run_rewarded();
            });
        });
        Ok(())
    }

    fn play_test_game
    (&mut self, team: &Team) -> Result<(), SztormRLError<ContractDP>> {
        thread::scope(|s|{
            s.spawn(||{
                self.environment.run_round_robin_uni_rewards_penalise(-100);
            });

            s.spawn(||{
                self.dummy.run();
            });

            match team{
                Team::Contractors => {
                    s.spawn(||{
                        self.declarer.run_rewarded();
                    });
                    s.spawn(||{
                        self.test_whist.run();
                    });
                    s.spawn(||{
                        self.test_offside.run();
                    });

                }
                Team::Defenders => {
                    s.spawn(||{
                        self.whist.run_rewarded();
                    });
                    s.spawn(||{
                        self.test_declarer.run();
                    });
                    s.spawn(||{
                        self.offside.run_rewarded();
                    });
                }
            }
        });

        //self.declarer_rewards.push()
        Ok(())
    }

    fn stash_result(&mut self, team: &Team) {
        match team{
            Team::Contractors => {
                self.declarer_rewards.push(self.declarer.current_universal_score());
                //self.whist_rewards.push(test_agents.whist.current_universal_score());
                //self.offside_rewards.push(test_agents.offside.current_universal_score());
            }
            Team::Defenders => {
                //self.declarer_rewards.push(test_agents.declarer.current_universal_score());
                self.whist_rewards.push(self.whist.current_universal_score());
                self.offside_rewards.push(self.offside.current_universal_score());
            }
        }
    }

    fn clear_rewards(&mut self){
        self.declarer_rewards.clear();
        self.whist_rewards.clear();
        self.offside_rewards.clear();
    }

    fn stash_trajectories(&mut self){
        let declarer_trajectory = self.declarer.take_trajectory();
        if !declarer_trajectory.is_empty(){
            self.declarer_trajectories.push(declarer_trajectory);
        }
        let whist_trajectory = self.whist.take_trajectory();
        if !whist_trajectory.is_empty(){
            self.whist_trajectories.push(whist_trajectory);
        }
        let offside_trajectory = self.offside.take_trajectory();
        if !offside_trajectory.is_empty(){
            self.offside_trajectories.push(offside_trajectory);
        }

    }

    pub fn train_agents_separately_one_epoch(
        &mut self,
        games_in_epoch: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
    ) -> Result<(), SztormRLError<ContractDP>>{
        self.clear_trajectories();
        let mut rng = thread_rng();
        for _ in 0..games_in_epoch{

            let distr = if let Some(pool) = distribution_pool{
                pool.choose(&mut rng).unwrap_or(&DealDistribution::Fair)

            } else {
                &DealDistribution::Fair
            };
            self.prepare_game(&mut rng, distr, &contract_randomizer);
            self.play_game()?;
            self.stash_trajectories();



        }
        debug!("Declarer batch input sizes: {:?}", self.declarer_trajectories.iter().map(|v|v.list().len()).collect::<Vec<usize>>());
        debug!("Whist batch input sizes: {:?}", self.whist_trajectories.iter().map(|v|v.list().len()).collect::<Vec<usize>>());
        debug!("Offside batch input sizes: {:?}", self.offside_trajectories.iter().map(|v|v.list().len()).collect::<Vec<usize>>());

        if !self.declarer_trajectories.is_empty(){
            self.declarer.policy_mut().batch_train_on_universal_rewards(&self.declarer_trajectories[..])?;
        }
        if !self.whist_trajectories.is_empty(){
            self.whist.policy_mut().batch_train_on_universal_rewards(&self.whist_trajectories[..])?;
        }
        if !self.offside_trajectories.is_empty(){
            self.offside.policy_mut().batch_train_on_universal_rewards(&self.offside_trajectories[..])?;
        }

        Ok(())

    }

    pub fn test_agents_team(&mut self, rng: &mut ThreadRng, team: &Team, number_of_tests: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer)
        -> Result<f64, SztormError<ContractDP>> {


        self.clear_rewards();

        match team{
            Team::Contractors => {
                self.whist.swap_comms_with_basic(&mut self.test_whist);
                self.offside.swap_comms_with_basic(&mut self.test_offside);
                for _ in 0..number_of_tests {
                    let distr = if let Some(pool) = distribution_pool {
                        pool.choose(rng).unwrap_or(&DealDistribution::Fair)
                    } else {
                        &DealDistribution::Fair
                    };
                    self.prepare_test_game(rng, distr, contract_randomizer,  Team::Contractors);
                    let _ = self.play_test_game(team);
                    self.stash_result(team);

                }
                self.whist.swap_comms_with_basic(&mut self.test_whist);
                self.offside.swap_comms_with_basic(&mut self.test_offside);

                debug!("Declarer's rewards: {:?}", self.declarer_rewards);
                let average = self.declarer_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.declarer_rewards.len() as f64;
                info!("Testing declarer. Declarer's average reward: {average:}");
                Ok(average)
            }
            Team::Defenders => {
                self.declarer.swap_comms_with_basic(&mut self.test_declarer);
                for _ in 0..number_of_tests {
                    let distr = if let Some(pool) = distribution_pool {
                        pool.choose(rng).unwrap_or(&DealDistribution::Fair)
                    } else {
                        &DealDistribution::Fair
                    };
                    self.prepare_test_game(rng, distr, contract_randomizer,  Team::Defenders);
                    let _ = self.play_test_game(team);
                    self.stash_result(team);

                }
                self.declarer.swap_comms_with_basic(&mut self.test_declarer);

                debug!("Whist's rewards: {:?}, offside's rewards {:?}", self.whist_rewards, self.offside_rewards);
                let average_w = self.whist_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.whist_rewards.len() as f64;
                let average_o = self.offside_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.offside_rewards.len() as f64;
                info!("Testing defender's. Whist's average reward: {average_w:}. Offside's average reward: {average_o:}");
                Ok((average_w + average_o) / 2.0)


            }
        }


    }





}

 */

pub struct TSession<
    PolicyD: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    PolicyW: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    PolicyO: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    TestPolicyD: Policy<ContractDP>,
    TestPolicyW: Policy<ContractDP>,
    TestPolicyO: Policy<ContractDP>,
    DIS2T: WayToTensor,
    WIS2T: WayToTensor,
    OIS2T: WayToTensor,
    DISTest2T: WayToTensor,
    WISTest2T: WayToTensor,
    OISTest2T: WayToTensor,
>
where
    <PolicyD as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>,
    <PolicyW as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>,
    <PolicyO as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>,
    <TestPolicyD as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>,
    <TestPolicyW as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>,
    <TestPolicyO as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>,

{

    environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
    declarer: AgentGenT<ContractDP, PolicyD, ContractAgentSyncComm>,
    whist: AgentGenT<ContractDP, PolicyW, ContractAgentSyncComm>,
    offside: AgentGenT<ContractDP, PolicyO, ContractAgentSyncComm>,
    dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,

    test_declarer: AgentGenT<ContractDP, TestPolicyD, ContractAgentSyncComm>,
    test_whist: AgentGenT<ContractDP, TestPolicyW, ContractAgentSyncComm>,
    test_offside: AgentGenT<ContractDP, TestPolicyO, ContractAgentSyncComm>,

    declarer_trajectories: Vec<AgentTrajectory<ContractDP, <PolicyD as Policy<ContractDP>>::StateType>>,
    whist_trajectories: Vec<AgentTrajectory<ContractDP, <PolicyW as Policy<ContractDP>>::StateType>>,
    offside_trajectories: Vec<AgentTrajectory<ContractDP, <PolicyO as Policy<ContractDP>>::StateType>>,
    declarer_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    whist_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    offside_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,

    _dis2t: PhantomData<DIS2T>,
    _wis2t: PhantomData<WIS2T>,
    _ois2t: PhantomData<OIS2T>,
    _dis_test2t: PhantomData<DISTest2T>,
    _wis_test2t: PhantomData<WISTest2T>,
    _ois_test2t: PhantomData<OISTest2T>,



}
impl <
    PolicyD: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    PolicyW: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    PolicyO: LearningNetworkPolicy<ContractDP, TrainConfig=TrainConfig>,
    TestPolicyD: Policy<ContractDP>,
    TestPolicyW: Policy<ContractDP>,
    TestPolicyO: Policy<ContractDP>,
    DIS2T: WayToTensor,
    WIS2T: WayToTensor,
    OIS2T: WayToTensor,
    DISTest2T: WayToTensor,
    WISTest2T: WayToTensor,
    OISTest2T: WayToTensor,
> TSession<
    PolicyD,
    PolicyW,
    PolicyO,
    TestPolicyD,
    TestPolicyW,
    TestPolicyO,
    DIS2T,
    WIS2T,
    OIS2T,
    DISTest2T,
    WISTest2T,
    OISTest2T,
>
where
    <PolicyD as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP> + for<'a> ConstructedState<ContractDP, ContractInfoSetSeed<'a>> + Clone,
    <PolicyW as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>  + for<'a> ConstructedState<ContractDP, ContractInfoSetSeed<'a>>  + Clone,
    <PolicyO as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>  + for<'a> ConstructedState<ContractDP, ContractInfoSetSeed<'a>> + Clone,
    <TestPolicyD as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>  + for<'a> ConstructedState<ContractDP, ContractInfoSetSeed<'a>> + Clone,
    <TestPolicyW as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>  + for<'a> ConstructedState<ContractDP, ContractInfoSetSeed<'a>> + Clone,
    <TestPolicyO as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP>  + for<'a> ConstructedState<ContractDP, ContractInfoSetSeed<'a>> + Clone,
{
    pub(crate) fn _new(
        environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
        declarer: AgentGenT<ContractDP, PolicyD, ContractAgentSyncComm>,
        whist: AgentGenT<ContractDP, PolicyW, ContractAgentSyncComm>,
        offside: AgentGenT<ContractDP, PolicyO, ContractAgentSyncComm>,
        dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,

        test_declarer: AgentGenT<ContractDP, TestPolicyD, ContractAgentSyncComm>,
        test_whist: AgentGenT<ContractDP, TestPolicyW, ContractAgentSyncComm>,
        test_offside: AgentGenT<ContractDP, TestPolicyO, ContractAgentSyncComm>,
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
            _dis2t: Default::default(),
            _wis2t: Default::default(),
            _ois2t: Default::default(),
            _dis_test2t: Default::default(),
            _wis_test2t: Default::default(),
            _ois_test2t: Default::default(),
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

    fn prepare_game(&mut self, rng: &mut ThreadRng, distribution: &DealDistribution, contract_randomizer: &ContractRandomizer ){
        let deal = distribution.sample(rng);
        let deal_description = DescriptionDeckDeal{
            probabilities: distribution.clone(),
            cards: deal,
        };

        let contract = contract_randomizer.sample(rng);
        let old_declarer_side = self.environment.state().contract_data().declarer();
        self.environment.replace_state(ContractEnvStateComplete::construct_from((&contract, &deal_description)));
        self.declarer.reset(<PolicyD as Policy<ContractDP>>::StateType::construct_from((&contract.declarer(), &contract, &deal_description)));
        self.whist.reset(<PolicyW as Policy<ContractDP>>::StateType::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
        self.dummy.reset(ContractDummyState::construct_from((&contract.declarer().next_i(2), &contract, &deal_description)));
        self.offside.reset(<PolicyO as Policy<ContractDP>>::StateType::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));

        self.declarer.change_id(contract.declarer());
        self.whist.change_id(contract.whist());
        self.dummy.change_id(contract.dummy());
        self.offside.change_id(contract.offside());
        self.environment.comms_mut().rotate(old_declarer_side, contract.declarer());

        debug!("Preparing game, trump: {}", &contract.bid().trump());
        debug!("Preparing game, declarer's side: {}", &contract.declarer());
    }

    fn prepare_test_game
    (
        &mut self,
        rng: &mut ThreadRng,
        distribution: &DealDistribution,
        contract_randomizer: &ContractRandomizer,
        tested_team: Team) {

        debug!("Preparing test game for team: {tested_team:?}");
        let deal = distribution.sample(rng);
        let deal_description = DescriptionDeckDeal{
            probabilities: distribution.clone(),
            cards: deal,
        };

        let contract = contract_randomizer.sample(rng);
        let old_declarer_side = self.environment.state().contract_data().declarer();
        self.environment.replace_state(ContractEnvStateComplete::construct_from((&contract, &deal_description)));
        self.dummy.reset(ContractDummyState::construct_from((&contract.declarer().next_i(2), &contract, &deal_description)));
        match tested_team{
            Team::Contractors => {
                self.declarer.reset(<PolicyD as Policy<ContractDP>>::StateType::construct_from((&contract.declarer(), &contract, &deal_description)));
                self.test_whist.reset(<TestPolicyW as Policy<ContractDP>>::StateType::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
                self.test_offside.reset(<TestPolicyO as Policy<ContractDP>>::StateType::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));
            }
            Team::Defenders => {
                self.test_declarer.reset(<TestPolicyD as Policy<ContractDP>>::StateType::construct_from((&contract.declarer(), &contract, &deal_description)));
                self.whist.reset(<PolicyW as Policy<ContractDP>>::StateType::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
                self.offside.reset(<PolicyO as Policy<ContractDP>>::StateType::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));
                debug!("Whist's , commited score: {}", self.whist.current_universal_score());
            }
        }

        self.declarer.change_id(contract.declarer());
        self.dummy.change_id(contract.dummy());
        self.offside.change_id(contract.offside());
        self.whist.change_id(contract.whist());
        self.environment.comms_mut().rotate(old_declarer_side, contract.declarer());
        self.test_whist.change_id(contract.whist());
        self.test_offside.change_id(contract.offside());
        self.test_declarer.change_id(contract.declarer());

        debug!("Preparing game, trump: {}", &contract.bid().trump());
        debug!("Preparing game, declarer's side: {}", &contract.declarer());
        debug!("Declarer ({}) cards: {:#}", &contract.declarer(), deal_description.cards[&contract.declarer()]);
        debug!("Whist ({}) cards: {:#}", &contract.whist(), deal_description.cards[&contract.whist()]);
        debug!("Dummy ({}) cards: {:#}", &contract.dummy(), deal_description.cards[&contract.dummy()]);
        debug!("Offside ({}) cards: {:#}", &contract.offside(), deal_description.cards[&contract.offside()]);


    }

    fn play_game(&mut self) -> Result<(), SztormRLError<ContractDP>>{
        thread::scope(|s|{
            s.spawn(||{
                self.environment.run_round_robin_uni_rewards_penalise(-100);
            });
            s.spawn(||{
                self.declarer.run_rewarded();
            });

            s.spawn(||{
                self.whist.run_rewarded();
            });

            s.spawn(||{
                self.dummy.run_rewarded();
            });

            s.spawn(||{
                self.offside.run_rewarded();
            });
        });
        Ok(())
    }

    fn play_test_game
    (&mut self, team: &Team) -> Result<(), SztormRLError<ContractDP>> {
        thread::scope(|s|{
            s.spawn(||{
                self.environment.run_round_robin_uni_rewards_penalise(-100);
            });

            s.spawn(||{
                self.dummy.run();
            });

            match team{
                Team::Contractors => {
                    s.spawn(||{
                        self.declarer.run_rewarded();
                    });
                    s.spawn(||{
                        self.test_whist.run();
                    });
                    s.spawn(||{
                        self.test_offside.run();
                    });

                }
                Team::Defenders => {
                    s.spawn(||{
                        self.whist.run_rewarded();
                    });
                    s.spawn(||{
                        self.test_declarer.run();
                    });
                    s.spawn(||{
                        self.offside.run_rewarded();
                    });
                }
            }
        });

        //self.declarer_rewards.push()
        Ok(())
    }


    fn stash_result(&mut self, team: &Team) {
        match team{
            Team::Contractors => {
                self.declarer_rewards.push(self.declarer.current_universal_score());
                //self.whist_rewards.push(test_agents.whist.current_universal_score());
                //self.offside_rewards.push(test_agents.offside.current_universal_score());
            }
            Team::Defenders => {
                //self.declarer_rewards.push(test_agents.declarer.current_universal_score());
                self.whist_rewards.push(self.whist.current_universal_score());
                self.offside_rewards.push(self.offside.current_universal_score());
            }
        }
    }

    fn clear_rewards(&mut self){
        self.declarer_rewards.clear();
        self.whist_rewards.clear();
        self.offside_rewards.clear();
    }

    fn stash_trajectories(&mut self){
        let declarer_trajectory = self.declarer.take_trajectory();
        if !declarer_trajectory.is_empty(){
            self.declarer_trajectories.push(declarer_trajectory);
        }
        let whist_trajectory = self.whist.take_trajectory();
        if !whist_trajectory.is_empty(){
            self.whist_trajectories.push(whist_trajectory);
        }
        let offside_trajectory = self.offside.take_trajectory();
        if !offside_trajectory.is_empty(){
            self.offside_trajectories.push(offside_trajectory);
        }

    }

    pub fn train_agents_separately_one_epoch(
        &mut self,
        games_in_epoch: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
    ) -> Result<(), SztormRLError<ContractDP>>{
        self.clear_trajectories();
        let mut rng = thread_rng();
        for _ in 0..games_in_epoch{

            let distr = if let Some(pool) = distribution_pool{
                pool.choose(&mut rng).unwrap_or(&DealDistribution::Fair)

            } else {
                &DealDistribution::Fair
            };
            self.prepare_game(&mut rng, distr, &contract_randomizer);
            self.play_game()?;
            self.stash_trajectories();



        }
        debug!("Declarer batch input sizes: {:?}", self.declarer_trajectories.iter().map(|v|v.list().len()).collect::<Vec<usize>>());
        debug!("Whist batch input sizes: {:?}", self.whist_trajectories.iter().map(|v|v.list().len()).collect::<Vec<usize>>());
        debug!("Offside batch input sizes: {:?}", self.offside_trajectories.iter().map(|v|v.list().len()).collect::<Vec<usize>>());

        if !self.declarer_trajectories.is_empty(){
            self.declarer.policy_mut().batch_train_on_universal_rewards(&self.declarer_trajectories[..])?;
        }
        if !self.whist_trajectories.is_empty(){
            self.whist.policy_mut().batch_train_on_universal_rewards(&self.whist_trajectories[..])?;
        }
        if !self.offside_trajectories.is_empty(){
            self.offside.policy_mut().batch_train_on_universal_rewards(&self.offside_trajectories[..])?;
        }

        Ok(())

    }

    pub fn test_agents_team(&mut self, rng: &mut ThreadRng, team: &Team, number_of_tests: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer, )
        -> Result<f64, SztormError<ContractDP>> {


        self.clear_rewards();

        match team{
            Team::Contractors => {
                self.whist.swap_comms(&mut self.test_whist);
                self.offside.swap_comms(&mut self.test_offside);
                for _ in 0..number_of_tests {
                    let distr = if let Some(pool) = distribution_pool {
                        pool.choose(rng).unwrap_or(&DealDistribution::Fair)
                    } else {
                        &DealDistribution::Fair
                    };
                    self.prepare_test_game(rng, distr, contract_randomizer,  Team::Contractors);
                    let _ = self.play_test_game(team);
                    self.stash_result(team);

                }
                self.whist.swap_comms(&mut self.test_whist);
                self.offside.swap_comms(&mut self.test_offside);

                debug!("Declarer's rewards: {:?}", self.declarer_rewards);
                let average = self.declarer_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.declarer_rewards.len() as f64;
                info!("Testing declarer. Declarer's average reward: {average:}");
                Ok(average)
            }
            Team::Defenders => {
                self.declarer.swap_comms(&mut self.test_declarer);
                for _ in 0..number_of_tests {
                    let distr = if let Some(pool) = distribution_pool {
                        pool.choose(rng).unwrap_or(&DealDistribution::Fair)
                    } else {
                        &DealDistribution::Fair
                    };
                    self.prepare_test_game(rng, distr, contract_randomizer,  Team::Defenders);
                    let _ = self.play_test_game(team, );
                    self.stash_result(team);

                }
                self.declarer.swap_comms(&mut self.test_declarer);

                debug!("Whist's rewards: {:?}, offside's rewards {:?}", self.whist_rewards, self.offside_rewards);
                let average_w = self.whist_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.whist_rewards.len() as f64;
                let average_o = self.offside_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.offside_rewards.len() as f64;
                info!("Testing defender's. Whist's average reward: {average_w:}. Offside's average reward: {average_o:}");
                Ok((average_w + average_o) / 2.0)


            }
        }


    }

    pub fn test_agents(&mut self, number_of_tests: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer)
        -> Result<(f64, f64), SztormError<ContractDP>> {

        /*
        let (_, test_decl_comm) = ContractEnvSyncComm::new_pair();
        let (_, test_whist_comm) = ContractEnvSyncComm::new_pair();
        let (_, test_offside_comm) = ContractEnvSyncComm::new_pair();


         */
        let mut rng = thread_rng();
        let distr = if let Some(pool) = distribution_pool{
                pool.choose(&mut rng).unwrap_or(&DealDistribution::Fair)

            } else {
                &DealDistribution::Fair
            };

        let deal_description = DescriptionDeckDeal{
            probabilities: distr.clone(),
            cards: distr.sample(&mut thread_rng()),
        };

        let contract = contract_randomizer.sample(&mut rng);



        let declarer_score = self.test_agents_team(
            &mut rng,
            &Team::Contractors,
            number_of_tests,
            distribution_pool,
            contract_randomizer, )?;



        let defender_score = self.test_agents_team(
            &mut rng,
            &Team::Defenders,
            number_of_tests,
            distribution_pool,
            contract_randomizer,
            )?;


        Ok((declarer_score, defender_score))

    }



    /*
    pub fn train_agents_single_store_one_epoch(
        &mut self,
        games_in_epoch: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
    ) -> Result<(), SztormRLError<ContractDP>> {
        self.clear_trajectories();
        let mut rng = thread_rng();
        for _ in 0..games_in_epoch{

            let distr = if let Some(pool) = distribution_pool{
                pool.choose(&mut rng).unwrap_or(&DealDistribution::Fair)

            } else {
                &DealDistribution::Fair
            };
            self.prepare_game(&mut rng, distr, &contract_randomizer);
            self.play_game()?;

            self.stash_trajectories_all_to_declarer();

        }

        if !self.declarer_trajectories.is_empty(){
            self.declarer.policy_mut().batch_train_on_universal_rewards(&self.declarer_trajectories[..])?;
        }
        self.whist.policy_mut().var_store_mut().copy(self.declarer.policy().var_store()).unwrap();
        self.offside.policy_mut().var_store_mut().copy(self.declarer.policy().var_store()).unwrap();
        Ok(())
    }

     */

    pub fn train_all_at_once(
        &mut self,
        epochs: usize,
        games_in_epoch: usize,
        games_in_test: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
    ) -> Result<(), SztormRLError<ContractDP>> {

        println!("Przed testem początkowym");
        self.test_agents(games_in_test, distribution_pool, contract_randomizer)?;
        println!("Po teście początkowym");
        for e in 1..=epochs{
            self.train_agents_separately_one_epoch(games_in_epoch, distribution_pool, contract_randomizer)?;
            //self.train_agents_singe_store_one_epoch(games_in_epoch, distribution_pool, contract_randomizer)?;
            info!("Completed epoch {e:} of training.");
            let _test_results = self.test_agents(games_in_test, distribution_pool, contract_randomizer)?;
        }
        Ok(())
    }


}

