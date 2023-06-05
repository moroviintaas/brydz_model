use std::cmp::min;
use std::thread;
use log::debug;
use rand::distributions::Distribution;
use rand::prelude::ThreadRng;
use rand_distr::Geometric;
use tch::Kind::Float;
use tch::Tensor;
use brydz_core::meta::HAND_SIZE;
use brydz_core::sztorm::agent::ContractAgent;
use brydz_core::sztorm::comm::{ContractAgentSyncComm, ContractEnvSyncComm};
use brydz_core::sztorm::env::ContractEnv;
use brydz_core::sztorm::spec::ContractProtocolSpec;
use brydz_core::sztorm::state::{BuildStateHistoryTensor, ContractDummyState, ContractEnvStateMin};
use sztorm::{InformationSet, PolicyAgent, RandomPolicy};
use sztorm::automatons::rr::{AgentAuto, EnvironmentRR};
use crate::{ContractStateHistQPolicy, EEPolicy};
use crate::error::BrydzSimError;
use sztorm::DistinctAgent;

pub(crate) type QNetStateHistAgent<St> = ContractAgent<St, ContractAgentSyncComm, EEPolicy<ContractStateHistQPolicy<St>>>;
pub(crate) type DummyAgent2 = ContractAgent<ContractDummyState, ContractAgentSyncComm, RandomPolicy<ContractProtocolSpec, ContractDummyState>>;
pub(crate) type SimpleEnv2 = ContractEnv<ContractEnvStateMin, ContractEnvSyncComm>;

pub fn train_episode_state_hist<St: InformationSet<ContractProtocolSpec> + BuildStateHistoryTensor + Send>(
    ready_env: &mut SimpleEnv2,
    ready_declarer: &mut QNetStateHistAgent<St>,
    ready_whist: &mut QNetStateHistAgent<St>,
    ready_offside: &mut QNetStateHistAgent<St>,
    ready_dummy: &mut DummyAgent2,
    rng: &mut ThreadRng, geo: &mut Geometric) -> Result<(), BrydzSimError>
where for<'a> f32: From<&'a <St as InformationSet<ContractProtocolSpec>>::RewardType>{

    let step_start_explore = min(geo.sample(rng), HAND_SIZE as u64);
    let _ = &mut ready_declarer.policy_mut().set_exploiting_start(step_start_explore*2);

    ready_whist.policy_mut().set_exploiting_start(step_start_explore);
    ready_offside.policy_mut().set_exploiting_start(step_start_explore);

    thread::scope(|s|{
        s.spawn(||{
            ready_env.env_run_rr().unwrap();
        });
        s.spawn(||{
            ready_declarer.run_rr().unwrap();
        });

        s.spawn(||{
            ready_whist.run_rr().unwrap();
        });

        s.spawn(||{
            ready_offside.run_rr().unwrap();
        });

        s.spawn(||{
            ready_dummy.run_rr().unwrap();
        });
    });

    for agent in [ready_declarer, ready_whist, ready_offside ]{
        let mut accumulated_reward = 0.0;
        for i in (agent.policy().exploitation_start() as usize.. agent.trace().len()).rev(){
            let (state, action, reward ) =  &agent.trace()[i];
            accumulated_reward += &Into::<f32>::into(reward);
            debug!("Applying train vector for {} (accumulated reward: {})", agent.id(), accumulated_reward);
            let t = state.state_history_tensor().f_flatten(0,1).unwrap();
            let ta = Tensor::of_slice(&action.sparse_representation());
            let input = tch::Tensor::cat(&[t,ta], 0);

            //let optimiser = agent.policy_mut().internal_policy_mut().optimizer_mut();
            let q = (agent.policy_mut().internal_policy_mut().model())(&input);
            let q_from_net = tch::Tensor::of_slice(&[accumulated_reward]);

            //println!("{q:} {q_from_net:}");
            let diff = &q-&q_from_net;
            let loss = (&diff * &diff).mean(Float);

            agent.policy_mut().internal_policy_mut().optimizer_mut().zero_grad();
            loss.backward();
            agent.policy_mut().internal_policy_mut().optimizer_mut().step();
            //println!("{loss:}");

        }
    }
    Ok(())


}