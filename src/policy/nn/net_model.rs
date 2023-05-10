use std::cell::{Cell, RefCell};
use std::slice::SliceIndex;
use std::sync::Mutex;
use log::debug;
use rand::thread_rng;
use tch::{Device, nn, Tensor};
use tch::nn::{Adam, Optimizer, OptimizerConfig, VarStore};
use brydz_core::error::BridgeCoreError;
use brydz_core::meta::HAND_SIZE;
use brydz_core::sztorm::spec::ContractProtocolSpec;
use brydz_core::sztorm::state::{ContractAction, ContractAgentInfoSetSimple};
use sztorm::{InformationSet, Policy, QFunction};
use sztorm::protocol::ProtocolSpecification;
use smallvec::SmallVec;
use rand::prelude::SliceRandom;

type Model = Box<dyn Fn(&Tensor) -> Tensor + Send>;
const HIDDEN_LAYER_1_SIZE: i64 = 512;
const CONTRACT_STATE_SIZE: i64 = 222;
const CONTRACT_ACTION_SIZE: i64 = 2;
const CONTRACT_Q_INPUT_SIZE: i64 = CONTRACT_STATE_SIZE + CONTRACT_ACTION_SIZE;



fn q_func_contract(p: &nn::Path, input_tensor_size: i64) -> Model {
    let seq = nn::seq()
        .add(nn::linear(p/"i", input_tensor_size, HIDDEN_LAYER_1_SIZE, Default::default()))
        .add(nn::linear(p/"h1", HIDDEN_LAYER_1_SIZE, 1, Default::default()));
    let device = p.device();
    Box::new(move |xs|{
        let xs = xs.to_device(device).apply(&seq);
        xs
    } )

}

pub struct ContractQNetSimple {
    pub model: Model,
    pub var_store: VarStore,
    pub device: Device,
    optimiser: Optimizer,

}

impl ContractQNetSimple {
    pub fn new(var_store: VarStore, learning_rate: f64) -> Self{
        let optimiser = Adam::default().build(&var_store, learning_rate).unwrap();
        Self{model: q_func_contract(&var_store.root(), CONTRACT_Q_INPUT_SIZE,),
        device: var_store.root().device(),
        var_store,
        optimiser}
    }

    pub fn optimizer(&self) -> &Optimizer{
        &self.optimiser
    }
    pub fn optimizer_mut(&mut self) -> &mut Optimizer{
        &mut self.optimiser
    }
}/*
impl QFunction<ContractProtocolSpec> for ContractQNetSimple{
    type StateType = ContractAgentInfoSetSimple;
    type QValue = f32;

    fn q_value(&self, state: &Self::StateType, action: &ContractAction) -> Result<Self::QValue, BridgeCoreError> {
        let in_array_state:[f32; CONTRACT_STATE_SIZE as usize] = state.into();
        let in_array_action: [f32; CONTRACT_ACTION_SIZE as usize] = action.into();
        let mut vec = Vec::from(in_array_state).append(&mut Vec::from(in_array_action));
    }
}*/




impl Policy<ContractProtocolSpec> for ContractQNetSimple{
    type StateType = ContractAgentInfoSetSimple;

    fn select_action(&self, state: &Self::StateType) -> Option<ContractAction> {
        let in_array_state:[f32; CONTRACT_STATE_SIZE as usize] = state.into();
        let mut q_input: Vec<f32> = Vec::from(in_array_state);
        q_input.append(&mut vec![0.0, 0.0]);
        let mut q_max: f32 = f32::MIN;

        //let guard = self.model.lock().unwrap();

        let mut current_best_action = None;
        for action in state.available_actions().into_iter(){
            let action_array: [f32; CONTRACT_ACTION_SIZE as usize] = (&action).into();
            q_input[(CONTRACT_Q_INPUT_SIZE-CONTRACT_ACTION_SIZE) as usize] = action_array[0];
            q_input[(CONTRACT_Q_INPUT_SIZE-CONTRACT_ACTION_SIZE) as usize +1] = action_array[1];
            let tensor = Tensor::from(&q_input[..]);

            let v:Vec<f32> = tch::no_grad(||{(&self.model)(&tensor)}).get(0).into();

            let current_q = v[0];
            debug!("Action {} checked with q value: {}", action, current_q);
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
        /*state.available_actions().into_iter().fold((None, f32::MIN), |acc, x|{

        })*/
    }
}

pub struct EEPolicy<IntPolicy: Policy<ContractProtocolSpec>>{
    start_exploiting: u64,
    exploiting_policy: IntPolicy,
    step_counter: u64,

}

impl<IntPolicy: Policy<ContractProtocolSpec>> EEPolicy<IntPolicy>{
    pub fn new(policy: IntPolicy) -> Self{
        Self{exploiting_policy: policy, start_exploiting: 0, step_counter: 0}
    }
    pub fn exploiting_policy_mut(&mut self) -> &mut IntPolicy{
        &mut self.exploiting_policy
    }
    pub fn exploitation_start(&self) -> u64{
        self.start_exploiting
    }
    pub fn set_exploiting_start(&mut self, start: u64){
        self.start_exploiting = start
    }
    pub fn exploiting_start(&self) -> u64{
        self.start_exploiting
    }
    pub fn reset_step_counter(&mut self){
        self.step_counter = 0;
    }
    pub fn get_step_counter(&self) -> u64{
        self.step_counter
    }
    pub fn internal_policy(&self) -> &IntPolicy{
        &self.exploiting_policy
    }
    pub fn internal_policy_mut(&mut self) -> &mut IntPolicy{
        &mut self.exploiting_policy
    }
}

impl<IntPolicy: Policy<ContractProtocolSpec>> Policy<ContractProtocolSpec> for EEPolicy<IntPolicy>{
    type StateType = IntPolicy::StateType;

    fn select_action(&self, state: &Self::StateType) -> Option<ContractAction> {
        //self.step_counter.set(self.step_counter.get() + 1);
        if self.step_counter >= self.start_exploiting{

            self.exploiting_policy.select_action(state)
        } else{
            let mut rng = thread_rng();
            let available_actions: SmallVec<[ContractAction; HAND_SIZE]> = state.available_actions().into_iter().collect();
            let action = available_actions.choose(&mut rng);
            action.map(|a| a.to_owned())
        }


    }

    fn select_action_mut(&mut self, state: &Self::StateType) -> Option<ContractAction> {
        self.step_counter += 1;
        if self.step_counter > self.start_exploiting{

            self.exploiting_policy.select_action(state)
        } else{
            let mut rng = thread_rng();
            let available_actions: SmallVec<[ContractAction; HAND_SIZE]> = state.available_actions().into_iter().collect();
            let action = available_actions.choose(&mut rng);
            action.map(|a| a.to_owned())
        }
    }
}