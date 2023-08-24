use std::fmt::{Debug, Display};
use smallvec::SmallVec;
use brydz_core::meta::HAND_SIZE;
use brydz_core::sztorm::{
    comm::{
        ContractEnvSyncComm
    },
    env::{
        ContractEnv
    },
    state::{
        ContractEnvStateComplete
    }
};
use brydz_core::sztorm::comm::ContractAgentSyncComm;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractAction, ContractInfoSet, ContractState, ContractStateConverter, CreatedContractInfoSet};
use sztorm::agent::AgentGenT;
use sztorm_rl::actor_critic::ActorCriticPolicy;
use sztorm_rl::tensor_repr::{ConvertToTensor, ConvertToTensorD, ConvStateToTensor, WayToTensor};

/*
type ContractA2CAgentLocal<ISW> = AgentGenT<
    ContractDP,
    ActorCriticPolicy<
        ContractDP,
        Box<dyn CreatedContractInfoSet<
            ActionIteratorType=(),
            RewardType=()>
            + ConvertToTensor<ISW>
        >,
        Box<dyn ConvStateToTensor<>>>,
    ContractAgentSyncComm>;
*/
//trait InfoSetCompl =
pub struct ContractA2CAgentLocalGen<ISW: WayToTensor, S: ConvertToTensor<ISW> + CreatedContractInfoSet + Debug + Display>(
    pub AgentGenT<
        ContractDP,
        ActorCriticPolicy<
            ContractDP,
            /*
            Box<dyn CreatedContractInfoSet<
                ActionIteratorType=SmallVec<[ContractAction; HAND_SIZE]>,
                RewardType=i32> + ConvertToTensorD<ISW>>,

             */
            S,
            //Box<dyn ConvertToTensorD<ISW>>,
            ISW>,
        ContractAgentSyncComm,
    >
);
pub trait ContractInfoSetTraitJoined<ISW: WayToTensor>: ConvertToTensor<ISW> + CreatedContractInfoSet + Debug{}
pub type ContractA2CAgentLocalBoxing<ISW> = ContractA2CAgentLocalGen<
    ISW,
    Box<dyn ContractInfoSetTraitJoined<
        ISW,
        ActionIteratorType=SmallVec<[ContractAction; HAND_SIZE]>,
        RewardType=i32>>>;

/*
pub struct DynamicContractA2CSession{
    environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
    declarer: AgentGenT<
    >
}*/