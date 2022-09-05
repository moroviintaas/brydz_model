use std::thread;
use bridge_core::bidding::Bid;
use bridge_core::cards::trump::Trump;
use bridge_core::deal::{Contract, RegDealStd};
use bridge_core::distribution::hand::BridgeHand;
use bridge_core::karty::cards::STANDARD_DECK;
use bridge_core::karty::suits::SuitStd::Spades;
use bridge_core::overseer::{Overseer, SimpleOverseer};
use bridge_core::player::side::Side;
use bridge_core::player::side::Side::{East, North, South, West};
use bridge_core::player::situation::Situation;
use bridge_core::protocol::ClientMessage;
use karty_bridge_bot_random::Bot;

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
#[allow(dead_code)]
fn basic_sim(){
    let deal = RegDealStd::new(Contract::new(Side::East, Bid::init(Trump::Colored(Spades), 2).unwrap()));
    let mut simple_overseer = SimpleOverseer::new(deal);
    let (n_tx, _n_rx) = simple_overseer.create_connection(&North);
    let (s_tx, _s_rx) = simple_overseer.create_connection(&South);
    let (e_tx, _e_rx) = simple_overseer.create_connection(&East);
    let (w_tx, _w_rx) = simple_overseer.create_connection(&West);
    println!("{}", simple_overseer.are_players_ready());
    thread::scope(|s|{
        s.spawn(||{
           let x =simple_overseer.wait_for_readiness_rr();
            println!("{:?}", x);
        });
        n_tx.send(ClientMessage::Ready).unwrap();
        s_tx.send(ClientMessage::Ready).unwrap();
        e_tx.send(ClientMessage::Ready).unwrap();
        w_tx.send(ClientMessage::Ready).unwrap();
    });
    println!("{}", simple_overseer.are_players_ready());
}

fn basic_sim_with_bot(){
    let contract = Contract::new(Side::East, Bid::init(Trump::Colored(Spades), 2).unwrap());
    let deal = RegDealStd::new(contract.clone());
    let mut simple_overseer = SimpleOverseer::new(deal);
    let (n_tx, n_rx) = simple_overseer.create_connection(&North);
    let (s_tx, s_rx) = simple_overseer.create_connection(&South);
    let (e_tx, e_rx) = simple_overseer.create_connection(&East);
    let (w_tx, w_rx) = simple_overseer.create_connection(&West);

    let mut card_supply = Vec::from(STANDARD_DECK);
    let hand_east = BridgeHand::init(&mut card_supply).unwrap();
    let hand_south = BridgeHand::init(&mut card_supply).unwrap();
    let hand_west = BridgeHand::init(&mut card_supply).unwrap();
    let hand_north = BridgeHand::init(&mut card_supply).unwrap();

    let mut bot_east = karty_bridge_bot_random::declarer::DeclarerOverChannel::new(e_tx, e_rx, Situation::new(Side::East, hand_east, contract.clone()));
    let mut bot_south = karty_bridge_bot_random::defender::DefenderOverChannel::new(s_tx, s_rx, Situation::new(Side::South, hand_south, contract.clone()));
    let mut bot_west = karty_bridge_bot_random::dummy::DummyOverChannel::new(w_tx, w_rx, Situation::new(Side::West, hand_west, contract.clone()));
    let mut bot_north = karty_bridge_bot_random::defender::DefenderOverChannel::new(n_tx, n_rx, Situation::new(Side::North, hand_north, contract));



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

fn main(){
    setup_logger().unwrap();
    
    println!("Hello!");
    basic_sim_with_bot();
}