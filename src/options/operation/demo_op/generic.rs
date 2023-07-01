use std::thread;
use brydz_core::bidding::Bid;
use brydz_core::cards::trump::TrumpGen;
use brydz_core::contract::{Contract, ContractParametersGen};
use brydz_core::deal::fair_bridge_deal;
use brydz_core::player::side::Side;
use brydz_core::sztorm::comm::ContractEnvSyncComm;
use brydz_core::sztorm::env::ContractProcessor;
use brydz_core::sztorm::spec::ContractProtocolSpec;
use brydz_core::sztorm::state::{ContractAgentInfoSetSimple, ContractDummyState, ContractEnvStateMin};
use karty::hand::CardSet;
use karty::suits::Suit::Spades;
use sztorm::{AgentAuto, AgentGen, RandomPolicy};
use sztorm::automatons::rr::{RoundRobinModelBuilder};
use sztorm::error::{CommError, SztormError};
use sztorm::protocol::{AgentMessage, EnvMessage};
use sztorm_net_ext::{ComplexComm, ComplexComm2048};
use sztorm_net_ext::tcp::TcpCommK2;

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

    let initial_state_east = ContractAgentInfoSetSimple::new(Side::East, hand_east, initial_contract.clone(), None);
    let initial_state_south = ContractAgentInfoSetSimple::new(Side::South, hand_south, initial_contract.clone(), None);
    let initial_state_west = ContractDummyState::new(Side::West, hand_west, initial_contract.clone());
    let initial_state_north = ContractAgentInfoSetSimple::new(Side::North, hand_north, initial_contract.clone(), None);

    let random_policy = RandomPolicy::<ContractProtocolSpec, ContractAgentInfoSetSimple>::new();
    let policy_dummy = RandomPolicy::<ContractProtocolSpec, ContractDummyState>::new();

    let agent_east = AgentGen::new(Side::East, initial_state_east, comm_east, random_policy.clone() );
    let mut agent_south = AgentGen::new(Side::South, initial_state_south, agent_comm_south, random_policy.clone() );
    let agent_west = AgentGen::new(Side::West, initial_state_west, comm_west, policy_dummy);
    let agent_north = AgentGen::new(Side::North, initial_state_north, comm_north, random_policy );


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