use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::thread::{scope};
use brydz_framework::brydz_core::deal::fair_bridge_deal;
use brydz_framework::error::comm::CommError;
use brydz_framework::world::agent::bot::{SimpleBot2Phase, SimpleRandomBot2PhaseStd, DummyBotPhase2Std};
use brydz_framework::world::agent::state::AgentStatePhase2Std;
use brydz_framework::world::environment::{RoundRobinContractEnvStd, EnvStatePhase2Std};
use brydz_network_extensions::tcp::speedy::{TcpComm};
use log::{debug, info};

use brydz_framework::brydz_core::bidding::Bid;
use brydz_framework::brydz_core::cards::trump::Trump;
use brydz_framework::brydz_core::contract::{ContractSpec, ContractStd};
use brydz_framework::brydz_core::deal::hand::{HandVector, StackHandStd};
use brydz_framework::brydz_core::karty::cards::STANDARD_DECK;
use brydz_framework::brydz_core::karty::suits::SuitStd::Spades;
use brydz_framework::brydz_core::player::side::{Side, SideAssociated};
use brydz_framework::brydz_core::player::situation::Situation;
use brydz_framework::error::BridgeErrorStd;
use brydz_framework::protocol::{ClientDealMessage, ServerDealMessage, DealAction, ServerDealMessageStd, ClientDealMessageStd};
use brydz_framework::world::agent::{AutomaticAgentOld, AutomaticAgentPhase2};
use brydz_framework::world::comm::{SyncComm, TokioComm};
use karty::cards::{ KING_HEARTS};
use karty::speedy::{Writable, Readable};

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
/*
#[allow(dead_code)]
fn basic_sim_with_bot3(){
    let contract = ContractSpec::new(Side::East, Bid::init(Trump::Colored(Spades), 2).unwrap());
    let deal = ContractStd::new(contract.clone());
    //let mut simple_overseer = SimpleOverseer::new(contract);
    let (comm_env_north, comm_north) = SyncComm::<ServerDealMessage, ClientDealMessage, BridgeErrorStd>::new_pair();
    let (comm_env_east, comm_east) = SyncComm::<ServerDealMessage, ClientDealMessage, BridgeErrorStd>::new_pair();
    let (comm_env_west, comm_west) = SyncComm::<ServerDealMessage, ClientDealMessage, BridgeErrorStd>::new_pair();
    let (comm_env_south, comm_south) = SyncComm::<ServerDealMessage, ClientDealMessage, BridgeErrorStd>::new_pair();

    let comm_assotiation = SideAssociated::new(comm_env_north, comm_env_east, comm_env_south, comm_env_west);


    let mut simple_overseer = RoundRobinDealEnvironment::new(comm_assotiation, deal, NoCardCheck::default());

    //let (n_tx, n_rx) = comm_north._decompose();
    //let (s_tx, s_rx) = comm_south._decompose();
    //let (e_tx, e_rx) = comm_east._decompose();
    //let (w_tx, w_rx) = comm_west._decompose();

    let mut card_supply = Vec::from(STANDARD_DECK);
    let hand_east = HandVector::drain_full_from_vec(&mut card_supply).unwrap();
    let hand_south = HandVector::drain_full_from_vec(&mut card_supply).unwrap();
    let hand_west = HandVector::drain_full_from_vec(&mut card_supply).unwrap();
    let hand_north = HandVector::drain_full_from_vec(&mut card_supply).unwrap();

    //let card_deal = fair_bridge_deal::<StackHandStd>();
    //let (hand_north, hand_east, hand_south, hand_west) = card_deal.destruct();
    
    


    //let mut bot_east = brydz_bot_random::declarer::DeclarerOverChannel::new(e_tx, e_rx, Situation::new(Side::East, hand_east, contract.clone()));

    let mut bot_east = brydz_bot_random::declarer::DeclarerBot::new(comm_east, Situation::new(Side::East, hand_east, contract.clone()));
    //let mut bot_south = brydz_bot_random::defender::DefenderOverChannel::new(s_tx, s_rx, Situation::new(Side::South, hand_south, contract.clone()));
    let mut bot_south = brydz_bot_random::defender::DefenderBot::new(comm_south, Situation::new(Side::South, hand_south, contract.clone()));
    let mut bot_west = DummyBot::new(comm_west, Situation::new(Side::West, hand_west, contract.clone()));
    //let mut bot_north = brydz_bot_random::defender::DefenderOverChannel::new(n_tx, n_rx, Situation::new(Side::North, hand_north, contract));
    let mut bot_north = brydz_bot_random::defender::DefenderBot::new(comm_north, Situation::new(Side::North, hand_north, contract));



    thread::scope(|s|{
        s.spawn(||{
           simple_overseer.run().unwrap();
            //println!("{:?}", x);
        });
        s.spawn(||{
            bot_east.run().unwrap();
        });
        s.spawn(||{
            bot_south.run().unwrap();
            //error!("South result: {:?}", &rs);
        });
        s.spawn(||{
            bot_west.run().unwrap();
        });
        s.spawn(||{
            bot_north.run().unwrap();
        });
    })


}

fn basic_sim_with_bot_tcp(){
    let contract = ContractSpec::new(Side::East, Bid::init(Trump::Colored(Spades), 2).unwrap());
    let deal = ContractStd::new(contract.clone());
    //let mut simple_overseer = SimpleOverseer::new(contract);


   let tcp_listener = std::net::TcpListener::bind("127.0.0.1:8420").unwrap();
    



    let mut card_supply = Vec::from(STANDARD_DECK);
    let hand_east = HandVector::drain_full_from_vec(&mut card_supply).unwrap();
    let hand_south = HandVector::drain_full_from_vec(&mut card_supply).unwrap();
    let hand_west = HandVector::drain_full_from_vec(&mut card_supply).unwrap();
    let hand_north = HandVector::drain_full_from_vec(&mut card_supply).unwrap();

    //let mut bot_east = brydz_bot_random::declarer::DeclarerOverChannel::new(e_tx, e_rx, Situation::new(Side::East, hand_east, contract.clone()));


    

    thread::scope(|s|{

        s.spawn(||{
            let (north_stream, _) = tcp_listener.accept().unwrap();
            info!("North connected");
            let (east_stream, _) = tcp_listener.accept().unwrap();
            info!("East connected");
            let (south_stream, _) = tcp_listener.accept().unwrap();
            info!("South connected");
            let (west_stream, _) = tcp_listener.accept().unwrap();
            info!("West connected");
            let comm_assotiation = SideAssociated::new(TcpComm::new(north_stream), TcpComm::new(east_stream), TcpComm::new(south_stream), TcpComm::new(west_stream));
        
            let mut simple_overseer = RoundRobinDealEnvironment::new(comm_assotiation, deal, NoCardCheck::default());
        
        
            simple_overseer.run().unwrap();
        });
             
        
            
        
        
        
        s.spawn(||{
            let stream_north_c = std::net::TcpStream::connect("127.0.0.1:8420").unwrap();
            info!("North connected (client)");
            let stream_east_c = std::net::TcpStream::connect("127.0.0.1:8420").unwrap();
            info!("East connected (client)");
            let stream_south_c = std::net::TcpStream::connect("127.0.0.1:8420").unwrap();
            info!("South connected (client)");
            let stream_west_c = std::net::TcpStream::connect("127.0.0.1:8420").unwrap();
            info!("West connected (client)");

            let comm_north = TcpComm::new(stream_north_c);
            let comm_east = TcpComm::new(stream_east_c);
            let comm_south = TcpComm::new(stream_south_c);
            let comm_west = TcpComm::new(stream_west_c);

            let mut bot_north = brydz_bot_random::defender::DefenderBot::new(comm_north, Situation::new(Side::North, hand_north, contract.clone()));
            let mut bot_east = brydz_bot_random::declarer::DeclarerBot::new(comm_east, Situation::new(Side::East, hand_east, contract.clone()));
            let mut bot_south = brydz_bot_random::defender::DefenderBot::new(comm_south, Situation::new(Side::South, hand_south, contract.clone()));
            let mut bot_west = DummyBot::new(comm_west, Situation::new(Side::West, hand_west, contract.clone()));

            
            thread::scope(|s2|{
                
                s2.spawn(||{
                    bot_north.run().unwrap();
                });
                s2.spawn(||{
                    bot_south.run().unwrap();
                });
                s2.spawn(||{
                    bot_west.run().unwrap();
                });
                s2.spawn(||{
                    bot_east.run().unwrap();
                });
            });
            
            
            
        });
        
    })


}

#[allow(dead_code)]
fn test_std_tcp(){
    let tcp_listener = TcpListener::bind("127.0.0.1:8420").unwrap();
    
    
    //let mut north_stream_client = TcpStream::connect("127.0.0.1:8420").unwrap();
    scope(|scop|{
        scop.spawn(||{
            let (mut north_stream_srv, _) = tcp_listener.accept().unwrap();
            //let (mut east_stream_srv, _) = tcp_listener.accept().unwrap();
            
            let mut buffer = [0u8;256];
            loop{
                match north_stream_srv.read(&mut buffer){
                    Ok(0) => {},
                    Ok(n) => {
                        debug!("Received data from north, bytes: {:?}", n);
                        let cm = ClientDealMessage::read_from_buffer(&buffer).unwrap();
                        println!("Message from client: {:?}", &cm);
                    }
                    Err(e) => {
                        debug!("Error receiving from north: {:?}", e);
                    },
                }
            }
            
        });
        scop.spawn(||{
            let mut north_stream_client = TcpStream::connect("127.0.0.1:8420").unwrap();
            let msg = ClientDealMessage::Action(DealAction::PlayCard(KING_HEARTS));
            let mut buffer = [0u8;25623];
            msg.write_to_buffer(&mut buffer).unwrap();
            north_stream_client.write_all(&buffer).unwrap();
            

        });
    });

}
*/

