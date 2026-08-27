use std::sync::Arc;

use crate::{
    bootstrap::{
        accounts::build_accounts_state, devices::build_device_state,
        households::build_households_state, inventory::build_inventory_item_state,
        scanning::build_scanning_state, shopping::build_shopping_state,
    },
    config::AppConfig,
    modules::households::{
        adapters::BroadcastHouseholdEvents,
        ports::{HouseholdEventPublisher, HouseholdEventSubscriber},
    },
    shared::{api::AppState, db::create_pool},
};

mod accounts;
mod devices;
mod households;
mod inventory;
mod scanning;
mod shopping;

pub async fn build_app_state(config: &AppConfig) -> Result<AppState, BootstrapError> {
    let pool = create_pool(&config.database).await?;

    let household_events = Arc::new(BroadcastHouseholdEvents::new(64));
    let household_events_publisher: Arc<dyn HouseholdEventPublisher> = household_events.clone();
    let household_events_subscriber: Arc<dyn HouseholdEventSubscriber> = household_events.clone();

    let accounts = build_accounts_state(&pool, &config.session);
    let households = build_households_state(
        &pool,
        household_events_publisher.clone(),
        household_events_subscriber.clone(),
    );
    let inventory = build_inventory_item_state(&pool, household_events_publisher.clone());
    let device = build_device_state(&pool);
    let scanning = build_scanning_state(&pool, household_events_publisher.clone());
    let shopping = build_shopping_state(&pool, household_events_publisher.clone());

    Ok(AppState {
        accounts,
        households,
        inventory,
        device,
        scanning,
        shopping,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("Failed to initialize database connection pool")]
    Database(#[from] sqlx::Error),
}
