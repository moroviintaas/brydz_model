use std::fmt::{Debug, Display, Formatter, Pointer};
use std::marker::PhantomData;
use std::thread;
use log::info;
use rand::prelude::Distribution;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use smallvec::SmallVec;
use tch::nn::linear;
use brydz_core::contract::{Contract, ContractParameters, ContractRandomizer};
use brydz_core::deal::{BiasedHandDistribution, DealDistribution, DescriptionDeckDeal};
use brydz_core::meta::HAND_SIZE;
use brydz_core::player::axis::Axis;
use brydz_core::player::side::Side;
use brydz_core::sztorm::{
    comm::{
        ContractEnvSyncComm
    },
    env::{
        ContractEnv
    },
    state::{
        ContractEnvStateComplete
    }
};
use brydz_core::sztorm::comm::ContractAgentSyncComm;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractAction, ContractAgentInfoSetSimple, ContractDummyState, ContractInfoSet, ContractState, ContractStateConverter, CreatedContractInfoSet};
use karty::hand::CardSet;
use sztorm::agent::{AgentGen, AgentGenT, AgentTrajectory, AutomaticAgent, EnvRewardedAgent, Policy, RandomPolicy, ResetAgent, TracingAgent};
use sztorm::comm::EnvCommEndpoint;
use sztorm::env::RoundRobinUniversalEnvironment;
use sztorm::error::SztormError;
use sztorm::protocol::DomainParameters;
use sztorm::state::agent::{InformationSet, ScoringInformationSet};
use sztorm::state::ConstructedState;
use sztorm_rl::actor_critic::ActorCriticPolicy;
use sztorm_rl::error::SztormRLError;
use sztorm_rl::tensor_repr::{ConvertToTensor, ConvertToTensorD, ConvStateToTensor, WayFromTensor, WayToTensor};

/*
type ContractA2CAgentLocal<ISW> = AgentGenT<
    ContractDP,
    ActorCriticPolicy<
        ContractDP,
        Box<dyn CreatedContractInfoSet<
            ActionIteratorType=(),
            RewardType=()>
            + ConvertToTensor<ISW>
        >,
        Box<dyn ConvStateToTensor<>>>,
    ContractAgentSyncComm>;
*/
//trait InfoSetCompl =
pub struct ContractA2CAgentLocalGen<
    ISW: WayToTensor,
    S: ConvertToTensor<ISW>  + ScoringInformationSet<ContractDP>
        + for<'a> ConstructedState<ContractDP, (&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>
        +  Debug + Display + Clone>(
    pub AgentGenT<
        ContractDP,
        ActorCriticPolicy<
            ContractDP,
            S,
            //Box<dyn ConvertToTensorD<ISW>>,
            ISW>,
        ContractAgentSyncComm,
    >
);

pub struct ContractTestAgentTmp<
    ISW: WayToTensor,
    S: ConvertToTensor<ISW>  + ScoringInformationSet<ContractDP>
        + for<'a> ConstructedState<ContractDP, (&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>
        +  Debug + Clone>(
    pub AgentGen<
        ContractDP,
        RandomPolicy<ContractDP, S>,
        ContractAgentSyncComm>,
    PhantomData<ISW>
)
where <<S as InformationSet<ContractDP>>::ActionIteratorType as IntoIterator>::IntoIter: ExactSizeIterator;


pub trait ContractInfoSetForLearning<ISW: WayToTensor>:
ConvertToTensor<ISW>
+ for<'a> ConstructedState<ContractDP, (&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>
+ ScoringInformationSet<ContractDP>
+ Debug {}

impl<ISW: WayToTensor, T: ConvertToTensor<ISW>
+ for<'a> ConstructedState<ContractDP, (&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>
+ ScoringInformationSet<ContractDP>
+ Debug > ContractInfoSetForLearning<ISW> for T{}


#[derive(Copy, Clone, Debug)]
pub enum Team{
    Contractors,
    Defenders
}
impl Team{
    pub fn opposite(&self) -> Team{
        match self{
            Team::Contractors => Team::Defenders,
            Team::Defenders => Team::Contractors
        }
    }
}







