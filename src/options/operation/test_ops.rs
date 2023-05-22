use std::thread::{self};
use log::{info, LevelFilter};
use rand::{Rng, thread_rng};
use rand::distributions::Standard;
use tch::nn::VarStore;
use brydz_core::bidding::Bid;
use brydz_core::cards::trump::TrumpGen;
use brydz_core::contract::{Contract, ContractParametersGen};
use brydz_core::deal::{BiasedDistribution, fair_bridge_deal};
use brydz_core::player::side::{Side, SideMap};
use brydz_core::player::side::Side::*;
use brydz_core::sztorm::agent::ContractAgent;
use brydz_core::sztorm::comm::ContractEnvSyncComm;
use brydz_core::sztorm::env::{ContractEnv, ContractProcessor};
use brydz_core::sztorm::spec::ContractProtocolSpec;
use brydz_core::sztorm::state::{ContractDummyState, ContractAgentInfoSetSimple, ContractEnvStateMin};
use karty::cards::ACE_SPADES;
use karty::hand::{CardSet};
use karty::suits::Suit::{Spades};
use sztorm::automatons::rr::{AgentAuto, EnvironmentRR, RoundRobinModelBuilder};
use sztorm::error::{CommError, SztormError};
use sztorm::protocol::{AgentMessage, EnvMessage};
use sztorm_net_ext::{ComplexComm, ComplexComm2048};
use sztorm_net_ext::tcp::{TcpCommK1, TcpCommK2};
use sztorm::{AgentGen, RandomPolicy};
use crate::ContractQNetSimple;
use crate::options::setup_logger;


pub fn tur_sim(){
    let contract = ContractParametersGen::new(Side::East, Bid::init(TrumpGen::Colored(Spades), 2).unwrap());
    let (comm_env_north, comm_north) = ContractEnvSyncComm::new_pair();
    let (comm_env_east, comm_east) = ContractEnvSyncComm::new_pair();
    let (comm_env_west, comm_west) = ContractEnvSyncComm::new_pair();
    let (comm_env_south, comm_south) = ContractEnvSyncComm::new_pair();

    let comm_association = SideMap::new(comm_env_north, comm_env_east, comm_env_south, comm_env_west);
    let initial_contract = Contract::new(contract);

    let env_initial_state = ContractEnvStateMin::new(initial_contract.clone(), None);
    let mut simple_env = ContractEnv::new(env_initial_state, comm_association);

    let card_deal = fair_bridge_deal::<CardSet>();
    let (hand_north, hand_east, hand_south, hand_west) = card_deal.destruct();

    let initial_state_east = ContractAgentInfoSetSimple::new(East, hand_east, initial_contract.clone(), None);
    let initial_state_south = ContractAgentInfoSetSimple::new(South, hand_south, initial_contract.clone(), None);
    let initial_state_west = ContractDummyState::new(West, hand_west, initial_contract.clone());
    let initial_state_north = ContractAgentInfoSetSimple::new(North, hand_north, initial_contract, None);


    let random_policy = RandomPolicy::<ContractProtocolSpec, ContractAgentInfoSetSimple>::new();
    let policy_dummy = RandomPolicy::<ContractProtocolSpec, ContractDummyState>::new();

    let mut agent_east = AgentGen::new(East, initial_state_east, comm_east, random_policy.clone() );
    let mut agent_south = AgentGen::new(South, initial_state_south, comm_south, random_policy.clone() );
    let mut agent_west = AgentGen::new(West, initial_state_west, comm_west, policy_dummy);
    let mut agent_north = AgentGen::new(North, initial_state_north, comm_north, random_policy );

    thread::scope(|s|{
        s.spawn(||{
            simple_env.env_run_rr().unwrap();
        });
        s.spawn(||{
            agent_east.run_rr().unwrap();
        });

        s.spawn(||{
            agent_south.run_rr().unwrap();
        });

        s.spawn(||{
            agent_west.run_rr().unwrap();
        });

        s.spawn(||{
            agent_north.run_rr().unwrap();
        });
    })

}

