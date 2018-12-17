use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(subcommand)]
    pub cmd: Option<Command>
}

#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "AppSettings::InferSubcommands"))]
pub enum Command {
    #[structopt(name = "generate-secret", alias = "secret")]
    GenerateSecret,
    #[structopt(name = "test-config", alias = "test")]
    TestConfig
}