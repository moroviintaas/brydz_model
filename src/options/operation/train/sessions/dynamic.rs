use std::fmt::{Debug, Display, Formatter, Pointer};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use smallvec::SmallVec;
use brydz_core::contract::{Contract, ContractParameters, ContractRandomizer};
use brydz_core::deal::{BiasedHandDistribution, DealDistribution, DescriptionDeckDeal};
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
use sztorm::agent::{AgentGen, AgentGenT, AgentTrajectory, RandomPolicy};
use sztorm::state::agent::{InformationSet, ScoringInformationSet};
use sztorm::state::ConstructedState;
use sztorm_rl::actor_critic::ActorCriticPolicy;
use sztorm_rl::error::SztormRLError;
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
pub struct ContractA2CAgentLocalGen<
    ISW: WayToTensor,
    S: ConvertToTensor<ISW>  + ScoringInformationSet<ContractDP>
        + ConstructedState<ContractDP, (Side, ContractParameters, DescriptionDeckDeal)>
        +  Debug + Display + Clone>(
    pub AgentGenT<
        ContractDP,
        ActorCriticPolicy<
            ContractDP,
            S,
            //Box<dyn ConvertToTensorD<ISW>>,
            ISW>,
        ContractAgentSyncComm,
    >
);
pub trait ContractInfoSetTraitJoined<ISW: WayToTensor>:
ConvertToTensor<ISW>
+ ConstructedState<ContractDP, (Side, ContractParameters, DescriptionDeckDeal)>
+ ScoringInformationSet<ContractDP>
+ Debug + Display{}










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
    declarer_trajectories: Vec<AgentTrajectory<ContractDP, S>>,
    whist_trajectories: Vec<AgentTrajectory<ContractDP, S>>,
    offside_trajectories: Vec<AgentTrajectory<ContractDP, S>>,



}

impl<ISW2T: WayToTensor, S: ContractInfoSetTraitJoined<ISW2T> + Clone> DynamicContractA2CSession<ISW2T, S>{

    fn reset_game(&mut self, rng: &mut ThreadRng, distribution: &DealDistribution, ){
        //let
        todo!()
    }
    //fn reset_game_pool(&mut self, rng: &mut ThreadRng, distribution_pool:  )

    pub fn train_agents_one_epoch(&mut self, games_in_epoch: usize,
        distribution_pool: Option<&[DealDistribution]>,
        contract_randomizer: &ContractRandomizer,
        ) -> Result<(), SztormRLError<ContractDP>>{

        let mut rng = thread_rng();
        for _ in 0..games_in_epoch{
            /*
            let distr = if let Some(pool) = distribution_pool{
                let d = pool.choose(&mut rng).unwrap_or(&DealDistribution::Fair);

            }

             */
        }


        Ok(())

    }

    pub fn train_agents(&mut self,
                        epochs: usize,
                        games_in_epoch: usize,

                        distribution_pool: Option<&[DealDistribution]>){

    }
}

pub struct DynamicContractA2CSessionBuilder<ISW2T: WayToTensor, S: ContractInfoSetTraitJoined<ISW2T> + Clone>{
    environment: Option<ContractEnv<ContractEnvStateComplete, ContractEnvSyncComm>>,
    declarer: Option<ContractA2CAgentLocalGen<ISW2T, S>>,
    whist: Option<ContractA2CAgentLocalGen<ISW2T, S>>,
    dummy: Option<AgentGen<ContractDP, RandomPolicy<ContractDP, ContractDummyState>, ContractAgentSyncComm>>,
    offside: Option<ContractA2CAgentLocalGen<ISW2T, S>>,


}

//impl<ISW2T: WayToTensor, S: ContractInfoSetTraitJoined<ISW2T> + Clone>