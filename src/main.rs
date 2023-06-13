use brydz_simulator::settings::{ContractConfig, PlayerCfg};
use brydz_simulator::settings::Connection::Local;
use std::str::FromStr;


use clap::Parser;
use tch::nn;
use brydz_core::sztorm::state::ContractAgentInfoSetSimple;

use brydz_simulator::error::BrydzSimError;
use brydz_simulator::{CONTRACT_Q_INPUT_STATE_HIST_SPARSE, options};
use brydz_simulator::options::operation::{Operation, SequentialB, sim2, train_session, train_session2};
use brydz_simulator::options::operation::gen2;
use brydz_simulator::options::operation::test_op::{test_sample_biased_deal_crossing, test_sample_biased_deal_single, test_sample_biased_distribution_parameters, TestCommands};


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
            //train_session(train_params)

            train_session2::<ContractAgentInfoSetSimple>(
                train_params,
                &SequentialB::new(Box::new(|p | {
                    nn::seq().add(nn::linear(p/"i", CONTRACT_Q_INPUT_STATE_HIST_SPARSE, 1024, Default::default()))
                        .add(nn::linear(p/"h1", 1024, 1024, Default::default()))
                        .add(nn::linear(p/"h2", 1024, 1, Default::default()))
                    }                    )
                ))



        },


        Operation::Test(command) => {
            match command{
                TestCommands::Local =>{
                    options::operation::test_op::tur_sim();
                    Ok(())
                }
                TestCommands::Tcp => {
                    options::operation::test_op::tur_sim_tcp();
                    Ok(())
                }
                TestCommands::Generic => {
                    match options::operation::test_op::test_generic_model(){
                        Ok(_) => Ok(()),
                        Err(e) => Err(BrydzSimError::Custom(format!("{e:}")))
                    }
                },
                TestCommands::RunNN => {
                    options::operation::test_op::test_with_untrained_network()?;
                    Ok(())
                },
                TestCommands::BiasedParams => {
                    Ok(test_sample_biased_distribution_parameters()?)
                },
                TestCommands::BiasedSample => {
                    test_sample_biased_deal_crossing()?;
                    test_sample_biased_deal_single()?;
                    Ok(())
                }
            }
        }
    }







    //
}
