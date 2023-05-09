use std::cmp::{max, min};
use std::path::PathBuf;
use std::thread;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand_distr::Geometric;
use tch::Device;
use tch::nn::VarStore;
use brydz_core::bidding::{Bid, Doubling};
use brydz_core::cards::trump::TrumpGen;
use brydz_core::contract::{Contract, ContractMechanics, ContractParameters, ContractParametersGen};
use brydz_core::deal::fair_bridge_deal;
use brydz_core::meta::HAND_SIZE;
use brydz_core::player::side::{Side, SideMap};
use brydz_core::player::side::Side::*;
use brydz_core::sztorm::agent::ContractAgent;
use brydz_core::sztorm::comm::{ContractAgentSyncComm, ContractEnvSyncComm};
use brydz_core::sztorm::env::{ContractEnv, ContractProcessor};
use brydz_core::sztorm::spec::ContractProtocolSpec;
use brydz_core::sztorm::state::{ContractAgentInfoSetSimple, ContractDummyState, ContractEnvStateMin};
use karty::hand::CardSet;
use karty::random::RandomSymbol;
use karty::suits::Suit;
use karty::suits::Suit::Spades;
use sztorm::{DistinctAgent, EnvCommEndpoint, PolicyAgent, RandomPolicy, SingleQPolicyGen};
use sztorm::automatons::rr::{AgentAuto, EnvironmentRR, RoundRobinModel};
use crate::{ContractQNetSimple, EEPolicy};
use crate::error::BrydzSimError;
use crate::options::operation::TrainOptions;
use rand_distr::Distribution;

fn load_var_store(path: Option<&PathBuf>) -> Result<VarStore, BrydzSimError>{
    Ok(match path{
        None => VarStore::new(Device::cuda_if_available()),
        Some(path) => {
            let mut vs = VarStore::new(Device::cuda_if_available());
            vs.load(path)?;
            vs
        }
    })
}

type SimpleQnetAgent = ContractAgent<ContractAgentInfoSetSimple, ContractAgentSyncComm, EEPolicy<ContractQNetSimple>>;
type DummyAgent  = ContractAgent<ContractDummyState, ContractAgentSyncComm, RandomPolicy<ContractProtocolSpec, ContractDummyState>>;
type SimpleEnv = ContractEnv<ContractEnvStateMin, ContractEnvSyncComm>;

pub fn train_on_single_game(ready_env: &mut SimpleEnv,
                            ready_declarer: &mut SimpleQnetAgent,
                            ready_whist: &mut SimpleQnetAgent,
                            ready_offside: &mut SimpleQnetAgent,
                            ready_dummy: &mut DummyAgent, rng: &mut ThreadRng, geo: &mut Geometric) -> Result<(), BrydzSimError>{


    let step_start_explore = min(geo.sample(rng), HAND_SIZE as u64);

    //ready_declarer.policy_mut().set_exploiting_start(step_start_explore*2);
    &mut ready_declarer.policy_mut().set_exploiting_start(step_start_explore*2);

    ready_whist.policy_mut().set_exploiting_start(step_start_explore);
    ready_offside.policy_mut().set_exploiting_start(step_start_explore);

    thread::scope(|s|{
        s.spawn(||{
            ready_env.env_run_rr().unwrap();
        });
        s.spawn(||{
            ready_declarer.run_rr().unwrap();
        });

        s.spawn(||{
            ready_whist.run_rr().unwrap();
        });

        s.spawn(||{
            ready_offside.run_rr().unwrap();
        });

        s.spawn(||{
            ready_dummy.run_rr().unwrap();
        });
    });

    println!("{:?}", ready_declarer.trace().iter().map(|(s,a,r)|(r)).collect::<Vec<_>>());
    println!("{:?}", ready_whist.trace().iter().map(|(s,a,r)|(r)).collect::<Vec<_>>());
    println!("{:?}", ready_offside.trace().iter().map(|(s,a,r)|(r)).collect::<Vec<_>>());

    println!("{:?}", ready_declarer.policy().get_step_counter());
    println!("{:?}", ready_declarer.policy().exploitation_start());
    todo!();
    Ok(())
}