//impl<ISW: WayToTensor, T: ContractInfoSetForLearning<ISW>> ContractInfoSetForLearning<ISW> for Box<T>{}
/*pub type ContractA2CAgentLocalBoxing<ISW, IS: ContractInfoSetTraitJoined<ISW>> = ContractA2CAgentLocalGen<
    ISW,
    //Box<dyn ContractInfoSetTraitJoined<
    //    ISW,
    //    ActionIteratorType=SmallVec<[ContractAction; HAND_SIZE]>,
    //    RewardType=i32
   //     >>
    IS
>;

 */



/*
pub struct DynamicContractA2CSession<
    ISW2T: WayToTensor,
    S: ContractInfoSetForLearning<ISW2T> + Clone>
where <<S as InformationSet<ContractDP>>::ActionIteratorType as IntoIterator>::IntoIter: ExactSizeIterator{
    environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
    declarer: ContractA2CAgentLocalGen<ISW2T, S>,
    whist: ContractA2CAgentLocalGen<ISW2T, S>,
    dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,
    offside: ContractA2CAgentLocalGen<ISW2T, S>,
    declarer_trajectories: Vec<AgentTrajectory<ContractDP, S>>,
    whist_trajectories: Vec<AgentTrajectory<ContractDP, S>>,
    offside_trajectories: Vec<AgentTrajectory<ContractDP, S>>,
    test_agent_declarer: ContractTestAgentTmp<ISW2T, S>,
    test_agent_whist: ContractTestAgentTmp<ISW2T, S>,
    test_agent_offside: ContractTestAgentTmp<ISW2T, S>,
    declarer_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    whist_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    offside_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    env_comm_test_declarer: ContractEnvSyncComm,
    env_comm_test_whist: ContractEnvSyncComm,
    env_comm_test_offside: ContractEnvSyncComm,
}

pub type BoxedContractA2CSession<ISW2T> = DynamicContractA2CSession<ISW2T, Box<dyn ContractInfoSetForLearning<
    ISW2T,
    ActionIteratorType=SmallVec<[ContractAction; HAND_SIZE]>,
    RewardType=i32>>>;




impl<ISW2T: WayToTensor> BoxedContractA2CSession<ISW2T>{
    fn set_declarer_state_type<
        S: ContractInfoSetForLearning<
            ISW2T,
            ActionIteratorType=SmallVec<[ContractAction; HAND_SIZE]>,
            RewardType=i32>>(&mut self){


    }
}

 */
