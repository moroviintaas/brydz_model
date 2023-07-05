use std::thread;
use log::info;
use brydz_core::bidding::Bid;
use brydz_core::cards::trump::TrumpGen;
use brydz_core::contract::{Contract, ContractParametersGen};
use brydz_core::deal::fair_bridge_deal;
use brydz_core::player::side::{Side, SideMap};
use brydz_core::sztorm::agent::ContractAgent;
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractProtocolSpec;
use brydz_core::sztorm::state::{ContractAgentInfoSetSimple, ContractDummyState, ContractEnvStateMin};
use karty::hand::CardSet;
use karty::suits::Suit::Spades;
use sztorm::automatons::rr::{EnvironmentRR};
use sztorm::error::CommError;
use sztorm::protocol::{AgentMessage, EnvMessage};
use sztorm::{AgentAuto, RandomPolicy};
use sztorm_net_ext::tcp::TcpCommK1;

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
            simple_env.run_rr().unwrap();
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

            let initial_state_east = ContractAgentInfoSetSimple::new(Side::East, hand_east, initial_contract.clone(), None);
            let initial_state_south = ContractAgentInfoSetSimple::new(Side::South, hand_south, initial_contract.clone(), None);
            let initial_state_west = ContractDummyState::new(Side::West, hand_west, initial_contract.clone());
            let initial_state_north = ContractAgentInfoSetSimple::new(Side::North, hand_north, initial_contract.clone(), None);


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