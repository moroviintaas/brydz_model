use std::thread;
use brydz_core::sztorm::spec::ContractProtocolSpec;
use sztorm::AgentAuto;
use sztorm::automatons::rr::{RoundRobinUniversalEnvironment};
use crate::options::operation::{DummyAgent, SimpleEnv};

pub fn single_play<D: AgentAuto<ContractProtocolSpec> + Send,
WHIST: AgentAuto<ContractProtocolSpec>+ Send,
OFFSIDE: AgentAuto<ContractProtocolSpec>+ Send>(ready_env: &mut SimpleEnv,
                   ready_declarer: &mut D,
                   ready_whist: &mut WHIST,
                   ready_offside: &mut OFFSIDE,
                   ready_dummy: &mut DummyAgent){

    thread::scope(|s|{
        s.spawn(||{
            ready_env.run_round_robin_uni_rewards().unwrap();
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
}