use std::thread;
use log::info;
use brydz_core::bidding::Bid;
use brydz_core::cards::trump::TrumpGen;
use brydz_core::contract::{Contract, ContractSpec};
use brydz_core::deal::fair_bridge_deal;
use brydz_core::player::side::{Side, SideMap};
use brydz_core::player::side::Side::*;
use brydz_core::sztorm::agent::ContractAgent;
use brydz_core::sztorm::comm::ContractEnvSyncComm;
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractProtocolSpec;
use brydz_core::sztorm::state::{ContractDummyState, ContractAgentStateMin, ContractEnvStateMin};
use brydz_simulator::settings::{ContractConfig, PlayerCfg};
use brydz_simulator::settings::Connection::Local;
use karty::hand::{CardSet};
use karty::suits::Suit::{Spades};
use sztorm::automatons::rr::{AgentRR, EnvironmentRR};
use sztorm::error::CommError;
use sztorm::protocol::{AgentMessage, EnvMessage};
use sztorm_net_ext::tcp::TcpCommK1;
use std::str::FromStr;
use sztorm::RandomPolicy;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
#[allow(unused)]
fn tur_sim(){
    let contract = ContractSpec::new(Side::East, Bid::init(TrumpGen::Colored(Spades), 2).unwrap());
    let (comm_env_north, comm_north) = ContractEnvSyncComm::new_pair();
    let (comm_env_east, comm_east) = ContractEnvSyncComm::new_pair();
    let (comm_env_west, comm_west) = ContractEnvSyncComm::new_pair();
    let (comm_env_south, comm_south) = ContractEnvSyncComm::new_pair();

    let comm_assotiation = SideMap::new(comm_env_north, comm_env_east, comm_env_south, comm_env_west);
    let initial_contract = Contract::new(contract);

    let env_initial_state = ContractEnvStateMin::new(initial_contract.clone(), None);
    let mut simple_env = ContractEnv::new(env_initial_state, comm_assotiation);

    let card_deal = fair_bridge_deal::<CardSet>();
    let (hand_north, hand_east, hand_south, hand_west) = card_deal.destruct();

    let initial_state_east = ContractAgentStateMin::new(East, hand_east, initial_contract.clone(), None);
    let initial_state_south = ContractAgentStateMin::new(South, hand_south, initial_contract.clone(), None);
    let initial_state_west = ContractDummyState::new(West, hand_west, initial_contract.clone());
    let initial_state_north = ContractAgentStateMin::new(North, hand_north, initial_contract, None);


    let random_policy = RandomPolicy::<ContractAgentStateMin>::new();
    let policy_dummy = RandomPolicy::<ContractDummyState>::new();

    let mut agent_east = ContractAgent::new(initial_state_east, comm_east, random_policy.clone() );
    let mut agent_south = ContractAgent::new(initial_state_south, comm_south, random_policy.clone() );
    let mut agent_west = ContractAgent::new(initial_state_west, comm_west, policy_dummy);
    let mut agent_north = ContractAgent::new(initial_state_north, comm_north, random_policy );

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
#[allow(unused)]
fn tur_sim_tcp(){
    let contract = ContractSpec::new(Side::East, Bid::init(TrumpGen::Colored(Spades), 2).unwrap());
    type TcpCommSim = TcpCommK1<AgentMessage<ContractProtocolSpec>, EnvMessage<ContractProtocolSpec>, CommError>;
    type TcpCommSimEnv = TcpCommK1<EnvMessage<ContractProtocolSpec>, AgentMessage<ContractProtocolSpec>, CommError>;
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

            let initial_state_east = ContractAgentStateMin::new(East, hand_east, initial_contract.clone(), None);
            let initial_state_south = ContractAgentStateMin::new(South, hand_south, initial_contract.clone(), None);
            let initial_state_west = ContractDummyState::new(West, hand_west, initial_contract.clone());
            let initial_state_north = ContractAgentStateMin::new(North, hand_north, initial_contract.clone(), None);


            let random_policy = RandomPolicy::<ContractAgentStateMin>::new();
            let policy_dummy = RandomPolicy::<ContractDummyState>::new();

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

fn serialize_settings_toml(){
    let sim_conf = ContractConfig::new_raw(
        PlayerCfg::new(String::from_str("AQT32.JT94.76.QT").unwrap(), Local),
        PlayerCfg::new(String::from_str("J97.Q875.AQT94.K").unwrap(), Local),
        PlayerCfg::new(String::from_str("K8.AK32.82.J9532").unwrap(), Local),
        PlayerCfg::new(String::from_str("654.6.KJ53.A8764").unwrap(), Local),
        String::from_str("2S").unwrap(),


    );

    let toml = toml::to_string(&sim_conf).unwrap();
    println!("{}", toml);
}

fn main() {
    setup_logger().unwrap();

    serialize_settings_toml();


    //tur_sim();
    //tur_sim_tcp();
}