pub fn tur_sim_tcp(){
    let contract = ContractParametersGen::new(Side::East, Bid::init(TrumpGen::Colored(Spades), 2).unwrap());
    type TcpCommSim = TcpCommK1<AgentMessage<ContractProtocolSpec>, EnvMessage<ContractProtocolSpec>, CommError<ContractProtocolSpec>>;
    type TcpCommSimEnv = TcpCommK1<EnvMessage<ContractProtocolSpec>, AgentMessage<ContractProtocolSpec>, CommError<ContractProtocolSpec>>;
    /*let contract = ContractSpec::new(Side::East, Bid::init(TrumpGen::Colored(Spades), 2).unwrap());
    let (comm_env_north, comm_north) = TcpCommSim::new_pair();
    let (comm_env_east, comm_east) = TcpCommSim::new_pair();
    let (comm_env_west, comm_west) = TcpCommSim::new_pair();
    let (comm_env_south, comm_south) = TcpCommSim::new_pair();*/
    let initial_contract = Contract::new(contract);

    let tcp_listener = std::net::TcpListener::bind("127.0.0.1:8420").unwrap();
    thread::scope(|s|{
        s.spawn(||{
            let (north_stream, _) = tcp_listener.accept().unwrap();
            info!("North connected");
            let (east_stream, _) = tcp_listener.accept().unwrap();
            info!("East connected");
            let (south_stream, _) = tcp_listener.accept().unwrap();
            info!("South connected");
            let (west_stream, _) = tcp_listener.accept().unwrap();
            info!("West connected");
            let comm_assotiation = SideMap::new(TcpCommSimEnv::new(north_stream), TcpCommSimEnv::new(east_stream), TcpCommSimEnv::new(south_stream), TcpCommSimEnv::new(west_stream));

            let env_initial_state = ContractEnvStateMin::new(initial_contract.clone(),None);
            let mut simple_env = ContractEnv::new(env_initial_state, comm_assotiation);
            simple_env.env_run_rr().unwrap();
        });


        s.spawn(||{
            let stream_north_c = std::net::TcpStream::connect("127.0.0.1:8420").unwrap();
            info!("North connected (client)");
            let stream_east_c = std::net::TcpStream::connect("127.0.0.1:8420").unwrap();
            info!("East connected (client)");
            let stream_south_c = std::net::TcpStream::connect("127.0.0.1:8420").unwrap();
            info!("South connected (client)");
            let stream_west_c = std::net::TcpStream::connect("127.0.0.1:8420").unwrap();
            info!("West connected (client)");

            let comm_north = TcpCommSim::new(stream_north_c);
            let comm_east = TcpCommSim::new(stream_east_c);
            let comm_south = TcpCommSim::new(stream_south_c);
            let comm_west = TcpCommSim::new(stream_west_c);

            let card_deal = fair_bridge_deal::<CardSet>();
            let (hand_north, hand_east, hand_south, hand_west) = card_deal.destruct();

            let initial_state_east = ContractAgentInfoSetSimple::new(East, hand_east, initial_contract.clone(), None);
            let initial_state_south = ContractAgentInfoSetSimple::new(South, hand_south, initial_contract.clone(), None);
            let initial_state_west = ContractDummyState::new(West, hand_west, initial_contract.clone());
            let initial_state_north = ContractAgentInfoSetSimple::new(North, hand_north, initial_contract.clone(), None);


            let random_policy = RandomPolicy::<ContractProtocolSpec, ContractAgentInfoSetSimple>::new();
            let policy_dummy = RandomPolicy::<ContractProtocolSpec, ContractDummyState>::new();

            let mut agent_east = ContractAgent::new(initial_state_east, comm_east, random_policy.clone() );
            let mut agent_south = ContractAgent::new(initial_state_south, comm_south, random_policy.clone() );
            let mut agent_west = ContractAgent::new(initial_state_west, comm_west, policy_dummy);
            let mut agent_north = ContractAgent::new(initial_state_north, comm_north, random_policy );

            thread::scope(|s|{
                s.spawn(||{
                    agent_east.run_rr().unwrap();
                });

                s.spawn(||{
                    agent_south.run_rr().unwrap();
                });

                s.spawn(||{
                    agent_west.run_rr().unwrap();
                });

                s.spawn(||{
                    agent_north.run_rr().unwrap();
                });
            })



        });
    });


}

