use crate::config::app::REDIS_PUB_SUB_CHANNEL;
use crate::config::state::AppState;
use crate::entity::{cedar_policy_set, cedar_schema, template_links};
use crate::errors::app_error::AppError;
use cedar_policy::{Policy, PolicyId, PolicySet, Schema, Template};
use futures_util::StreamExt as _;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::str::FromStr;
use std::sync::Arc;
use crate::schemas::cedar_policy::TemplateLinkRecord;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

const MAX_RETRY_ATTEMPTS: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(5);
const CONNECTION_RETRY_DELAY: Duration = Duration::from_secs(10);

pub fn default_page() -> u64 {
    1
}

pub fn default_page_size() -> u64 {
    10
}

pub fn default_true() -> bool {
    true
}

pub fn default_false() -> bool {
    false
}

pub fn default_number_1() -> i8 {
    1
}

// Background task: listen for policy update notifications on Redis Pub/Sub
pub async fn subscribe_to_policy_updates(state: AppState) {
    let mut consecutive_failures = 0;

    loop {
        info!("Subscribing to Redis channel '{}'...", REDIS_PUB_SUB_CHANNEL);

        match establish_subscription(&state).await {
            Ok(()) => {
                consecutive_failures = 0;
                info!("Redis subscription ended, reconnecting...");
            }
            Err(e) => {
                consecutive_failures += 1;
                error!(
                    "Redis subscription failed (attempt {}/{}): {}",
                    consecutive_failures, MAX_RETRY_ATTEMPTS, e
                );

                if consecutive_failures >= MAX_RETRY_ATTEMPTS {
                    error!("Too many consecutive failures, backing off");
                    sleep(CONNECTION_RETRY_DELAY * consecutive_failures).await;
                } else {
                    sleep(CONNECTION_RETRY_DELAY).await;
                }
            }
        }
    }
}

async fn establish_subscription(state: &AppState) -> Result<(), AppError> {
    let pub_sub = state.redis.get_async_pubsub().await?;

    let (mut sink, mut stream) = pub_sub.split();

    // Subscribe to the channel
    sink.subscribe(REDIS_PUB_SUB_CHANNEL).await?;

    info!("Subscribed to Redis channel, waiting for notifications...");

    // Message loop
    while let Some(msg) = stream.next().await {
        let payload: String = match msg.get_payload() {
            Ok(p) => p,
            Err(e) => {
                warn!("Cannot read payload from Redis message: {}, skipping this update.", e);
                continue; // skip this one, move on to the next
            }
        };

        if payload.is_empty() {
            warn!("Empty payload, skipping this update.");
            continue;
        }

        info!("Policy update notification: '{}', reloading...", payload);

        if let Err(e) = reload_policies_and_schema(state).await {
            error!("Failed to reload policies: {}", e);
            // Keep listening; one failed reload shouldn't stop us
        }
    }

    warn!("Redis message stream ended");
    Ok(())
}

pub async fn reload_policies_and_schema(state: &AppState) -> Result<(), AppError> {
    info!("Reloading all Cedar policies, templates, links and schema from the database...");

    let (policies_result, schema_result, links_result) = tokio::join!(
        load_active_policies_and_templates(state.db.clone()),
        load_active_schema(state.db.clone()),
        load_all_template_links(state.db.clone())
    );

    let new_policies = policies_result?;
    let new_schema = schema_result?;
    let new_links = links_result?;

    state.auth_service.update_schema(new_schema).await;

    state
        .auth_service
        .update_policies_and_templates_in_cache(&new_policies)
        .await?;
    state
        .auth_service
        .update_template_link_records_in_cache(&new_links)
        .await?;

    Ok(())
}

pub async fn load_active_schema(db: Arc<DatabaseConnection>) -> Result<Schema, AppError> {
    info!("Loading the active Cedar schema from the database...");
    let active_schema_model = cedar_schema::Entity::find().one(db.as_ref()).await?;

    match active_schema_model {
        Some(model) => {
            info!("Found schema (ID: {}). Parsing...", model.schema_id);
            let (schema, warning) = Schema::from_cedarschema_str(model.schema.as_str())?;
            warning.for_each(|w| {
                warn!("Warning: {}", w);
            });
            Ok(schema)
        }
        None => {
            warn!("No active schema found, starting with an empty one");
            return Ok(Schema::from_json_value(serde_json::json!({}))?);
            // Err(AppError::SchemaError("CRITICAL: No active schema found in the database!".to_string()))
        }
    }
}

pub async fn load_active_policies_and_templates(
    db: Arc<DatabaseConnection>,
) -> Result<PolicySet, AppError> {
    info!("Loading active Cedar policies and templates from the database...");
    let active_policy_models = cedar_policy_set::Entity::find()
        .filter(cedar_policy_set::Column::IsActive.eq(true))
        .all(db.as_ref())
        .await?;

    if active_policy_models.is_empty() {
        warn!("No active policies or templates found, starting with an empty set.");
        return Ok(PolicySet::new());
    }
    info!(
        "Reloaded and cached {} policies/templates",
        active_policy_models.len()
    );
    let mut policy_set = PolicySet::new();

    for model in active_policy_models {
        let policy_id = PolicyId::from_str(model.policy_uuid.as_str())?;
        if let Ok(template) = Template::parse(Some(policy_id.clone()), &model.policy_text) {
            policy_set.add_template(template)?;
        } else {
            let policy = Policy::parse(Some(policy_id), &model.policy_text.as_str())?;
            policy_set.add(policy)?;
        }
    }

    Ok(policy_set)
}

pub async fn load_all_template_links(
    db: Arc<DatabaseConnection>,
) -> Result<Vec<TemplateLinkRecord>, AppError> {
    info!("Loading all template links from the database...");
    let link_models = template_links::Entity::find().all(db.as_ref()).await?;

    let link_records = link_models
        .into_iter()
        .map(|model| {
            Ok(TemplateLinkRecord {
                link_uuid: model.link_uuid.parse()?,
                template_uuid: model.template_uuid.parse()?,
                principal_uid: model.principal_uid.parse()?,
                resource_uid: model.resource_uid.parse()?,
            })
        })
        .collect::<Result<Vec<TemplateLinkRecord>, AppError>>()?;
    info!("Reloaded and cached {} template links", link_records.len());
    Ok(link_records)
}
