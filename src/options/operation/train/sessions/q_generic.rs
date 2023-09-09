use std::fmt::Debug;
use std::thread;
use log::{debug, info};
use rand::prelude::{Distribution, SliceRandom};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use tch::{Device, nn, Tensor};
use tch::nn::{Adam, Optimizer, VarStore};
use brydz_core::contract::{ContractMechanics, ContractParameters, ContractRandomizer};
use brydz_core::deal::{DealDistribution, DescriptionDeckDeal};
use brydz_core::error::BridgeCoreErrorGen::Contract;
use brydz_core::player::side::{Side, SideMap};
use brydz_core::sztorm::comm::{ContractAgentSyncComm, ContractEnvSyncComm};
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractActionWayToTensor, ContractAgentInfoSetAllKnowing, ContractAgentInfoSetSimple, ContractDummyState, ContractEnvStateComplete, ContractInfoSetConvert420Normalised, ContractState};
use karty::suits::Suit;
use sztorm::agent::{Agent, AgentGen, AgentGenT, AgentTrajectory, AutomaticAgent, AutomaticAgentRewarded, EnvRewardedAgent, Policy, PolicyAgent, RandomPolicy, ResetAgent, TracingAgent};
use sztorm::env::{RoundRobinPenalisingUniversalEnvironment, RoundRobinUniversalEnvironment, StatefulEnvironment};
use sztorm::error::SztormError;
use sztorm::protocol::DomainParameters;
use sztorm::state::agent::ScoringInformationSet;
use sztorm::state::ConstructedState;
use sztorm_rl::actor_critic::ActorCriticPolicy;
use sztorm_rl::q_learning_policy::{QLearningPolicy, QSelector};
use sztorm_rl::tensor_repr::{ConvertToTensor, WayToTensor};
use sztorm_rl::torch_net::{A2CNet, NeuralNetCloner, QValueNet, TensorA2C};
use crate::options::operation::sessions::{ContractInfoSetForLearning, SessionAgentTrait, Team};
use crate::options::operation::TrainOptions;


pub type ContractQPolicyLocalAgent<ISW, S> = AgentGenT<
    ContractDP,
    QLearningPolicy<
        ContractDP,
        S,
        ISW, ContractActionWayToTensor>,
    ContractAgentSyncComm>;



pub struct TestAgents<P: Policy<ContractDP>>
where P: Policy<ContractDP, StateType = ContractAgentInfoSetAllKnowing>{
    pub declarer: AgentGen<ContractDP, P, ContractAgentSyncComm>,
    pub whist: AgentGen<ContractDP, P, ContractAgentSyncComm>,
    pub offside: AgentGen<ContractDP, P, ContractAgentSyncComm>,
}


pub struct GenericContractQLearningSession<
    DISW2T: WayToTensor,
    WISW2T: WayToTensor,
    OISW2T: WayToTensor,
    DIS: ContractInfoSetForLearning<DISW2T> + Clone,
    WIS: ContractInfoSetForLearning<WISW2T> + Clone,
    OIS: ContractInfoSetForLearning<OISW2T> + Clone,
    //TP: Policy<ContractDP, StateType= ContractAgentInfoSetAllKnowing>,
>{
    environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
    declarer: ContractQPolicyLocalAgent<DISW2T, DIS>,
    whist: ContractQPolicyLocalAgent<WISW2T, WIS>,
    offside: ContractQPolicyLocalAgent<OISW2T, OIS>,
    dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,

    declarer_trajectories: Vec<AgentTrajectory<ContractDP, DIS>>,
    whist_trajectories: Vec<AgentTrajectory<ContractDP, WIS>>,
    offside_trajectories: Vec<AgentTrajectory<ContractDP, OIS>>,
    declarer_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    whist_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,
    offside_rewards: Vec<<ContractDP as DomainParameters>::UniversalReward>,



    //test_declarer: AgentGen<ContractDP, TP, ContractAgentSyncComm>,
    //test_whist: AgentGen<ContractDP, TP, ContractAgentSyncComm>,
    //test_offside: AgentGen<ContractDP, TP, ContractAgentSyncComm>,
}



impl<
    DISW2T: WayToTensor,
    WISW2T: WayToTensor,
    OISW2T: WayToTensor,
    DIS: ContractInfoSetForLearning<DISW2T> + Clone,
    WIS: ContractInfoSetForLearning<WISW2T> + Clone,
    OIS: ContractInfoSetForLearning<OISW2T> + Clone,
    //TP: Policy<ContractDP, StateType= ContractAgentInfoSetAllKnowing> + Clone,
