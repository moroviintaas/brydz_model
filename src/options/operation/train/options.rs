use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum TestPolicyChoice{
    RandomPolicy,
    InitLikeLearning,
}




#[derive(Args)]
pub struct TrainOptions{

    #[arg(short = 'd', long = "declarer_save", help = "Declarer VarStore save file")]
    pub declarer_save: Option<PathBuf>,
    #[arg(short = 'w', long = "whist_save", help = "Whist VarStore save file")]
    pub whist_save: Option<PathBuf>,
    #[arg(short = 'o', long = "offside_save", help = "Offside VarStore save file")]
    pub offside_save: Option<PathBuf>,

    #[arg(short = 'D', long = "declarer_load", help = "Declarer VarStore load file")]
    pub declarer_load: Option<PathBuf>,
    #[arg(short = 'W', long = "whist_load", help = "Whist VarStore load file")]
    pub whist_load: Option<PathBuf>,
    #[arg(short = 'O', long = "offside_load", help = "Offside VarStore load file")]
    pub offside_load: Option<PathBuf>,

    #[arg(short = 'e', long = "epochs", help = "Number of epochs", default_value = "10")]
    pub epochs: u32,

    #[arg(short = 'g', long = "games", help = "games iin epoch", default_value = "100")]
    pub games: u32,

    #[arg(short = 't', long = "tests", help = "test_set_number", default_value = "100")]
    pub tests_set_size: u32,

    #[arg(short = 'l', long = "layers", help = "Add hidden layers", default_value = "1024,512")]
    pub hidden_layers: Vec<u32>,

    #[arg(long = "separate", help = "Separate learning for different agents")]
    pub separate: bool


}