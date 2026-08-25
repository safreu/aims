use crate::{
    bootstrap::{
        accounts::build_accounts_state, devices::build_device_state,
        households::build_households_state, inventory::build_inventory_item_state,
        scanning::build_scanning_state,
    },
    config::AppConfig,
    shared::{api::AppState, db::create_pool},
};

mod accounts;
mod devices;
mod households;
mod inventory;
mod scanning;

pub async fn build_app_state(config: &AppConfig) -> Result<AppState, BootstrapError> {
    let pool = create_pool(&config.database).await?;

    let accounts = build_accounts_state(&pool, &config.session);
    let households = build_households_state(&pool);
    let inventory = build_inventory_item_state(&pool);
    let device = build_device_state(&pool);
    let scanning = build_scanning_state(&pool);

    Ok(AppState {
        accounts,
        households,
        inventory,
        device,
        scanning,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("Failed to initialize database connection pool")]
    Database(#[from] sqlx::Error),
}