> GenericContractQLearningSession<
    DISW2T,
    WISW2T,
    OISW2T,
    DIS,
    WIS,
    OIS,
    //TP
    >{

    pub fn new_rand_init(
        declarer_policy: QLearningPolicy<ContractDP, DIS, DISW2T, ContractActionWayToTensor>,
        whist_policy: QLearningPolicy<ContractDP, WIS, WISW2T, ContractActionWayToTensor>,
        offside_policy: QLearningPolicy<ContractDP, OIS, OISW2T, ContractActionWayToTensor>,
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


        let declarer = ContractQPolicyLocalAgent::new(
            contract_params.declarer(),
            DIS::construct_from((&contract_params.declarer(), &contract_params, &deal_description)),
            comm_decl_env, declarer_policy);

        let whist = ContractQPolicyLocalAgent::new(
            contract_params.declarer().next_i(1),
            WIS::construct_from((&contract_params.declarer().next_i(1), &contract_params, &deal_description)),
            comm_whist_env, whist_policy);

        let offside = ContractQPolicyLocalAgent::new(
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

        //let test_declarer = AgentGen::new(contract_params.declarer(), (), (), ())




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

    fn clear_trajectories(&mut self){
        self.declarer.take_trajectory();
        self.whist.take_trajectory();
        self.offside.take_trajectory();
        self.offside_trajectories.clear();
        self.whist_trajectories.clear();
        self.declarer_trajectories.clear();
    }

    fn store_single_game_results_in_test<P: Policy<ContractDP>>(&mut self, team: &Team, test_agents: &TestAgents<P>)
    where P: Policy<ContractDP, StateType = ContractAgentInfoSetAllKnowing>{
        match team{
            Team::Contractors => {
                self.declarer_rewards.push(self.declarer.current_universal_score());
                self.whist_rewards.push(test_agents.whist.current_universal_score());
                self.offside_rewards.push(test_agents.offside.current_universal_score());
            }
            Team::Defenders => {
                self.declarer_rewards.push(test_agents.declarer.current_universal_score());
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
        self.declarer.reset(DIS::construct_from((&contract.declarer(), &contract, &deal_description)));
        self.whist.reset(WIS::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
        self.dummy.reset(ContractDummyState::construct_from((&contract.declarer().next_i(2), &contract, &deal_description)));
        self.offside.reset(OIS::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));

        self.declarer.change_id(contract.declarer());
        self.whist.change_id(contract.whist());
        self.dummy.change_id(contract.dummy());
        self.offside.change_id(contract.offside());
        self.environment.comms_mut().rotate(old_declarer_side, contract.declarer());

        debug!("Preparing game, trump: {}", &contract.bid().trump());
        debug!("Preparing game, declarer's side: {}", &contract.declarer());
    }

    fn prepare_test_game<P: Policy<ContractDP>>
    (
        &mut self,
        rng: &mut ThreadRng,
        distribution: &DealDistribution,
        contract_randomizer: &ContractRandomizer,
        test_agents: &mut TestAgents<P>,
        tested_team: Team)
    where P: Policy<ContractDP, StateType = ContractAgentInfoSetAllKnowing>{

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
                self.declarer.reset(DIS::construct_from((&contract.declarer(), &contract, &deal_description)));
                test_agents.whist.reset(ContractAgentInfoSetAllKnowing::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
                test_agents.offside.reset(ContractAgentInfoSetAllKnowing::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));
            }
            Team::Defenders => {
                test_agents.declarer.reset(ContractAgentInfoSetAllKnowing::construct_from((&contract.declarer(), &contract, &deal_description)));
                self.whist.reset(WIS::construct_from((&contract.declarer().next_i(1), &contract, &deal_description)));
                self.offside.reset(OIS::construct_from((&contract.declarer().next_i(3), &contract, &deal_description)));
                debug!("Whist's , commited score: {}", self.whist.current_universal_score());
            }
        }

        self.declarer.change_id(contract.declarer());
        self.dummy.change_id(contract.dummy());
        self.offside.change_id(contract.offside());
        self.whist.change_id(contract.whist());
        self.environment.comms_mut().rotate(old_declarer_side, contract.declarer());
        test_agents.whist.change_id(contract.whist());
        test_agents.offside.change_id(contract.offside());
        test_agents.declarer.change_id(contract.declarer());

        debug!("Preparing game, trump: {}", &contract.bid().trump());
        debug!("Preparing game, declarer's side: {}", &contract.declarer());
        debug!("Declarer ({}) cards: {:#}", &contract.declarer(), deal_description.cards[&contract.declarer()]);
        debug!("Whist ({}) cards: {:#}", &contract.whist(), deal_description.cards[&contract.whist()]);
        debug!("Dummy ({}) cards: {:#}", &contract.dummy(), deal_description.cards[&contract.dummy()]);
        debug!("Offside ({}) cards: {:#}", &contract.offside(), deal_description.cards[&contract.offside()]);


    }

    fn play_game(&mut self) -> Result<(), SztormError<ContractDP>>{
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

    fn play_test_game<P: Policy<ContractDP>>
    (&mut self, team: &Team, test_agents: &mut TestAgents<P>) -> Result<(), SztormError<ContractDP>>
    where P: Policy<ContractDP, StateType = ContractAgentInfoSetAllKnowing>{
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
                        test_agents.whist.run_rewarded();
                    });
                    s.spawn(||{
                        test_agents.offside.run_rewarded();
                    });

                }
                Team::Defenders => {
                    s.spawn(||{
                        self.whist.run_rewarded();
                    });
                    s.spawn(||{
                        test_agents.declarer.run_rewarded();
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

    fn stash_result<P: Policy<ContractDP>>(&mut self, team: &Team, test_agents: &TestAgents<P>)
    where P: Policy<ContractDP, StateType = ContractAgentInfoSetAllKnowing>{
        match team{
            Team::Contractors => {
                self.declarer_rewards.push(self.declarer.current_universal_score());
                self.whist_rewards.push(test_agents.whist.current_universal_score());
                self.offside_rewards.push(test_agents.offside.current_universal_score());
            }
            Team::Defenders => {
                self.declarer_rewards.push(test_agents.declarer.current_universal_score());
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
            self.stash_trajectories();



        }
        debug!("Declarer batch input sizes: {:?}", self.declarer_trajectories.iter().map(|v|v.list().len()).collect::<Vec<usize>>());
        debug!("Whist batch input sizes: {:?}", self.whist_trajectories.iter().map(|v|v.list().len()).collect::<Vec<usize>>());
        debug!("Offside batch input sizes: {:?}", self.offside_trajectories.iter().map(|v|v.list().len()).collect::<Vec<usize>>());

        if !self.declarer_trajectories.is_empty(){
            self.declarer.policy_mut().batch_train_env_rewards(&self.declarer_trajectories[..], 0.99)?;
        }
        if !self.whist_trajectories.is_empty(){
            self.whist.policy_mut().batch_train_env_rewards(&self.whist_trajectories[..], 0.99)?;
        }
        if !self.offside_trajectories.is_empty(){
            self.offside.policy_mut().batch_train_env_rewards(&self.offside_trajectories[..], 0.99)?;
        }




        Ok(())

    }

    pub fn test_agents_team<P: Policy<ContractDP>>(&mut self, rng: &mut ThreadRng, team: &Team, number_of_tests: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
        test_agents: &mut TestAgents<P>)
        -> Result<f64, SztormError<ContractDP>>
    where P: Policy<ContractDP, StateType = ContractAgentInfoSetAllKnowing>{


        self.clear_rewards();

        match team{
            Team::Contractors => {
                self.whist.swap_comms_with_basic(&mut test_agents.whist);
                self.offside.swap_comms_with_basic(&mut test_agents.offside);
                for _ in 0..number_of_tests {
                    let distr = if let Some(pool) = distribution_pool {
                        pool.choose(rng).unwrap_or(&DealDistribution::Fair)
                    } else {
                        &DealDistribution::Fair
                    };
                    self.prepare_test_game(rng, distr, contract_randomizer, test_agents, Team::Contractors);
                    let _ = self.play_test_game(team, test_agents);
                    self.stash_result(team, test_agents);

                }
                self.whist.swap_comms_with_basic(&mut test_agents.whist);
                self.offside.swap_comms_with_basic(&mut test_agents.offside);

                debug!("Declarer's rewards: {:?}", self.declarer_rewards);
                let average = self.declarer_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.declarer_rewards.len() as f64;
                info!("Testing declarer. Declarer's average reward: {average:}");
                Ok(average)
            }
            Team::Defenders => {
                self.declarer.swap_comms_with_basic(&mut test_agents.declarer);
                for _ in 0..number_of_tests {
                    let distr = if let Some(pool) = distribution_pool {
                        pool.choose(rng).unwrap_or(&DealDistribution::Fair)
                    } else {
                        &DealDistribution::Fair
                    };
                    self.prepare_test_game(rng, distr, contract_randomizer, test_agents, Team::Defenders);
                    let _ = self.play_test_game(team, test_agents);
                    self.stash_result(team, test_agents);

                }
                self.declarer.swap_comms_with_basic(&mut test_agents.declarer);

                debug!("Whist's rewards: {:?}, offside's rewards {:?}", self.whist_rewards, self.offside_rewards);
                let average_w = self.whist_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.whist_rewards.len() as f64;
                let average_o = self.offside_rewards.iter().map(|i| *i as f64).sum::<f64>() / self.offside_rewards.len() as f64;
                info!("Testing defender's. Whist's average reward: {average_w:}. Offside's average reward: {average_o:}");
                Ok((average_w + average_o) / 2.0)


            }
        }


    }

    pub fn test_agents<P: Policy<ContractDP>>(&mut self, number_of_tests: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
        tester_policy: P)
        -> Result<(f64, f64), SztormError<ContractDP>>
    where P: Policy<ContractDP, StateType = ContractAgentInfoSetAllKnowing> + Clone{

        let (_, test_decl_comm) = ContractEnvSyncComm::new_pair();
        let (_, test_whist_comm) = ContractEnvSyncComm::new_pair();
        let (_, test_offside_comm) = ContractEnvSyncComm::new_pair();

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

        let mut test_agents = TestAgents{
            declarer: AgentGen::new(
                self.declarer.id(),
                ContractAgentInfoSetAllKnowing::construct_from((&self.declarer.id(), &contract, &deal_description)),
                test_decl_comm, tester_policy.clone()),
            whist: AgentGen::new(
                self.whist.id(),
                ContractAgentInfoSetAllKnowing::construct_from((&self.whist.id(), &contract, &deal_description)),
                test_whist_comm, tester_policy.clone()),

            offside: AgentGen::new(
                self.offside.id(),
                ContractAgentInfoSetAllKnowing::construct_from((&self.offside.id(), &contract, &deal_description)),
                test_offside_comm, tester_policy),

        };

        let declarer_score = self.test_agents_team(
            &mut rng,
            &Team::Contractors,
            number_of_tests,
            distribution_pool,
            contract_randomizer,
            &mut test_agents)?;



        let defender_score = self.test_agents_team(
            &mut rng,
            &Team::Defenders,
            number_of_tests,
            distribution_pool,
            contract_randomizer,
            &mut test_agents)?;


        Ok((declarer_score, defender_score))

    }

    pub fn train<P: Policy<ContractDP>>(
        &mut self,
        epochs: usize,
        games_in_epoch: usize,
        games_in_test: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
        tester_policy: P
    ) -> Result<(), SztormError<ContractDP>>
    where P: Policy<ContractDP, StateType = ContractAgentInfoSetAllKnowing> + Clone{

        println!("Przed testem początkowym");
        self.test_agents(games_in_test, distribution_pool, contract_randomizer, tester_policy.clone())?;
        println!("Po teście początkowym");
        for e in 1..=epochs{
            self.train_agents_separately_one_epoch(games_in_epoch, distribution_pool, contract_randomizer)?;
            //self.train_agents_singe_store_one_epoch(games_in_epoch, distribution_pool, contract_randomizer)?;
            info!("Completed epoch {e:} of training.");
            let _test_results = self.test_agents(games_in_test, distribution_pool, contract_randomizer, tester_policy.clone())?;
        }
        Ok(())
    }
}

impl<
    W2T: WayToTensor,
    IS: ContractInfoSetForLearning<W2T> + Clone,
    //TP: Policy<ContractDP, StateType= ContractAgentInfoSetAllKnowing> + Clone,
> GenericContractQLearningSession<
    W2T, W2T, W2T,
    IS,IS, IS
    >{

    fn stash_trajectories_all_to_declarer(&mut self){
        let declarer_trajectory = self.declarer.take_trajectory();
        if !declarer_trajectory.is_empty(){
            self.declarer_trajectories.push(declarer_trajectory);
        }
        let whist_trajectory = self.whist.take_trajectory();
        if !whist_trajectory.is_empty(){
            self.declarer_trajectories.push(whist_trajectory);
        }
        let offside_trajectory = self.offside.take_trajectory();
        if !offside_trajectory.is_empty(){
            self.declarer_trajectories.push(offside_trajectory);
        }
    }

    pub fn train_agents_single_store_one_epoch(
        &mut self,
        games_in_epoch: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
    ) -> Result<(), SztormError<ContractDP>> {
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
            self.declarer.policy_mut().batch_train_env_rewards(&self.declarer_trajectories[..], 0.99)?;
        }
        self.whist.policy_mut().var_store_mut().copy(self.declarer.policy().var_store()).unwrap();
        self.offside.policy_mut().var_store_mut().copy(self.declarer.policy().var_store()).unwrap();
        Ok(())
    }

    pub fn train_all_at_once<P: Policy<ContractDP>>(
        &mut self,
        epochs: usize,
        games_in_epoch: usize,
        games_in_test: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
        tester_policy: P
    ) -> Result<(), SztormError<ContractDP>>
    where P: Policy<ContractDP, StateType = ContractAgentInfoSetAllKnowing> + Clone{

        println!("Przed testem początkowym");
        self.test_agents(games_in_test, distribution_pool, contract_randomizer, tester_policy.clone())?;
        println!("Po teście początkowym");
        for e in 1..=epochs{
            self.train_agents_single_store_one_epoch(games_in_epoch, distribution_pool, contract_randomizer)?;
            //self.train_agents_singe_store_one_epoch(games_in_epoch, distribution_pool, contract_randomizer)?;
            info!("Completed epoch {e:} of training.");
            let _test_results = self.test_agents(games_in_test, distribution_pool, contract_randomizer, tester_policy.clone())?;
        }
        Ok(())
    }




}

pub fn train_session_q(options: &TrainOptions) -> Result<(), SztormError<ContractDP>>{
    let network_pattern  = NeuralNetCloner::new(|path|{
        let seq = nn::seq()
            .add(nn::linear(path / "input", 422, 2048, Default::default()))
            .add(nn::linear(path / "h1", 2048, 2048, Default::default()))
            .add(nn::linear(path / "h2", 2048, 1024, Default::default()))
            //.add(nn::linear(path / "h3", 1024, 512, Default::default()))
        ;
        //let actor = nn::linear(path / "al", 1024, 52, Default::default());
        //let critic = nn::linear(path / "cl", 1024, 1, Default::default());
        let q_layer = nn::linear(path /  "q", 1024, 1, Default::default());
        let device = path.device();

        {move |xs: &Tensor|{
            let xs = xs.to_device(device).apply(&seq);
            //(xs.apply(&critic), xs.apply(&actor))
            xs.apply(&q_layer)
        }}
    });
    let declarer_net = QValueNet::new(VarStore::new(Device::Cpu), network_pattern.get_net_closure());
    let whist_net = QValueNet::new(VarStore::new(Device::Cpu), network_pattern.get_net_closure());
    let offside_net = QValueNet::new(VarStore::new(Device::Cpu), network_pattern.get_net_closure());
    let declarer_optimiser = declarer_net.build_optimizer(Adam::default(), 5e-5).unwrap();
    let whist_optimiser = whist_net.build_optimizer(Adam::default(), 5e-5).unwrap();
    let offside_optimiser = offside_net.build_optimizer(Adam::default(), 5e-5).unwrap();

    let declarer_policy: QLearningPolicy<ContractDP, ContractAgentInfoSetSimple, ContractInfoSetConvert420Normalised, ContractActionWayToTensor>  =
        QLearningPolicy::new(declarer_net, declarer_optimiser, ContractInfoSetConvert420Normalised {}, ContractActionWayToTensor {}, QSelector::MultinomialLogits);
    let whist_policy: QLearningPolicy<ContractDP, ContractAgentInfoSetSimple, ContractInfoSetConvert420Normalised, ContractActionWayToTensor>  =
        QLearningPolicy::new(whist_net, whist_optimiser, ContractInfoSetConvert420Normalised {}, ContractActionWayToTensor {}, QSelector::MultinomialLogits);
    let offside_policy: QLearningPolicy<ContractDP, ContractAgentInfoSetSimple, ContractInfoSetConvert420Normalised, ContractActionWayToTensor>  =
        QLearningPolicy::new(offside_net, offside_optimiser, ContractInfoSetConvert420Normalised {}, ContractActionWayToTensor {}, QSelector::MultinomialLogits);

    let mut session = GenericContractQLearningSession::new_rand_init(declarer_policy, whist_policy, offside_policy);
    let test_policy = RandomPolicy::<ContractDP, ContractAgentInfoSetAllKnowing>::new();
    session.train_all_at_once(1000, 64, 100, None, &Default::default(), test_policy).unwrap();
    Ok(())
}