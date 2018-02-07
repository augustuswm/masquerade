use env::Env;

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    key: String,
    envs: Vec<Env>,
}