/*
impl<ISW2T: WayToTensor, S: ContractInfoSetForLearning<ISW2T> + Clone> DynamicContractA2CSession<ISW2T, S>
where <<S as InformationSet<ContractDP>>::ActionIteratorType as IntoIterator>::IntoIter: ExactSizeIterator{

    pub(crate) fn _new(
        environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
        declarer: ContractA2CAgentLocalGen<ISW2T, S>,
        whist: ContractA2CAgentLocalGen<ISW2T, S>,
        dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,
        offside: ContractA2CAgentLocalGen<ISW2T, S>,
        test_agent_declarer: ContractTestAgentTmp<ISW2T, S>,
        test_agent_whist: ContractTestAgentTmp<ISW2T, S>,
        test_agent_offside: ContractTestAgentTmp<ISW2T, S>,
        env_comm_test_declarer: ContractEnvSyncComm,
        env_comm_test_whist: ContractEnvSyncComm,
        env_comm_test_offside: ContractEnvSyncComm,
    ) -> Self{
        Self{environment, declarer, whist, dummy, offside,
            declarer_trajectories: Vec::new(),
            whist_trajectories: Vec::new(),
            offside_trajectories: Vec::new(),
            test_agent_declarer,
            test_agent_whist,
            test_agent_offside,
            declarer_rewards: Vec::new(),
            whist_rewards: Vec::new(),
            offside_rewards: Vec::new(),

            env_comm_test_declarer,
            env_comm_test_whist,
            env_comm_test_offside,
        }
    }

    fn stash_trajectories(&mut self){
        self.declarer_trajectories.push(self.declarer.0.take_trajectory());
        self.whist_trajectories.push(self.whist.0.take_trajectory());
        self.offside_trajectories.push(self.offside.0.take_trajectory());
    }
    fn discard_last_trajectory(&mut self){
        self.declarer.0.take_trajectory();
        self.whist.0.take_trajectory();
        self.offside.0.take_trajectory();
    }

    fn swap_agents(&mut self, team: &Team){
        match team{
            Team::Contractors => {
                self.declarer.0.swap_comms_with_basic(&mut self.test_agent_declarer.0);

            }
            Team::Defenders => {
                self.whist.0.swap_comms_with_basic(&mut self.test_agent_whist.0);
                self.offside.0.swap_comms_with_basic(&mut self.test_agent_offside.0);
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
        self.stash_trajectories();
        self.environment.replace_state(ContractEnvStateComplete::construct_from((&contract, &deal_description)));
        self.declarer.0.reset(S::construct_from((&contract.declarer(), &contract, &deal_description)));
        self.whist.0.reset(S::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
        self.dummy.reset(ContractDummyState::construct_from((&contract.declarer().next_i(2), &contract, &deal_description)));
        self.offside.0.reset(S::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));

    }

    fn prepare_test_game(&mut self, rng: &mut ThreadRng, distribution: &DealDistribution, contract_randomizer: &ContractRandomizer, tested_team: &Team){
        let deal = distribution.sample(rng);
        let deal_description = DescriptionDeckDeal{
            probabilities: distribution.clone(),
            cards: deal,
        };
        let contract = contract_randomizer.sample(rng);
        self.discard_last_trajectory();
        match tested_team{
            Team::Contractors => {
                self.declarer.0.reset(S::construct_from((&contract.declarer(), &contract, &deal_description)));
                self.dummy.reset(ContractDummyState::construct_from((&contract.declarer().next_i(2), &contract, &deal_description)));
                self.test_agent_whist.0.reset(S::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
                self.test_agent_offside.0.reset(S::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)))
            }
            Team::Defenders => {
                self.test_agent_declarer.0.reset(S::construct_from((&contract.declarer(), &contract, &deal_description)));
                self.dummy.reset(ContractDummyState::construct_from((&contract.declarer().next_i(2), &contract, &deal_description)));
                self.whist.0.reset(S::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
                self.offside.0.reset(S::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));
            }
        }
    }

    fn clear_trajectories(&mut self){
        self.declarer.0.take_trajectory();
        self.whist.0.take_trajectory();
        self.offside.0.take_trajectory();
        self.offside_trajectories.clear();
        self.whist_trajectories.clear();
        self.declarer_trajectories.clear();
    }

    fn play_game(&mut self) -> Result<(), SztormError<ContractDP>>{
        thread::scope(|s|{
            s.spawn(||{
                self.environment.run_round_robin_uni_rewards().unwrap();
            });
            s.spawn(||{
                self.declarer.0.run().unwrap();
            });

            s.spawn(||{
                self.whist.0.run().unwrap();
            });

            s.spawn(||{
                self.dummy.run().unwrap();
            });

            s.spawn(||{
                self.offside.0.run().unwrap();
            });
        });
        Ok(())
    }

    fn play_test_game(&mut self, team: &Team) -> Result<(), SztormError<ContractDP>>{
        thread::scope(|s|{
            s.spawn(||{
                self.environment.run_round_robin_uni_rewards().unwrap();
            });

            s.spawn(||{
                self.dummy.run().unwrap();
            });

            match team{
                Team::Contractors => {
                    s.spawn(||{
                        self.declarer.0.run().unwrap();
                    });
                    s.spawn(||{
                        self.test_agent_whist.0.run().unwrap();
                    });
                    s.spawn(||{
                        self.test_agent_offside.0.run().unwrap();
                    });

                }
                Team::Defenders => {
                    s.spawn(||{
                        self.whist.0.run().unwrap();
                    });
                    s.spawn(||{
                        self.test_agent_declarer.0.run().unwrap();
                    });
                    s.spawn(||{
                        self.offside.0.run().unwrap();
                    });
                }
            }
        });
        Ok(())
    }

    fn store_single_game_results_in_test(&mut self, team: &Team){
        match team{
            Team::Contractors => {
                self.declarer_rewards.push(self.declarer.0.current_universal_score());
                self.whist_rewards.push(self.test_agent_whist.0.current_universal_score());
                self.offside_rewards.push(self.test_agent_offside.0.current_universal_score());
            }
            Team::Defenders => {
                self.declarer_rewards.push(self.test_agent_declarer.0.current_universal_score());
                self.whist_rewards.push(self.whist.0.current_universal_score());
                self.offside_rewards.push(self.offside.0.current_universal_score());
            }
        }
    }
    //fn reset_game_pool(&mut self, rng: &mut ThreadRng, distribution_pool:  )

    pub fn train_agents_one_epoch(&mut self, games_in_epoch: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
        ) -> Result<(), SztormError<ContractDP>>{

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
        }
        Ok(())

    }

    pub fn test_agents(&mut self, team: &Team, number_of_tests: usize,
            distribution_pool: Option<&[DealDistribution]>,
            contract_randomizer: &ContractRandomizer)
        -> Result<f64, SztormError<ContractDP>>{
        let mut rng = thread_rng();

        self.declarer_rewards.clear();
        self.whist_rewards.clear();
        self.offside_rewards.clear();

        //
        self.swap_agents(&team.opposite());

        for _ in 0..number_of_tests{
            let distr = if let Some(pool) = distribution_pool{
                pool.choose(&mut rng).unwrap_or(&DealDistribution::Fair)

            } else {
                &DealDistribution::Fair
            };
            self.prepare_test_game(&mut rng, distr, contract_randomizer, team);
            self.play_test_game(team).unwrap();
            self.store_single_game_results_in_test(team);

        }

        let average_declarer: f64 = self.declarer_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.declarer_rewards.len() as f64;
        let average_whist: f64 = self.whist_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.whist_rewards.len() as f64;
        let average_offside: f64 = self.offside_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.offside_rewards.len() as f64;

        self.swap_agents(&team.opposite());

        let (average_tested, average_enemy) = match team{
            Team::Contractors => (average_declarer, average_whist + average_offside / 2.0),
            Team::Defenders => (average_whist + average_offside / 2.0, average_declarer)
        };
        info!("Average score  for team {team:?}: {average_tested:}. While opposite: {average_enemy:}");
        Ok(average_tested)


    }

    pub fn train_agents(&mut self,
                        epochs: usize,
                        games_in_epoch: usize,

                        distribution_pool: Option<&[DealDistribution]>){

    }
}

pub struct DynamicContractA2CSessionBuilder<ISW2T: WayToTensor, S: ContractInfoSetTraitJoined<ISW2T> + Clone>{
    environment: Option<ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>>,
    declarer: Option<ContractA2CAgentLocalGen<ISW2T, S>>,
    whist: Option<ContractA2CAgentLocalGen<ISW2T, S>>,
    dummy: Option<AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>>,
    offside: Option<ContractA2CAgentLocalGen<ISW2T, S>>,
    env_declarer_comm: Option<ContractEnvSyncComm>,
    env_whist_comm: Option<ContractEnvSyncComm>,
    env_offside_comm: Option<ContractEnvSyncComm>,
    env_dummy_comm: Option<ContractEnvSyncComm>,
    env_declarer_hold: Option<ContractEnvSyncComm>,
    env_whist_hold: Option<ContractEnvSyncComm>,
    env_offside_hold: Option<ContractEnvSyncComm>,





}

impl <ISW2T: WayToTensor, S: ContractInfoSetTraitJoined<ISW2T> + Clone> DynamicContractA2CSessionBuilder<ISW2T, S>{

    //pub fn register_dummy

}


 */

//impl<ISW2T: WayToTensor, S: ContractInfoSetTraitJoined<ISW2T> + Clone>