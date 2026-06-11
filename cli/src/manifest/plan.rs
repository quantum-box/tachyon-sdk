use anyhow::Result;
use tachyon_sdk::apis::configuration::Configuration;

use super::{apply, ApplyArgs};

pub(crate) async fn run(args: &ApplyArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    apply::run(args, config, tenant_id, true, "plan").await
}