#[allow(dead_code)]
fn basic_sim_with_bot(){
    let contract = ContractSpec::new(Side::East, Bid::init(Trump::Colored(Spades), 2).unwrap());
    let deal = ContractStd::new(contract.clone());
    //let mut simple_overseer = SimpleOverseer::new(contract);
    let (comm_env_north, comm_north) = SyncComm::<ServerDealMessageStd, ClientDealMessageStd, CommError>::new_pair();
    let (comm_env_east, comm_east) = SyncComm::<ServerDealMessageStd, ClientDealMessageStd, CommError>::new_pair();
    let (comm_env_west, comm_west) = SyncComm::<ServerDealMessageStd, ClientDealMessageStd, CommError>::new_pair();
    let (comm_env_south, comm_south) = SyncComm::<ServerDealMessageStd, ClientDealMessageStd, CommError>::new_pair();

    let comm_assotiation = SideAssociated::new(comm_env_north, comm_env_east, comm_env_south, comm_env_west);


    let initial_contract = ContractStd::new(contract);
    let mut simple_overseer = RoundRobinContractEnvStd::new(
        comm_assotiation, 
        EnvStatePhase2Std::new(initial_contract.clone()));

    //let (n_tx, n_rx) = comm_north._decompose();
    //let (s_tx, s_rx) = comm_south._decompose();
    //let (e_tx, e_rx) = comm_east._decompose();
    //let (w_tx, w_rx) = comm_west._decompose();

    //let mut card_supply = Vec::from(STANDARD_DECK);
    //let hand_east = HandVector::drain_full_from_vec(&mut card_supply).unwrap();
    //let hand_south = HandVector::drain_full_from_vec(&mut card_supply).unwrap();
    //let hand_west = HandVector::drain_full_from_vec(&mut card_supply).unwrap();
    //let hand_north = HandVector::drain_full_from_vec(&mut card_supply).unwrap();

    let card_deal = fair_bridge_deal::<StackHandStd>();
    let (hand_north, hand_east, hand_south, hand_west) = card_deal.destruct();
    
    let mut bot_east = SimpleRandomBot2PhaseStd::new
    (AgentStatePhase2Std::new(
        Side::East, 
        hand_east, 
        initial_contract.clone()), comm_east);
    let mut bot_west = DummyBotPhase2Std::new
    (AgentStatePhase2Std::new(
        Side::West, 
        hand_west, 
        initial_contract.clone()), comm_west);
    let mut bot_north = SimpleRandomBot2PhaseStd::new
    (AgentStatePhase2Std::new(
        Side::North, 
        hand_north, 
        initial_contract.clone()), comm_north);
    let mut bot_south = SimpleRandomBot2PhaseStd::new
    (AgentStatePhase2Std::new(
        Side::South, 
        hand_south, 
        initial_contract.clone()), comm_south);
    //let mut bot_south = brydz_bot_random::defender::DefenderBot::new(comm_south, Situation::new(Side::South, hand_south, contract.clone()));
    //let mut bot_west = DummyBot::new(comm_west, Situation::new(Side::West, hand_west, contract.clone()));
    //let mut bot_north = brydz_bot_random::defender::DefenderBot::new(comm_north, Situation::new(Side::North, hand_north, contract));


    //bot_south.run().unwrap();

    thread::scope(|s|{
        
        s.spawn(||{
           simple_overseer.run().unwrap();
            //println!("{:?}", x);
        });
        s.spawn(||{
            bot_south.run().unwrap();
            //error!("South result: {:?}", &rs);
        });
        
        
        s.spawn(||{
            bot_east.run().unwrap();
        });
        
        
        
        s.spawn(||{
            bot_west.run().unwrap();
        });
        s.spawn(||{
            bot_north.run().unwrap();
        });
    })


}

fn main(){
    setup_logger().unwrap();
    
    println!("Hello!");
    //basic_sim_with_bot2();
    //basic_sim_with_bot3();
    //basic_sim_with_bot_tokio();
    //basic_sim_with_bot_tokio_tcp();
    //basic_sim_with_box();
    //let rt = tokio::runtime::Runtime::new().unwrap();
    //rt.block_on(async {basic_sim_with_bot_tcp().await});
    //test_tcp();
    //test_std_tcp();
    //basic_sim_with_bot_tcp();
    //basic_sim_with_bot3();
    basic_sim_with_bot();
}