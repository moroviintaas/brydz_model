use std::thread;
use log::info;
use brydz_core::bidding::Bid;
use brydz_core::cards::trump::TrumpGen;
use brydz_core::contract::{Contract, ContractParametersGen};
use brydz_core::deal::fair_bridge_deal;
use brydz_core::player::side::{Side, SideMap};
use brydz_core::player::side::Side::*;
use brydz_core::sztorm::agent::ContractAgent;
use brydz_core::sztorm::comm::ContractEnvSyncComm;
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractProtocolSpec;
use brydz_core::sztorm::state::{ContractDummyState, ContractAgentInfoSetSimple, ContractEnvStateMin};
use brydz_simulator::settings::{ContractConfig, PlayerCfg};
use brydz_simulator::settings::Connection::Local;
use karty::hand::{CardSet};
use karty::suits::Suit::{Spades};
use sztorm::automatons::rr::{AgentRR, EnvironmentRR};
use sztorm::error::CommError;
use sztorm::protocol::{AgentMessage, EnvMessage};
use sztorm_net_ext::tcp::TcpCommK1;
use std::str::FromStr;
use sztorm::{AgentGen, RandomPolicy};


use clap::Parser;
use brydz_simulator::options;
use brydz_simulator::options::operation::{generate_contracts, Operation};

//use crate::options::operation::{GenContract, Operation};
//mod error;
//mod options;
//mod error;



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

    let cli = options::Cli::parse();
    options::setup_logger(cli.log_level).unwrap();
    serialize_settings_toml();
    match &cli.command{
        Operation::Gen2(distribute) => {
            generate_contracts(distribute);
        }
        Operation::TestLocal =>{
            options::operation::test_ops::tur_sim();
        }
        Operation::TestTcp => {
            options::operation::test_ops::tur_sim_tcp();
        }
    }







    //
}
