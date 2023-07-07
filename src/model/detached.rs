use std::thread;
use brydz_core::sztorm::spec::ContractProtocolSpec;
use sztorm::agent::AutomaticAgent;
use sztorm::env::RoundRobinUniversalEnvironment;
use crate::options::operation::{DummyAgent, SimpleEnv};

pub fn single_play<D: AutomaticAgent<ContractProtocolSpec> + Send,
WHIST: AutomaticAgent<ContractProtocolSpec>+ Send,
OFFSIDE: AutomaticAgent<ContractProtocolSpec>+ Send>(ready_env: &mut SimpleEnv,
                                                     ready_declarer: &mut D,
                                                     ready_whist: &mut WHIST,
                                                     ready_offside: &mut OFFSIDE,
                                                     ready_dummy: &mut DummyAgent){

    thread::scope(|s|{
        s.spawn(||{
            ready_env.run_round_robin_uni_rewards().unwrap();
        });
        s.spawn(||{
            ready_declarer.run().unwrap();
        });

        s.spawn(||{
            ready_whist.run().unwrap();
        });

        s.spawn(||{
            ready_offside.run().unwrap();
        });

        s.spawn(||{
            ready_dummy.run().unwrap();
        });
    });
}