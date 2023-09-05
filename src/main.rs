use brydz_simulator::settings::{ContractConfig, PlayerCfg};
use brydz_simulator::settings::Connection::Local;
use std::str::FromStr;


use clap::Parser;
use tch::{Device, nn, Tensor};
use tch::Device::Cpu;
use tch::nn::{Adam, VarStore};
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractAgentInfoSetAllKnowing, ContractAgentInfoSetSimple, ContractInfoSetConvert420, ContractInfoSetConvert420Normalised};

use brydz_simulator::error::BrydzSimError;
use brydz_simulator::{
    options};
use brydz_simulator::options::operation::{Operation,
                                          sim2,
};
use brydz_simulator::options::operation::gen2;
use brydz_simulator::options::operation::demo_op::{test_sample_biased_deal_crossing, test_sample_biased_deal_single, test_sample_biased_distribution_parameters, DemoCommands};
use brydz_simulator::options::operation::sessions::GenericContractA2CSession;
use sztorm::agent::RandomPolicy;
use sztorm_rl::actor_critic::ActorCriticPolicy;
use sztorm_rl::torch_net::{A2CNet, NeuralNetCloner, TensorA2C};


//use crate::options::operation::{GenContract, Operation};
//mod error;
//mod options;
//mod error;


#[allow(dead_code)]
fn serialize_settings_toml(){
    let sim_conf = ContractConfig::new_raw(
        PlayerCfg::new(String::from_str("AQT32.JT94.76.QT").unwrap(), Local),
        PlayerCfg::new(String::from_str("J97.Q875.AQT94.K").unwrap(), Local),
        PlayerCfg::new(String::from_str("K8.AK32.82.J9532").unwrap(), Local),
        PlayerCfg::new(String::from_str("654.6.KJ53.A8764").unwrap(), Local),
        String::from_str("2S").unwrap(),


    );

    let toml = toml::to_string(&sim_conf).unwrap();
    println!("{}", toml);
}

fn main() -> Result<(), BrydzSimError> {

    let cli = options::Cli::parse();
    options::setup_logger(cli.log_level, &cli.log_file).unwrap();
    //serialize_settings_toml();
    match &cli.command{
        Operation::ContractGen(gen_options) => gen2(gen_options),

        Operation::LocalSimContract(options) => {
            sim2(options)
        }//sim2(options)}

        Operation::Train(train_params) => {
            let network_pattern  = NeuralNetCloner::new(|path|{
                let seq = nn::seq()
                    .add(nn::linear(path / "input", 420, 2048, Default::default()))
                    .add(nn::linear(path / "h1", 2048, 2048, Default::default()))
                    .add(nn::linear(path / "h2", 2048, 1024, Default::default()))
                    //.add(nn::linear(path / "h3", 1024, 512, Default::default()))
                ;
                let actor = nn::linear(path / "al", 1024, 52, Default::default());
                let critic = nn::linear(path / "cl", 1024, 1, Default::default());
                let device = path.device();

                {move |xs: &Tensor|{
                    let xs = xs.to_device(device).apply(&seq);
                    //(xs.apply(&critic), xs.apply(&actor))
                    TensorA2C{critic: xs.apply(&critic), actor: xs.apply(&actor)}
                }}
            });
            let declarer_net = A2CNet::new(VarStore::new(Device::Cpu), network_pattern.get_net_closure());
            let whist_net = A2CNet::new(VarStore::new(Device::Cpu), network_pattern.get_net_closure());
            let offside_net = A2CNet::new(VarStore::new(Device::Cpu), network_pattern.get_net_closure());
            let declarer_optimiser = declarer_net.build_optimizer(Adam::default(), 5e-5).unwrap();
            let whist_optimiser = whist_net.build_optimizer(Adam::default(), 5e-5).unwrap();
            let offside_optimiser = offside_net.build_optimizer(Adam::default(), 5e-5).unwrap();
            let declarer_policy: ActorCriticPolicy<ContractDP, ContractAgentInfoSetSimple, ContractInfoSetConvert420Normalised>  =
                ActorCriticPolicy::new(declarer_net, declarer_optimiser, ContractInfoSetConvert420Normalised {});
            let whist_policy: ActorCriticPolicy<ContractDP, ContractAgentInfoSetSimple, ContractInfoSetConvert420Normalised> =
                ActorCriticPolicy::new(whist_net, whist_optimiser, ContractInfoSetConvert420Normalised {});
            let offside_policy: ActorCriticPolicy<ContractDP, ContractAgentInfoSetSimple, ContractInfoSetConvert420Normalised> =
                ActorCriticPolicy::new(offside_net, offside_optimiser, ContractInfoSetConvert420Normalised {});
            let mut session = GenericContractA2CSession::new_rand_init(declarer_policy, whist_policy, offside_policy);

            let test_policy = RandomPolicy::<ContractDP, ContractAgentInfoSetAllKnowing>::new();
            session.train_all_at_once(1000, 512, 1000, None, &Default::default(), test_policy).unwrap();
            //train_session(train_params)

            /*train_session2_with_assumption::<ContractAgentInfoSetSimple>(
                train_params,
                &SequentialB::new(Box::new(|p | {
                    nn::seq().add(nn::linear(p/"i", CONTRACT_Q_INPUT_STATE_HIST_SPARSE, 1024, Default::default()))
                        .add(nn::linear(p/"h1", 1024, 1024, Default::default()))
                        .add(nn::linear(p/"h2", 1024, 1, Default::default()))
                    }                    )
                ))

             */
            Ok(())



        },


        Operation::Demo(command) => {
            match command{
                DemoCommands::Local =>{
                    options::operation::demo_op::tur_sim();
                    Ok(())
                }
                DemoCommands::Tcp => {
                    options::operation::demo_op::tur_sim_tcp();
                    Ok(())
                }
                DemoCommands::Generic => {
                    match options::operation::demo_op::test_generic_model(){
                        Ok(_) => Ok(()),
                        Err(e) => Err(BrydzSimError::Custom(format!("{e:}")))
                    }
                },
                DemoCommands::RunNN => {
                    options::operation::demo_op::test_with_untrained_network()?;
                    Ok(())
                },
                DemoCommands::BiasedParams => {
                    Ok(test_sample_biased_distribution_parameters()?)
                },
                DemoCommands::BiasedSample => {
                    test_sample_biased_deal_crossing()?;
                    test_sample_biased_deal_single()?;
                    Ok(())
                }
            }
        }
    }







    //
}
