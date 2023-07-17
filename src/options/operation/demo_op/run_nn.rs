use tch::nn::VarStore;
use brydz_core::bidding::Bid;
use brydz_core::cards::trump::TrumpGen;
use brydz_core::contract::{Contract, ContractParametersGen};
use brydz_core::deal::fair_bridge_deal;
use brydz_core::player::side::Side;
use brydz_core::sztorm::comm::ContractEnvSyncComm;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractAgentInfoSetSimple, ContractDummyState, ContractEnvStateMin};
use karty::hand::CardSet;
use karty::suits::Suit::Spades;
use sztorm::agent::{AgentGen, RandomPolicy};
use sztorm::env::RoundRobinModelBuilder;
use sztorm::error::SztormError;
use sztorm_net_ext::ComplexComm2048;
use crate::SyntheticContractQNetSimple;

pub fn test_with_untrained_network() -> Result<(), SztormError<ContractDP>>{

    let vs_east = VarStore::new(tch::Device::Cpu);

    let policy_east = SyntheticContractQNetSimple::new(vs_east, 0.25);

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

    let initial_state_east = ContractAgentInfoSetSimple::new(Side::East, hand_east, initial_contract.clone(), None);
    let initial_state_south = ContractAgentInfoSetSimple::new(Side::South, hand_south, initial_contract.clone(), None);
    let initial_state_west = ContractDummyState::new(Side::West, hand_west, initial_contract.clone());
    let initial_state_north = ContractAgentInfoSetSimple::new(Side::North, hand_north, initial_contract.clone(), None);

    let random_policy = RandomPolicy::<ContractDP, ContractAgentInfoSetSimple>::new();
    let policy_dummy = RandomPolicy::<ContractDP, ContractDummyState>::new();

    let agent_east = AgentGen::new(Side::East, initial_state_east, comm_east, policy_east);
    let agent_south = AgentGen::new(Side::South, initial_state_south, comm_south, random_policy.clone() );
    let agent_west = AgentGen::new(Side::West, initial_state_west, comm_west, policy_dummy);
    let agent_north = AgentGen::new(Side::North, initial_state_north, comm_north, random_policy );


    let mut model = RoundRobinModelBuilder::new()
        .with_env_state(ContractEnvStateMin::new(initial_contract, None))?
        .with_local_agent(Box::new(agent_east), ComplexComm2048::StdSync(comm_env_east))?
        .with_local_agent(Box::new(agent_south), ComplexComm2048::StdSync(comm_env_south))?
        .with_local_agent(Box::new(agent_west), ComplexComm2048::StdSync(comm_env_west))?
        .with_local_agent(Box::new(agent_north), ComplexComm2048::StdSync(comm_env_north))?
        //.with_remote_agent(Side::South, env_comm_south)?
        .build()?;






    model.play().unwrap();

    Ok(())
}