use std::fmt::{Debug, Display, Formatter, Pointer};
use smallvec::SmallVec;
use brydz_core::contract::Contract;
use brydz_core::deal::BiasedHandDistribution;
use brydz_core::meta::HAND_SIZE;
use brydz_core::player::side::Side;
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
use brydz_core::sztorm::state::{ContractAction, ContractDummyState, ContractInfoSet, ContractState, ContractStateConverter, CreatedContractInfoSet};
use karty::hand::CardSet;
use sztorm::agent::{AgentGen, AgentGenT, RandomPolicy};
use sztorm::state::agent::{InformationSet, ScoringInformationSet};
use sztorm_rl::actor_critic::ActorCriticPolicy;
use sztorm_rl::tensor_repr::{ConvertToTensor, ConvertToTensorD, ConvStateToTensor, WayFromTensor, WayToTensor};

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
pub struct ContractA2CAgentLocalGen<ISW: WayToTensor, S: ConvertToTensor<ISW> + CreatedContractInfoSet + Debug + Display + Clone>(
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
pub trait ContractInfoSetTraitJoined<ISW: WayToTensor>: ConvertToTensor<ISW> + CreatedContractInfoSet + Debug + Display{}










impl<ISW: WayToTensor, T: ContractInfoSetTraitJoined<ISW>> ContractInfoSetTraitJoined<ISW> for Box<T>{}
/*pub type ContractA2CAgentLocalBoxing<ISW, IS: ContractInfoSetTraitJoined<ISW>> = ContractA2CAgentLocalGen<
    ISW,
    //Box<dyn ContractInfoSetTraitJoined<
    //    ISW,
    //    ActionIteratorType=SmallVec<[ContractAction; HAND_SIZE]>,
    //    RewardType=i32
   //     >>
    IS
>;

 */




pub struct DynamicContractA2CSession<ISW2T: WayToTensor, S: ContractInfoSetTraitJoined<ISW2T> + Clone>{
    environment: ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>,
    declarer: ContractA2CAgentLocalGen<ISW2T, S>,
    whist: ContractA2CAgentLocalGen<ISW2T, S>,
    dummy: AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>,
    offside: ContractA2CAgentLocalGen<ISW2T, S>,
}