pub fn test_generic_model() -> Result<(), SztormError<ContractProtocolSpec>>{
    type TcpCommSim = TcpCommK2<AgentMessage<ContractProtocolSpec>, EnvMessage<ContractProtocolSpec>, CommError<ContractProtocolSpec>>;
    type TcpCommSimEnv = TcpCommK2<EnvMessage<ContractProtocolSpec>, AgentMessage<ContractProtocolSpec>, CommError<ContractProtocolSpec>>;
    let contract_params = ContractParametersGen::new(Side::East, Bid::init(TrumpGen::Colored(Spades), 2).unwrap());
    let (comm_env_north, comm_north) = ContractEnvSyncComm::new_pair();
    
    let (comm_env_east, comm_east) = ContractEnvSyncComm::new_pair();
    let (comm_env_west, comm_west) = ContractEnvSyncComm::new_pair();
    let (_comm_env_south, _comm_south) = ContractEnvSyncComm::new_pair();

    let tcp_listener = std::net::TcpListener::bind("127.0.0.1:8420").unwrap();
    let (t, r) = std::sync::mpsc::channel();

    thread::spawn(move ||{
        let (south_stream_env_side, _) = tcp_listener.accept().unwrap();
        t.send(south_stream_env_side).unwrap();
    } );

    let stream_south_agent_side = std::net::TcpStream::connect("127.0.0.1:8420").unwrap();
    let south_stream_env_side = r.recv().unwrap();
    let env_comm_south = ComplexComm::Tcp(TcpCommSimEnv::new(south_stream_env_side)) ;

    let agent_comm_south = ComplexComm::Tcp(TcpCommSim::new(stream_south_agent_side));

/*
    let comm_env_north = ComplexComm2048::StdSync(comm_env_north);
    let comm_env_east = ComplexComm2048::StdSync(comm_env_east);
    let comm_env_south = ComplexComm2048::StdSync(comm_env_south);
    let comm_env_west = ComplexComm2048::StdSync(comm_env_west);

    let comm_north = ComplexComm2048::StdSync(comm_north);
    let comm_east = ComplexComm2048::StdSync(comm_east);
    let comm_south = ComplexComm2048::StdSync(comm_south);
    let comm_west = ComplexComm2048::StdSync(comm_west);
*/
    let card_deal = fair_bridge_deal::<CardSet>();
    let (hand_north, hand_east, hand_south, hand_west) = card_deal.destruct();
    let initial_contract = Contract::new(contract_params);

    let initial_state_east = ContractAgentInfoSetSimple::new(East, hand_east, initial_contract.clone(), None);
    let initial_state_south = ContractAgentInfoSetSimple::new(South, hand_south, initial_contract.clone(), None);
    let initial_state_west = ContractDummyState::new(West, hand_west, initial_contract.clone());
    let initial_state_north = ContractAgentInfoSetSimple::new(North, hand_north, initial_contract.clone(), None);

    let random_policy = RandomPolicy::<ContractProtocolSpec, ContractAgentInfoSetSimple>::new();
    let policy_dummy = RandomPolicy::<ContractProtocolSpec, ContractDummyState>::new();

    let agent_east = AgentGen::new(East, initial_state_east, comm_east, random_policy.clone() );
    let mut agent_south = AgentGen::new(South, initial_state_south, agent_comm_south, random_policy.clone() );
    let agent_west = AgentGen::new(West, initial_state_west, comm_west, policy_dummy);
    let agent_north = AgentGen::new(North, initial_state_north, comm_north, random_policy );


    let mut model = RoundRobinModelBuilder::new()
        .with_env_state(ContractEnvStateMin::new(initial_contract, None))?
        .with_env_action_process_fn(ContractProcessor{})?
        .with_local_agent(Box::new(agent_east), ComplexComm2048::StdSync(comm_env_east))?
        //.with_local_agent(Box::new(agent_south), agent_comm_south)?
        .with_local_agent(Box::new(agent_west), ComplexComm2048::StdSync(comm_env_west))?
        .with_local_agent(Box::new(agent_north), ComplexComm2048::StdSync(comm_env_north))?
        .with_remote_agent(Side::South, env_comm_south)?
        .build()?;



    

    thread::spawn(move || {
        agent_south.run_rr().unwrap();
    });
    model.play().unwrap();

    Ok(())
}

