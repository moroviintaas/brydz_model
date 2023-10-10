use std::fmt::Debug;
use brydz_core::contract::ContractParameters;
use brydz_core::deal::DescriptionDeckDeal;
use brydz_core::player::side::Side;
use brydz_core::sztorm::comm::{ContractAgentSyncComm};

use brydz_core::sztorm::spec::ContractDP;

use sztorm::agent::{AgentGen, AgentGenT, AutomaticAgentRewarded, Policy, PolicyAgent, PresentPossibleActions, ScoringInformationSet, StatefulAgent};
use sztorm::domain::Construct;

use sztorm_rl::LearningNetworkPolicy;
use sztorm_rl::tensor_repr::{ConvertToTensor, WayToTensor};

pub trait ContractInfoSetForLearning<ISW: WayToTensor>:
ConvertToTensor<ISW>
+ for<'a> Construct<(&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>
+ ScoringInformationSet<ContractDP>
+ PresentPossibleActions<ContractDP>
+ Debug {}

impl<ISW: WayToTensor, T: ConvertToTensor<ISW>
+ for<'a> Construct<(&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>
+ ScoringInformationSet<ContractDP>
+ PresentPossibleActions<ContractDP>
+ Debug > ContractInfoSetForLearning<ISW> for T{}

pub trait SessionAgentTraitDyn<
    ISW: WayToTensor,
    P: Policy<ContractDP>
> where <P as Policy<ContractDP>>::InfoSetType: ContractInfoSetForLearning<ISW>
 + for<'a> Construct<(&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>{}

pub trait SessionAgentTrait<
    ISW: WayToTensor,
    P: Policy<ContractDP>
> where <P as Policy<ContractDP>>::InfoSetType: ContractInfoSetForLearning<ISW>
 + for<'a> Construct<(&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>{

    fn create_for_session(
        side: Side,
        contract_params: &ContractParameters,
        deal_description: & DescriptionDeckDeal,
        comm: ContractAgentSyncComm,
        policy: P
    ) -> Self;
}

impl<
    ISW: WayToTensor,
    P: Policy<ContractDP>
> SessionAgentTrait<ISW, P> for AgentGenT<ContractDP, P, ContractAgentSyncComm>
where for<'a> <P as Policy<ContractDP>>::InfoSetType: Construct<(&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>
+ ScoringInformationSet<ContractDP> + ConvertToTensor<ISW> + PresentPossibleActions<ContractDP>
{
    fn create_for_session(side: Side, contract_params: &ContractParameters, deal_description: &DescriptionDeckDeal, comm: ContractAgentSyncComm, policy: P) -> Self {
        type IS<P> = <P as Policy<ContractDP>>::InfoSetType;
        AgentGenT::new(
            side,
            <IS<P>>::construct_from((&side, &contract_params, &deal_description)),
            comm, policy)
    }
}

impl<
    ISW: WayToTensor,
    P: Policy<ContractDP>
> SessionAgentTrait<ISW, P> for AgentGen<ContractDP, P, ContractAgentSyncComm>
where for<'a> <P as Policy<ContractDP>>::InfoSetType:
    Construct<(&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>
    + PresentPossibleActions<ContractDP>
    + ScoringInformationSet<ContractDP> + ConvertToTensor<ISW>
{
    fn create_for_session(side: Side, contract_params: &ContractParameters, deal_description: &DescriptionDeckDeal, comm: ContractAgentSyncComm, policy: P) -> Self {
        type IS<P> = <P as Policy<ContractDP>>::InfoSetType;
        AgentGen::new(
            side,
            <IS<P>>::construct_from((&side, &contract_params, &deal_description)),
            comm, policy)
    }
}

/*
impl <
    ISW: WayToTensor,
    P: Policy<ContractDP>
> SessionAgent<ISW, P>
where <P as Policy<ContractDP>>::StateType: ContractInfoSetForLearning<ISW>
 + for<'a> ConstructedState<ContractDP, (&'a Side, &'a ContractParameters, &'a DescriptionDeckDeal)>{

}

 */


pub trait ContractLearningAgent: AutomaticAgentRewarded<ContractDP>  + PolicyAgent<ContractDP>
where <Self as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>,
<Self as StatefulAgent<ContractDP>>::InfoSetType: ScoringInformationSet<ContractDP>{}

impl <T: AutomaticAgentRewarded<ContractDP>  + PolicyAgent<ContractDP>>
ContractLearningAgent for T
where <T as PolicyAgent<ContractDP>>::Policy: LearningNetworkPolicy<ContractDP>,
<T as StatefulAgent<ContractDP>>::InfoSetType: ScoringInformationSet<ContractDP>
{}


/*
impl<
    P: LearningNetworkPolicy<ContractDP>,
    Comm: > ContractLearningAgent for AgentGenT<ContractDP, P, Comm>
where Comm: CommEndpoint<
        OutwardType = AgentMessage<ContractDP>,
        InwardType = EnvMessage<ContractDP>,
        Error=CommError<ContractDP>>,
    <P as Policy<ContractDP>>::StateType: ScoringInformationSet<ContractDP> + Clone

{}

 */

