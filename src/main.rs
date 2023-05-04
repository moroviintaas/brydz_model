use brydz_simulator::settings::{ContractConfig, PlayerCfg};
use brydz_simulator::settings::Connection::Local;
use std::str::FromStr;


use clap::Parser;

use brydz_simulator::error::BrydzSimError;
use brydz_simulator::options;
use brydz_simulator::options::operation::{Operation, sim2};
use brydz_simulator::options::operation::gen2;



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

fn main() {

    let cli = options::Cli::parse();
    options::setup_logger(cli.log_level).unwrap();
    //serialize_settings_toml();
    let result = match &cli.command{
        Operation::ContractGen(gen_options) => gen2(gen_options),
        /* {
            let my_config = PrettyConfig::new()
                .depth_limit(4)
                // definitely superior (okay, just joking)
                .indentor("\t".to_owned());
            let contracts = generate_contracts(gen_options).unwrap();

            match &gen_options.output_file{
                None => {
                    println!("{}", to_string_pretty(&contracts, my_config).unwrap())
                }
                Some(file) => {
                    let mut output = std::fs::File::create(file).unwrap();
                    write!(output, "{}", to_string_pretty(&contracts, my_config).unwrap()).unwrap()
                }
            };

            //println!("{}", toml::to_string(&contracts).unwrap());
        }*/
        Operation::LocalSimContract(options) => {
            sim2(options)
        }//sim2(options)}
        Operation::TestLocal =>{
            options::operation::test_ops::tur_sim();
            Ok(())
        }
        Operation::TestTcp => {
            options::operation::test_ops::tur_sim_tcp();
            Ok(())
        }
        Operation::TestGeneric => {
            match options::operation::test_ops::test_generic_model(){
                Ok(_) => Ok(()),
                Err(e) => Err(BrydzSimError::Custom(format!("{e:}")))
            }
        }

    };
    result.unwrap()







    //
}
