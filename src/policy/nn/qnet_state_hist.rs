use std::marker::PhantomData;
use log::debug;
use tch::{Device, Tensor};
use tch::nn::{Adam, Optimizer, OptimizerConfig, Path, Sequential, VarStore};
use brydz_core::sztorm::spec::ContractProtocolSpec;
use brydz_core::sztorm::state::{BuildStateHistoryTensor, ContractAction};
use sztorm::{InformationSet, Policy};
use crate::policy::nn::Model;
use crate::{tch_model};
use crate::options::operation::{SequentialB, SequentialGen};

const CONTRACT_STATE_HISTORY_SIZE: i64 = (7 + (4 * 13)) * 53;
const CONTRACT_ACTION_SPARSE_SIZE: i64 = 53;
pub const CONTRACT_Q_INPUT_STATE_HIST_SPARSE: i64 = CONTRACT_STATE_HISTORY_SIZE + CONTRACT_ACTION_SPARSE_SIZE;

pub trait SequentialBuilder: Fn(&Path) -> Sequential{}

pub struct ContractStateHistQPolicy<S: BuildStateHistoryTensor + InformationSet<ContractProtocolSpec>>{
    model: Model,
    var_store: VarStore,
    device: Device,
    optimizer: Optimizer,
    state: PhantomData<S>,
}

impl<S: BuildStateHistoryTensor + InformationSet<ContractProtocolSpec>> ContractStateHistQPolicy<S>{
    pub fn new(var_store: VarStore, learning_rate: f64, seq_gen: &SequentialB) -> Self{
        let optimizer = Adam::default().build(&var_store, learning_rate)
            .expect("Error creating optimiser");
        Self{
            model: tch_model(&var_store.root(), seq_gen.build_sequential(&var_store.root())),
            device: var_store.root().device(),
            var_store,
            optimizer,
            state: Default::default()
        }
    }

    pub fn optimizer(&self) -> &Optimizer{&self.optimizer}
    pub fn optimizer_mut(&mut self) -> &mut Optimizer{ &mut self.optimizer}
    pub fn model(&self) -> &Model{
        &self.model
    }
    pub fn var_store(&self) -> &VarStore {&self.var_store}
    pub fn device(&self) -> &Device {&self.device}
}


impl<S: BuildStateHistoryTensor + InformationSet<ContractProtocolSpec>>
Policy<ContractProtocolSpec> for ContractStateHistQPolicy<S>
//where <<S as InformationSet<ContractProtocolSpec>>::ActionIteratorType as IntoIterator>::Item: Debug
{
    type StateType = S;

    fn select_action(&self, state: &Self::StateType) -> Option<ContractAction> {
        let in_array_state = state.state_history_tensor().f_flatten(0,1).unwrap();
        let mut current_best_action = None;
        let mut q_max: f32 = f32::MIN;

        for action in state.available_actions().into_iter(){
            let action_tensor = Tensor::from_slice(&action.sparse_representation());
            let input_tensor = Tensor::cat(&[&in_array_state, &action_tensor], 0);

            //let tensor = Tensor::from(&q_input[..]);

            let v:Vec<f32> = tch::no_grad(||{(self.model)(&input_tensor)}).get(0).try_into().unwrap();

            let current_q = v[0];
            debug!("Action {:?} checked with q value: {}", action, current_q);
            match current_best_action{
                None=>{
                    current_best_action = Some(action);
                    q_max = current_q;

                },
                Some(_) => {
                    if current_q > q_max{
                        q_max = current_q;
                        current_best_action = Some(action);
                    }
                }
            }

        }
        current_best_action
    }

}