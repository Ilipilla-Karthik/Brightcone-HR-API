use crate::models::_entities::employee_assets::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct AssetParams {
    pub employee_code: String,
    pub asset_type: String,
    pub model_name: String,
    pub serial_number: String,
    pub status: Option<String>,
    pub assigned_date: Option<String>,
}

#[debug_handler]
pub async fn my_assets(State(ctx): State<AppContext>) -> Result<Response> {
    let mut list: Vec<Model> = Entity::find()
        .order_by_desc(employee_assets::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    // Auto-seed default assets in PostgreSQL if empty
    if list.is_empty() {
        let defaults = vec![
            (
                "Laptop",
                "Apple MacBook Pro 16\" (M3 Pro, 36GB, 1TB)",
                "C02G1234MD6R",
                "Active / Assigned",
                "2022-08-15",
            ),
            (
                "Monitor",
                "Dell UltraSharp 27\" 4K USB-C Hub (U2723QE)",
                "CN0942X",
                "Active / Assigned",
                "2022-08-16",
            ),
            (
                "Peripherals",
                "Jabra Evolve 75 Wireless Headset",
                "JB88491",
                "Active / Assigned",
                "2022-08-16",
            ),
        ];

        for (atype, model, serial, status, date) in defaults {
            let active = ActiveModel {
                asset_id: Set(Uuid::new_v4()),
                employee_code: Set("BC-1038".to_string()),
                asset_type: Set(atype.to_string()),
                model_name: Set(model.to_string()),
                serial_number: Set(serial.to_string()),
                status: Set(status.to_string()),
                assigned_date: Set(date.to_string()),
                ..Default::default()
            };
            let _ = active.insert(&ctx.db).await;
        }

        list = Entity::find()
            .order_by_desc(employee_assets::Column::CreatedAt)
            .all(&ctx.db)
            .await?;
    }

    format::json(list)
}

#[debug_handler]
pub async fn create_asset(
    State(ctx): State<AppContext>,
    Json(params): Json<AssetParams>,
) -> Result<Response> {
    let active = ActiveModel {
        asset_id: Set(Uuid::new_v4()),
        employee_code: Set(params.employee_code),
        asset_type: Set(params.asset_type),
        model_name: Set(params.model_name),
        serial_number: Set(params.serial_number),
        status: Set(params.status.unwrap_or_else(|| "Assigned".to_string())),
        assigned_date: Set(params
            .assigned_date
            .unwrap_or_else(|| "2026-10-16".to_string())),
        ..Default::default()
    };
    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/assets")
        .add("/my", get(my_assets))
        .add("/", get(my_assets))
        .add("/", post(create_asset))
}