pub fn test_with_untrained_network() -> Result<(), SztormError<ContractProtocolSpec>>{

    let vs_east = VarStore::new(tch::Device::Cpu);

    let policy_east = ContractQNetSimple::new(vs_east, 0.25);

    //type TcpCommSim = TcpCommK2<AgentMessage<ContractProtocolSpec>, EnvMessage<ContractProtocolSpec>, CommError<ContractProtocolSpec>>;
    //type TcpCommSimEnv = TcpCommK2<EnvMessage<ContractProtocolSpec>, AgentMessage<ContractProtocolSpec>, CommError<ContractProtocolSpec>>;
    let contract_params = ContractParametersGen::new(Side::East, Bid::init(TrumpGen::Colored(Spades), 2).unwrap());
    let (comm_env_north, comm_north) = ContractEnvSyncComm::new_pair();

    let (comm_env_east, comm_east) = ContractEnvSyncComm::new_pair();
    let (comm_env_west, comm_west) = ContractEnvSyncComm::new_pair();
    let (comm_env_south, comm_south) = ContractEnvSyncComm::new_pair();




    let card_deal = fair_bridge_deal::<CardSet>();
    let (hand_north, hand_east, hand_south, hand_west) = card_deal.destruct();
    let initial_contract = Contract::new(contract_params);

    let initial_state_east = ContractAgentInfoSetSimple::new(East, hand_east, initial_contract.clone(), None);
    let initial_state_south = ContractAgentInfoSetSimple::new(South, hand_south, initial_contract.clone(), None);
    let initial_state_west = ContractDummyState::new(West, hand_west, initial_contract.clone());
    let initial_state_north = ContractAgentInfoSetSimple::new(North, hand_north, initial_contract.clone(), None);

    let random_policy = RandomPolicy::<ContractProtocolSpec, ContractAgentInfoSetSimple>::new();
    let policy_dummy = RandomPolicy::<ContractProtocolSpec, ContractDummyState>::new();

    let agent_east = AgentGen::new(East, initial_state_east, comm_east, policy_east);
    let agent_south = AgentGen::new(South, initial_state_south, comm_south, random_policy.clone() );
    let agent_west = AgentGen::new(West, initial_state_west, comm_west, policy_dummy);
    let agent_north = AgentGen::new(North, initial_state_north, comm_north, random_policy );


    let mut model = RoundRobinModelBuilder::new()
        .with_env_state(ContractEnvStateMin::new(initial_contract, None))?
        .with_env_action_process_fn(ContractProcessor{})?
        .with_local_agent(Box::new(agent_east), ComplexComm2048::StdSync(comm_env_east))?
        .with_local_agent(Box::new(agent_south), ComplexComm2048::StdSync(comm_env_south))?
        .with_local_agent(Box::new(agent_west), ComplexComm2048::StdSync(comm_env_west))?
        .with_local_agent(Box::new(agent_north), ComplexComm2048::StdSync(comm_env_north))?
        //.with_remote_agent(Side::South, env_comm_south)?
        .build()?;






    model.play().unwrap();

    Ok(())
}

pub fn test_sample_biased_distribution_parameters() -> Result<(), SztormError<ContractProtocolSpec>>{
    //setup_logger(LevelFilter::Debug, &None).unwrap();

    let mut trng = thread_rng();
    let tries = 100;
    let mut ace_spades_north = Vec::with_capacity(tries);
    for i in 0..tries{
        let sample: BiasedDistribution = trng.gen();
        //println!("{:?}", ron::to_string(&sample));
        print!("\r{:3}/100",i+1);
        ace_spades_north.push(f32::try_from(sample[North][&ACE_SPADES]).unwrap());

    }

    let sum = ace_spades_north.iter().map(|n| *n as f64).sum::<f64>();
    let count = ace_spades_north.len();

    println!("\rMean of probabilities that north has Ace Spades: {}", sum/(count as f64));



    Ok(())

}