use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::thread;
use std::thread::{scope, spawn};
use rand::seq::SliceRandom;
use rand::thread_rng;
use brydz_core::bidding::Bid;
use brydz_core::cards::trump::Trump;
use brydz_core::deal::{Contract, RegDealStd};
use brydz_core::distribution::hand::BridgeHand;
use brydz_core::error::BridgeErrorStd;
use brydz_core::karty::cards::STANDARD_DECK;
use brydz_core::karty::suits::SuitStd::Spades;
use brydz_core::player::side::{Side, SideAssociated};
use brydz_core::player::side::Side::{East, North, South, West};
use brydz_core::player::situation::Situation;
use brydz_core::protocol::{ClientDealMessage, ServerDealMessage};
use brydz_core::world::agent::AutomaticAgent;
use brydz_core::world::ChannelDummy;
use brydz_core::world::comm::SyncComm;
use brydz_core::world::environment::{ChannelDealEnvironment, NoCardCheck, RoundRobinEnvironmentStd};
use brydz_core::world::environment::StagingEnvironment;
use brydz_network_extensions::tcp::speedy::TcpSpeedyForwarder;
use brydz_network_extensions::tokio;
use karty::cards::CardStd;
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

fn basic_sim_with_bot2(){
    let contract = Contract::new(Side::East, Bid::init(Trump::Colored(Spades), 2).unwrap());
    let deal = RegDealStd::new(contract.clone());
    //let mut simple_overseer = SimpleOverseer::new(deal);
    let mut simple_overseer = ChannelDealEnvironment::<NoCardCheck>::new(deal);
    let (n_tx, n_rx) = simple_overseer.create_connection(&North);
    let (s_tx, s_rx) = simple_overseer.create_connection(&South);
    let (e_tx, e_rx) = simple_overseer.create_connection(&East);
    let (w_tx, w_rx) = simple_overseer.create_connection(&West);

    let mut card_supply = Vec::from(STANDARD_DECK);
    let hand_east = BridgeHand::init(&mut card_supply).unwrap();
    let hand_south = BridgeHand::init(&mut card_supply).unwrap();
    let hand_west = BridgeHand::init(&mut card_supply).unwrap();
    let hand_north = BridgeHand::init(&mut card_supply).unwrap();

    let mut bot_east = brydz_bot_random::declarer::DeclarerOverChannel::new(e_tx, e_rx, Situation::new(Side::East, hand_east, contract.clone()));
    let mut bot_south = brydz_bot_random::defender::DefenderOverChannel::new(s_tx, s_rx, Situation::new(Side::South, hand_south, contract.clone()));
    let mut bot_west = ChannelDummy::new(w_tx, w_rx, Situation::new(Side::West, hand_west, contract.clone()));
    let mut bot_north = brydz_bot_random::defender::DefenderOverChannel::new(n_tx, n_rx, Situation::new(Side::North, hand_north, contract));



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

fn basic_sim_with_bot3(){
    let contract = Contract::new(Side::East, Bid::init(Trump::Colored(Spades), 2).unwrap());
    let deal = RegDealStd::new(contract.clone());
    //let mut simple_overseer = SimpleOverseer::new(deal);
    let (comm_env_north, comm_north) = SyncComm::<ServerDealMessage, ClientDealMessage, BridgeErrorStd>::new_pair();
    let (comm_env_east, comm_east) = SyncComm::<ServerDealMessage, ClientDealMessage, BridgeErrorStd>::new_pair();
    let (comm_env_west, comm_west) = SyncComm::<ServerDealMessage, ClientDealMessage, BridgeErrorStd>::new_pair();
    let (comm_env_south, comm_south) = SyncComm::<ServerDealMessage, ClientDealMessage, BridgeErrorStd>::new_pair();

    let comm_assotiation = SideAssociated::new(comm_env_north, comm_env_east, comm_env_south, comm_env_west);


    let mut simple_overseer = RoundRobinEnvironmentStd::new(comm_assotiation, deal, NoCardCheck::default());
    /*
    let (n_tx, n_rx) = simple_overseer.create_connection(&North);
    let (s_tx, s_rx) = simple_overseer.create_connection(&South);
    let (e_tx, e_rx) = simple_overseer.create_connection(&East);
    let (w_tx, w_rx) = simple_overseer.create_connection(&West);
    */
    let (n_tx, n_rx) = comm_north._decompose();
    let (s_tx, s_rx) = comm_south._decompose();
    let (e_tx, e_rx) = comm_east._decompose();
    let (w_tx, w_rx) = comm_west._decompose();

    let mut card_supply = Vec::from(STANDARD_DECK);
    let hand_east = BridgeHand::init(&mut card_supply).unwrap();
    let hand_south = BridgeHand::init(&mut card_supply).unwrap();
    let hand_west = BridgeHand::init(&mut card_supply).unwrap();
    let hand_north = BridgeHand::init(&mut card_supply).unwrap();

    let mut bot_east = brydz_bot_random::declarer::DeclarerOverChannel::new(e_tx, e_rx, Situation::new(Side::East, hand_east, contract.clone()));
    let mut bot_south = brydz_bot_random::defender::DefenderOverChannel::new(s_tx, s_rx, Situation::new(Side::South, hand_south, contract.clone()));
    let mut bot_west = ChannelDummy::new(w_tx, w_rx, Situation::new(Side::West, hand_west, contract.clone()));
    let mut bot_north = brydz_bot_random::defender::DefenderOverChannel::new(n_tx, n_rx, Situation::new(Side::North, hand_north, contract));



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
#[allow(dead_code)]
async fn basic_sim_with_bot_tcp() {
    let contract = Contract::new(Side::East, Bid::init(Trump::Colored(Spades), 2).unwrap());
    let deal = RegDealStd::new(contract.clone());
    //let mut simple_overseer = SimpleOverseer::new(deal);
    let mut simple_overseer = ChannelDealEnvironment::<NoCardCheck>::new(deal);
    //let (n_tx, n_rx) = simple_overseer.create_connection(&North);
    let (s_tx, s_rx) = simple_overseer.create_connection(&South);
    let (e_tx, e_rx) = simple_overseer.create_connection(&East);
    let (w_tx, w_rx) = simple_overseer.create_connection(&West);

    let mut card_supply = Vec::from(STANDARD_DECK);
    let hand_east = BridgeHand::init(&mut card_supply).unwrap();
    let hand_south = BridgeHand::init(&mut card_supply).unwrap();
    let hand_west = BridgeHand::init(&mut card_supply).unwrap();
    let hand_north = BridgeHand::init(&mut card_supply).unwrap();

    let mut bot_east = brydz_bot_random::declarer::DeclarerOverChannel::new(e_tx, e_rx, Situation::new(Side::East, hand_east, contract.clone()));
    let mut bot_south = brydz_bot_random::defender::DefenderOverChannel::new(s_tx, s_rx, Situation::new(Side::South, hand_south, contract.clone()));
    let mut bot_west = ChannelDummy::new(w_tx, w_rx, Situation::new(Side::West, hand_west, contract.clone()));
    //let mut bot_north = brydz_bot_random::defender::DefenderOverChannel::new(n_tx, n_rx, Situation::new(Side::North, hand_north, contract));



    //let listener_env = tokio::net::TcpListener::bind("127.0.0.1:8069").await.unwrap();
    let stream_north = tokio::net::TcpStream::connect("127.0.0.1:8069").await.unwrap();
    //let (stream_env, stream_addr) = listener_env.accept().await.unwrap();

    let (north_client_tx_connector, north_client_rx) = channel();
    let (north_client_tx, north_client_rx_connector) = channel();


    let connector_north_env = TcpSpeedyForwarder::new(north_client_rx_connector, north_client_tx_connector, stream_north);

    //let cen = connector_env_north.run().await.unwrap();
    //let cne = connector_north_env.run().await.unwrap();
    //let cen = connector_env_north.run();
    let cne = connector_north_env.run();

    /*let h_cen = tokio::spawn( async move {
        let connector_env_north = TcpSpeedyForwarder::new(n_rx, n_tx, stream_env);
        connector_env_north.run().await;
    });

     */

    let mut bot_north = brydz_bot_random::defender::DefenderOverChannel::new(north_client_tx, north_client_rx, Situation::new(Side::North, hand_north, contract));






    thread::scope(|s| {
        s.spawn(|| {
            simple_overseer.run().unwrap();
            //println!("{:?}", x);
        });
        s.spawn(|| {
            bot_east.run().unwrap();
        });
        s.spawn(|| {
            bot_south.run().unwrap();
            //error!("South result: {:?}", &rs);
        });
        s.spawn(|| {
            bot_west.run().unwrap();
        });
        s.spawn(|| {
            bot_north.run().unwrap();
        });
    });

    //cen.await.unwrap();
    cne.await.unwrap();
}



#[allow(dead_code)]
fn test_tcp(){
    let tcp_listenr = TcpListener::bind("0.0.0.0:8420").unwrap();

    scope(|scope| {
        scope.spawn(||{
            for stream in tcp_listenr.incoming(){
                match stream{
                    Ok(mut st) => {
                        spawn(move ||{
                            let mut buffer = [0u8;16];
                            println!("Listener, came connection: {:?}, timeout is: {:?}", &st, &st.read_timeout());
                            loop{
                                match st.read(&mut buffer){
                                    Ok(_) => {
                                        println!("Listener received: {:?}.", buffer);
                                        match CardStd::read_from_buffer(&buffer){
                                            Ok(card) => {
                                                println!("Listenr card: {:#}", card);
                                            }
                                            Err(_) => {println!("Failed parsing card")}
                                        }
                                        st.write_all(&[1u8;1]).unwrap();
                                    }
                                    Err(e) => {
                                        println!("Failed reading: {:?}", e);
                                    }
                                }

                            }
                        } );
                    }
                    Err(e) => {println!("{:?}", e)}
                }
            }
        });

        scope.spawn(||{
            let mut connection = TcpStream::connect("127.0.0.1:8420").unwrap();
            loop{
                let mut rng = thread_rng();
                let mut buffer = [0u8;64];
                let mut send_buffer = [0u8; 16];
                let random_card = STANDARD_DECK.choose(&mut rng).unwrap();
                println!("Client random card: {:#}", random_card);
                match random_card.write_to_buffer(&mut send_buffer){
                    Ok(_) => {
                        println!("Serialized: {:?}", send_buffer);
                    },
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
                connection.write_all(&send_buffer).unwrap();
                match connection.read(&mut buffer){
                    Ok(_) => {
                        println!("Cliente received {:?} from server.", &buffer);

                    }
                    Err(e) => {
                        println!("Client failed receiving froms erver: {:?}", e)
                    }
                }
            }

        })
        ;

    })
}



fn main(){
    setup_logger().unwrap();
    
    println!("Hello!");
    basic_sim_with_bot2();
    basic_sim_with_bot3();
    //let rt = tokio::runtime::Runtime::new().unwrap();
    //rt.block_on(async {basic_sim_with_bot_tcp().await});
    //test_tcp();
}