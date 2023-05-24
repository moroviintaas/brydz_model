use std::thread::{self};
use log::{info};
use rand::{Rng, thread_rng};
use tch::nn::VarStore;
use brydz_core::bidding::Bid;
use brydz_core::cards::trump::TrumpGen;
use brydz_core::contract::{Contract, ContractParametersGen};
use brydz_core::deal::{BiasedHandDistribution, fair_bridge_deal};
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