fn random_contract_params(declarer: Side, rng: &mut ThreadRng) -> ContractParameters{
    let contract_value = rng.gen_range(1..=7);
    let trump = TrumpGen::<Suit>::random(rng);
    let doubling = *[Doubling::None, Doubling::Redouble, Doubling::Redouble].choose(rng).unwrap();

    ContractParameters::new_d(declarer, Bid::init(trump, contract_value).unwrap(), doubling)
}

fn renew_world(contract_params: ContractParameters, cards: SideMap<CardSet>,
               env: &mut SimpleEnv,
               declarer: &mut SimpleQnetAgent, whist: &mut SimpleQnetAgent, offside: &mut SimpleQnetAgent,
               dummy: &mut DummyAgent) -> Result<(), BrydzSimError>{
    let contract = Contract::new(contract_params);
    let dummy_side = contract.dummy();
    env.replace_state(ContractEnvStateMin::new(contract.clone(), None));
    declarer.replace_state(ContractAgentInfoSetSimple::new(*declarer.id(), cards[declarer.id()], contract.clone(), None));
    whist.replace_state(ContractAgentInfoSetSimple::new(*whist.id(), cards[whist.id()], contract.clone(), None));
    offside.replace_state(ContractAgentInfoSetSimple::new(*offside.id(), cards[offside.id()], contract.clone(), None));
    dummy.replace_state(ContractDummyState::new(dummy_side, cards[&dummy_side], contract));


    Ok(())

}

pub fn train_session(train_options: &TrainOptions) -> Result<(), BrydzSimError>{
    let mut rng = thread_rng();
    let mut geo = Geometric::new(0.25).unwrap();

    let policy_declarer = EEPolicy::new(ContractQNetSimple::new(load_var_store(train_options.declarer_load.as_ref())?));
    let policy_whist = EEPolicy::new(ContractQNetSimple::new(load_var_store(train_options.whist_load.as_ref())?));
    let policy_offside = EEPolicy::new(ContractQNetSimple::new(load_var_store(train_options.offside_load.as_ref())?));
    let policy_dummy = RandomPolicy::<ContractProtocolSpec, ContractDummyState>::new();



    let contract = Contract::new(random_contract_params(North, &mut rng));

    let (comm_env_north, comm_north) = ContractEnvSyncComm::new_pair();
    let (comm_env_east, comm_east) = ContractEnvSyncComm::new_pair();
    let (comm_env_west, comm_west) = ContractEnvSyncComm::new_pair();
    let (comm_env_south, comm_south) = ContractEnvSyncComm::new_pair();
    let comm_association = SideMap::new(comm_env_north, comm_env_east, comm_env_south, comm_env_west);

    let card_deal = fair_bridge_deal::<CardSet>();
    let initial_state_declarer = ContractAgentInfoSetSimple::new(North, card_deal[&North], contract.clone(), None);
    let initial_state_whist = ContractAgentInfoSetSimple::new(East, card_deal[&East], contract.clone(), None);
    let initial_state_offside = ContractAgentInfoSetSimple::new(West, card_deal[&West], contract.clone(), None);
    let initial_state_dummy = ContractDummyState::new(South, card_deal[&South], contract.clone());
    let env_state = ContractEnvStateMin::new(contract, None);

    let mut declarer = SimpleQnetAgent::new(initial_state_declarer, comm_north, policy_declarer);
    let mut whist = SimpleQnetAgent::new(initial_state_whist, comm_east, policy_whist);
    let mut offside = SimpleQnetAgent::new(initial_state_offside, comm_west, policy_offside);
    let mut dummy = DummyAgent::new(initial_state_dummy, comm_south, policy_dummy);

    let mut env = SimpleEnv::new(env_state, comm_association);

    for e in 0..train_options.epochs{
        train_on_single_game( &mut env, &mut declarer, &mut whist, &mut offside, &mut dummy, &mut rng, &mut geo)?;
        let contract_params = random_contract_params(North, &mut rng);
        renew_world(contract_params, fair_bridge_deal(), &mut env, &mut declarer, &mut whist, &mut offside, &mut dummy)?;


    }

    Ok(())
}