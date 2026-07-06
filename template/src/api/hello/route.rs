use serde::{Deserialize, Serialize};
use threadloom::server;

#[derive(Serialize, Deserialize)]
pub struct HelloArgs {
    pub name: String,
}

#[server]
pub async fn hello(args: HelloArgs) -> Result<String, String> {
    if args.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    Ok(format!("Hello {} from Type-Safe RPC!", args.name))
}
