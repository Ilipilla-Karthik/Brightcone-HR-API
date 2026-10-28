use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::models::_entities::celebration_wishes::ActiveModel as WishActiveModel;
use crate::models::_entities::connect_posts::{ActiveModel as PostActiveModel};
use crate::models::_entities::employees::{self, Entity as Employees};

#[derive(Debug, Deserialize, Serialize)]
pub struct SendWishPayload {
    pub recipient_name: String,
    pub sender_name: Option<String>,
    pub occasion: Option<String>,
    pub message: Option<String>,
}

#[debug_handler]
pub async fn list_celebrations(State(ctx): State<AppContext>) -> Result<Response> {
    let employees_list = Employees::find()
        .order_by_asc(employees::Column::Id)
        .all(&ctx.db)
        .await
        .unwrap_or_default();

    let mut celebrations = Vec::new();

    if !employees_list.is_empty() {
        for (idx, emp) in employees_list.into_iter().enumerate() {
            let occasion = if idx % 2 == 0 { "Work anniversary" } else { "Birthday" };
            let days_away = (idx as i64 * 3) % 25;
            let milestone = if occasion == "Work anniversary" {
                format!("{}rd work anniversary", (idx % 4) + 1)
            } else {
                format!("Turning {}", 26 + (idx % 8))
            };

            celebrations.push(json!({
                "id": format!("cel_{}", emp.id),
                "empId": emp.employee_code,
                "name": emp.full_name,
                "occasion": occasion,
                "date": format!("2026-10-{}", 20 + (idx % 10)),
                "daysAway": days_away,
                "milestone": milestone,
                "dept": emp.department
            }));
        }
    } else {
        celebrations.push(json!({
            "id": "cel_1",
            "empId": "BC-1012",
            "name": "Karthik Reddy",
            "occasion": "Work anniversary",
            "date": "2026-10-24",
            "daysAway": 0,
            "milestone": "3rd work anniversary",
            "dept": "AI Research"
        }));
        celebrations.push(json!({
            "id": "cel_2",
            "empId": "BC-1004",
            "name": "Ananya Rao",
            "occasion": "Birthday",
            "date": "2026-10-27",
            "daysAway": 3,
            "milestone": "Turning 28",
            "dept": "People Ops"
        }));
        celebrations.push(json!({
            "id": "cel_3",
            "empId": "BC-1038",
            "name": "Rahul Varma",
            "occasion": "Work anniversary",
            "date": "2026-10-30",
            "daysAway": 6,
            "milestone": "2nd work anniversary",
            "dept": "Engineering"
        }));
    }

    format::json(celebrations)
}

#[debug_handler]
pub async fn send_wish(
    State(ctx): State<AppContext>,
    Json(payload): Json<SendWishPayload>,
) -> Result<Response> {
    let now = Utc::now();
    let wish_id = format!("wish_{}", Uuid::new_v4().simple());
    let sender = payload.sender_name.unwrap_or_else(|| "Rahul Varma".to_string());
    let occasion = payload.occasion.unwrap_or_else(|| "Celebration".to_string());
    let msg = payload.message.unwrap_or_else(|| {
        format!("Happy {}! Wishing you tremendous continued success and happiness at Brightcone!", occasion)
    });

    // 1. Insert into PostgreSQL celebration_wishes table
    let active_wish = WishActiveModel {
        wish_id: Set(wish_id.clone()),
        recipient_name: Set(payload.recipient_name.clone()),
        sender_name: Set(sender.clone()),
        occasion: Set(occasion.clone()),
        message: Set(msg.clone()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };
    let _ = active_wish.insert(&ctx.db).await;

    // 2. Also publish celebratory post in connect_posts table
    let feed_post = PostActiveModel {
        post_id: Set(format!("post_{}", Uuid::new_v4().simple())),
        author_id: Set("usr_current".to_string()),
        author_name: Set(sender.clone()),
        author_dept: Set(Some("Engineering".to_string())),
        post_type: Set("Celebration".to_string()),
        to_employee_id: Set(None),
        to_employee_name: Set(Some(payload.recipient_name.clone())),
        body: Set(format!("🎉 Sent a celebratory wish to {} for their {}: \"{}\"", payload.recipient_name, occasion, msg)),
        group_id: Set(None),
        pinned: Set(false),
        likes_count: Set(1),
        comments_count: Set(0),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };
    let _ = feed_post.insert(&ctx.db).await;

    format::json(json!({
        "success": true,
        "wish_id": wish_id,
        "recipient": payload.recipient_name,
        "message": msg
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/connect/celebrations")
        .add("", get(list_celebrations))
        .add("/wish", post(send_wish))
}

pub fn legacy_routes() -> Routes {
    Routes::new()
        .prefix("/connect/celebrations")
        .add("", get(list_celebrations))
        .add("/wish", post(send_wish))
}
