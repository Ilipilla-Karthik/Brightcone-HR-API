use crate::models::_entities::departments::{self, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, QueryOrder};

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_asc(departments::Column::Name)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/departments")
        .add("/", get(list))
}
