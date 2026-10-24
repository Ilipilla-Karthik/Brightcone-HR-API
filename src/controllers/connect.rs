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

pub async fn list_celebrations() -> Result<Response> {
    format::json(json!([
        {
            "id": "cel_1",
            "empId": "BC-1012",
            "name": "Karthik Reddy",
            "occasion": "Work anniversary",
            "date": "2026-10-24",
            "daysAway": 0,
            "milestone": "3rd work anniversary",
            "dept": "AI Research"
        },
        {
            "id": "cel_2",
            "empId": "BC-1004",
            "name": "Ananya Rao",
            "occasion": "Birthday",
            "date": "2026-10-27",
            "daysAway": 3,
            "milestone": "Turning 28",
            "dept": "People Ops"
        },
        {
            "id": "cel_3",
            "empId": "BC-1038",
            "name": "Rahul Varma",
            "occasion": "Work anniversary",
            "date": "2026-10-30",
            "daysAway": 6,
            "milestone": "2nd work anniversary",
            "dept": "Engineering"
        },
        {
            "id": "cel_4",
            "empId": "BC-1019",
            "name": "Priya Menon",
            "occasion": "Birthday",
            "date": "2026-11-04",
            "daysAway": 11,
            "milestone": "Turning 32",
            "dept": "Engineering"
        },
        {
            "id": "cel_5",
            "empId": "BC-1031",
            "name": "Sneha Iyer",
            "occasion": "Work anniversary",
            "date": "2026-11-12",
            "daysAway": 19,
            "milestone": "1st work anniversary",
            "dept": "Design"
        }
    ]))
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

pub async fn list_events() -> Result<Response> {
    format::json(json!([
        {
            "id": "EV-101",
            "name": "Brightcone Diwali Celebration",
            "etype": "Festival",
            "date": "2026-10-31",
            "time": "16:00",
            "venue": "Hyderabad HQ, 4th Floor Cafeteria",
            "organiser": "Ananya Rao",
            "budget": 75000,
            "att": 140,
            "status": "Confirmed",
            "notes": "Traditional attire, rangoli competition, sweets and high tea."
        },
        {
            "id": "EV-102",
            "name": "Agentic AI Hackathon 2026",
            "etype": "Celebration",
            "date": "2026-11-14",
            "time": "09:00",
            "venue": "Bengaluru Innovation Lab and Zoom",
            "organiser": "Karthik Reddy",
            "budget": 120000,
            "att": 85,
            "status": "Planned",
            "notes": "48-hour build sprint on autonomous agents and workflows."
        },
        {
            "id": "EV-103",
            "name": "Annual Leadership Offsite",
            "etype": "Offsite",
            "date": "2026-11-28",
            "time": "08:30",
            "venue": "Goa Marriott Resort",
            "organiser": "Naresh Vemparala",
            "budget": 350000,
            "att": 24,
            "status": "Planned",
            "notes": "FY27 product roadmap planning and executive team bonding."
        }
    ]))
}

pub async fn list_townhalls() -> Result<Response> {
    format::json(json!([
        {
            "id": "TH-26-01",
            "title": "Q3 All-Hands and AI Product Roadmap",
            "date": "2026-10-30",
            "time": "15:00",
            "mode": "Hybrid",
            "host": "Naresh Vemparala",
            "agenda": "Q3 performance review, customer ARR milestones, and preview of Sign & Connect apps.",
            "att": 142,
            "status": "Scheduled",
            "minutes": "Meeting deck will be uploaded to Learn portal within 24 hours.",
            "actions": "Product demo recordings to be distributed across engineering teams."
        },
        {
            "id": "TH-26-02",
            "title": "People and Culture Open Forum",
            "date": "2026-10-15",
            "time": "16:30",
            "mode": "Online",
            "host": "Ananya Rao",
            "agenda": "Annual benefits renewal, wellness stipend review, and Q&A session.",
            "att": 118,
            "status": "Completed",
            "minutes": "HR confirmed increase of wellness stipend by 15% starting Nov 1.",
            "actions": "Benefits enrollment guide circulated to all employees."
        }
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
        .add("/celebrations", get(list_celebrations))
        .add("/directory", get(list_directory))
        .add("/groups", get(list_groups))
        .add("/events", get(list_events))
        .add("/townhalls", get(list_townhalls))
        .add("/feedback", post(create_feedback))
        .add("/feedback", get(list_feedback))
}
