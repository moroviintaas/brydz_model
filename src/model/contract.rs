use brydz_core::contract::{Contract, ContractParameters};
use brydz_core::player::side::{Side, SideMap};
use brydz_core::player::side::Side::*;
use brydz_core::sztorm::comm::ContractEnvSyncComm;
use brydz_core::sztorm::env::ContractProcessor;
use brydz_core::sztorm::spec::ContractProtocolSpec;
use brydz_core::sztorm::state::{ContractAction, ContractAgentInfoSetSimple, ContractDummyState, ContractEnvStateMin, ContractState, ContractStateUpdate};
use sztorm::automatons::rr::{RoundRobinModel, RoundRobinModelBuilder};
use sztorm::{ActionProcessor, AgentGen, EnvCommEndpoint, EnvironmentState, RandomPolicy, State};
use sztorm::error::{CommError, SetupError};
use sztorm::protocol::{AgentMessage, EnvMessage};
use sztorm_net_ext::{ComplexComm1024, ComplexComm2048};
use crate::error::{BrydzSimError, SimulationError};
use crate::SimContractParams;
/*
pub(crate) fn contract_process_action(mut state: ContractEnvStateMin, agent_id: Side, action: ContractAction)
                                      -> Result<Vec<(Side, ContractStateUpdate)>, SetupError<ContractProtocolSpec>>{
    let state_update =
            if state.is_turn_of_dummy() && Some(agent_id) == state.current_player(){
                ContractStateUpdate::new(state.dummy_side(), action)
            } else {
                ContractStateUpdate::new(agent_id.to_owned(), action)
            };
            state.update(state_update)?;
            Ok(vec![(North,state_update),(East,state_update),(South,state_update), (West, state_update)])
}
*/
pub(crate) type LocalModelContract<ProcessAction> =
RoundRobinModel<ContractProtocolSpec, ContractEnvStateMin, ProcessAction, ComplexComm1024<EnvMessage<ContractProtocolSpec>, AgentMessage<ContractProtocolSpec>, CommError<ContractProtocolSpec>>>;

pub fn generate_local_model(params: &SimContractParams) -> Result<LocalModelContract<ContractProcessor>, BrydzSimError>{
    let (comm_env_north, comm_north) = ContractEnvSyncComm::new_pair();
    let (comm_env_east, comm_east) = ContractEnvSyncComm::new_pair();
    let (comm_env_west, comm_west) = ContractEnvSyncComm::new_pair();
    let (comm_env_south, comm_south) = ContractEnvSyncComm::new_pair();

    let agent_comm_map = SideMap::new(comm_north, comm_east, comm_south, comm_west);
    let env_comm_map = SideMap::new(comm_env_north, comm_env_east, comm_env_south, comm_env_west);

    let card_deal = params.cards();
    let initial_contract = Contract::new(params.parameters().clone());
    let declarer = params.parameters().declarer();
    let dummy = params.parameters().declarer().next_i(2);
    let def1 = params.parameters().declarer().next_i(1);
    let def2 = params.parameters().declarer().next_i(3);

    //let (hand_north, hand_east, hand_south, hand_west) = card_deal.destruct();

    //this must be differed when Agent has different state's type
    let initial_state_declarer = ContractAgentInfoSetSimple::new(declarer, card_deal[&declarer], initial_contract.clone(), None);
    let initial_state_def1 = ContractAgentInfoSetSimple::new(def1, card_deal[&def1], initial_contract.clone(), None);
    let initial_state_dummy = ContractDummyState::new(dummy, card_deal[&dummy], initial_contract.clone());
    let initial_state_def2 = ContractAgentInfoSetSimple::new(def2, card_deal[&def2], initial_contract.clone(), None);

    //policy select
    let random_policy = RandomPolicy::<ContractProtocolSpec, ContractAgentInfoSetSimple>::new();
    let policy_dummy = RandomPolicy::<ContractProtocolSpec, ContractDummyState>::new();

    let (comm_declarer, comm_def1, comm_dummy, comm_def2) = agent_comm_map.destruct_start_with(declarer);
    let (comm_env_declarer, comm_env_def1, comm_env_dummy, comm_env_def2) = env_comm_map.destruct_start_with(declarer);

    let mut agent_declarer = AgentGen::new(declarer, initial_state_declarer, comm_declarer, random_policy.clone() );
    let mut agent_def1 = AgentGen::new(def1, initial_state_def1, comm_def1, random_policy.clone() );
    let mut agent_dummy = AgentGen::new(dummy, initial_state_dummy, comm_dummy, policy_dummy);
    let mut agent_def2 = AgentGen::new(def2, initial_state_def2, comm_def2, random_policy );

    let mut model = RoundRobinModelBuilder::new()
        .with_env_state(ContractEnvStateMin::new(initial_contract, None))?
        .with_env_action_process_fn(ContractProcessor{})?
        .with_local_agent(Box::new(agent_declarer), ComplexComm1024::StdSync(comm_env_declarer))?
        .with_local_agent(Box::new(agent_def1), ComplexComm1024::StdSync(comm_env_def1))?
        .with_local_agent(Box::new(agent_dummy), ComplexComm1024::StdSync(comm_env_dummy))?
        .with_local_agent(Box::new(agent_def2), ComplexComm1024::StdSync(comm_env_def2))?
        //.with_remote_agent(Side::South, env_comm_south)?
        .build()?;

    Ok(model)
}



