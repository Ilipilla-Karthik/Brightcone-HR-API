use chrono::{DateTime, Utc};
use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::models::_entities::connect_comments::{self, ActiveModel as CommentActiveModel, Entity as ConnectComments};
use crate::models::_entities::connect_feedback::{self, ActiveModel as FeedbackActiveModel, Entity as ConnectFeedback};
use crate::models::_entities::connect_posts::{self, ActiveModel as PostActiveModel, Entity as ConnectPosts, Model as PostModel};
use crate::models::_entities::employees::{self, Entity as Employees};

#[derive(Debug, Deserialize)]
pub struct CreatePostPayload {
    pub author_name: Option<String>,
    pub author_dept: Option<String>,
    pub post_type: String,
    pub to_employee_name: Option<String>,
    pub body: String,
    pub group_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentPayload {
    pub author_name: Option<String>,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFeedbackPayload {
    pub category: String,
    pub body: String,
}

pub async fn list_posts(State(ctx): State<AppContext>) -> Result<Response> {
    let posts = ConnectPosts::find()
        .order_by_desc(connect_posts::Column::Pinned)
        .order_by_desc(connect_posts::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    format::json(posts)
}

pub async fn create_post(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreatePostPayload>,
) -> Result<Response> {
    let now: DateTime<Utc> = Utc::now();
    let post_id = format!("post_{}", Uuid::new_v4().simple());
    let author_name = payload.author_name.unwrap_or_else(|| "Ananya Rao".to_string());
    let author_dept = payload.author_dept.or_else(|| Some("People Ops".to_string()));

    let new_post = PostActiveModel {
        post_id: Set(post_id),
        author_id: Set("usr_current".to_string()),
        author_name: Set(author_name),
        author_dept: Set(author_dept),
        post_type: Set(payload.post_type),
        to_employee_id: Set(None),
        to_employee_name: Set(payload.to_employee_name),
        body: Set(payload.body),
        group_id: Set(payload.group_id),
        pinned: Set(false),
        likes_count: Set(0),
        comments_count: Set(0),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };

    let inserted = new_post.insert(&ctx.db).await?;
    format::json(inserted)
}

pub async fn toggle_like(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let post = ConnectPosts::find()
        .filter(connect_posts::Column::PostId.eq(&id))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let mut active: PostActiveModel = post.into();
    let current_likes = *active.likes_count.as_ref();
    active.likes_count = Set(current_likes + 1);
    active.updated_at = Set(Utc::now().into());

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

pub async fn list_comments(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let comments = ConnectComments::find()
        .filter(connect_comments::Column::PostId.eq(&id))
        .order_by_asc(connect_comments::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    format::json(comments)
}

pub async fn add_comment(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateCommentPayload>,
) -> Result<Response> {
    let now: DateTime<Utc> = Utc::now();
    let comment_id = format!("cmt_{}", Uuid::new_v4().simple());
    let author_name = payload.author_name.unwrap_or_else(|| "Rahul Varma".to_string());

    let new_cmt = CommentActiveModel {
        comment_id: Set(comment_id),
        post_id: Set(id.clone()),
        author_id: Set("usr_current".to_string()),
        author_name: Set(author_name),
        body: Set(payload.body),
        created_at: Set(now.into()),
        ..Default::default()
    };
    let inserted = new_cmt.insert(&ctx.db).await?;

    // Increment post comments count
    if let Ok(Some(post)) = ConnectPosts::find().filter(connect_posts::Column::PostId.eq(&id)).one(&ctx.db).await {
        let mut active: PostActiveModel = post.into();
        let cur = *active.comments_count.as_ref();
        active.comments_count = Set(cur + 1);
        let _ = active.update(&ctx.db).await;
    }

    format::json(inserted)
}

pub async fn toggle_pin(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let post = ConnectPosts::find()
        .filter(connect_posts::Column::PostId.eq(&id))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let mut active: PostActiveModel = post.into();
    let cur_pin = *active.pinned.as_ref();
    active.pinned = Set(!cur_pin);
    active.updated_at = Set(Utc::now().into());

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

pub async fn list_directory(State(ctx): State<AppContext>) -> Result<Response> {
    let emps = Employees::find().all(&ctx.db).await?;
    format::json(emps)
}

pub async fn list_groups() -> Result<Response> {
    format::json(json!([
        { "id": "gp_eng", "name": "Engineering & AI", "members": 48, "description": "Platform, ML, agentic architectures" },
        { "id": "gp_design", "name": "Design & Craft", "members": 16, "description": "Design systems, UI/UX critiques" },
        { "id": "gp_founders", "name": "Book Club", "members": 24, "description": "Monthly books on craft and leadership" },
        { "id": "gp_fitness", "name": "Runners & Cyclists", "members": 32, "description": "Weekly weekend 5k and gravel rides" }
    ]))
}

pub async fn create_feedback(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateFeedbackPayload>,
) -> Result<Response> {
    let now: DateTime<Utc> = Utc::now();
    let feedback_id = format!("fb_{}", Uuid::new_v4().simple());

    let new_fb = FeedbackActiveModel {
        feedback_id: Set(feedback_id),
        category: Set(payload.category),
        body: Set(payload.body),
        status: Set("New".to_string()),
        created_at: Set(now.into()),
        ..Default::default()
    };

    let inserted = new_fb.insert(&ctx.db).await?;
    format::json(inserted)
}

pub async fn list_feedback(State(ctx): State<AppContext>) -> Result<Response> {
    let fbs = ConnectFeedback::find()
        .order_by_desc(connect_feedback::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    format::json(fbs)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/connect")
        .add("/posts", get(list_posts))
        .add("/posts", post(create_post))
        .add("/posts/{id}/like", post(toggle_like))
        .add("/posts/{id}/comments", get(list_comments))
        .add("/posts/{id}/comments", post(add_comment))
        .add("/posts/{id}/pin", patch(toggle_pin))
        .add("/celebrations", get(crate::controllers::celebrations::list_celebrations))
        .add("/celebrations/wish", post(crate::controllers::celebrations::send_wish))
        .add("/directory", get(list_directory))
        .add("/groups", get(list_groups))
        .add("/events", get(crate::controllers::events::list_events))
        .add("/events", post(crate::controllers::events::create_event))
        .add("/events/{id}", patch(crate::controllers::events::update_event))
        .add("/events/{id}", put(crate::controllers::events::update_event))
        .add("/events/{id}", delete(crate::controllers::events::delete_event))
        .add("/townhalls", get(crate::controllers::townhalls::list_townhalls))
        .add("/townhalls", post(crate::controllers::townhalls::create_townhall))
        .add("/townhalls/{id}", patch(crate::controllers::townhalls::update_townhall))
        .add("/townhalls/{id}", put(crate::controllers::townhalls::update_townhall))
        .add("/townhalls/{id}", delete(crate::controllers::townhalls::delete_townhall))
        .add("/feedback", post(create_feedback))
        .add("/feedback", get(list_feedback))
}
