//! Generic, registry-driven records API — the live `R` engine of the HTML spec.
//!
//! One controller serves every registry kind (`services::rec_defs`):
//! - `GET    /api/v1/rec/defs`                 kind summaries
//! - `GET    /api/v1/rec/:kind`                list records (HTML field keys)
//! - `POST   /api/v1/rec/:kind`                create
//! - `GET    /api/v1/rec/:kind/export`         CSV export
//! - `GET    /api/v1/rec/:kind/:code`          one record
//! - `PUT|PATCH /api/v1/rec/:kind/:code`       patch update (+ history entry)
//! - `POST   /api/v1/rec/:kind/:code/archive`  archive / restore
//! The same routes are mounted a second time at `/api/rec/*` (PLAN.md contract).
//!
//! Security: every handler needs a signed-in [`AuthUser`]. Reads are owner
//! scoped unless the role holds `records:read` (HR) or the domain permission
//! for the kind; writes need `records:manage` (HR), `payroll:manage` or
//! `recruitment:manage`, except a signed-in employee may add/update rows that
//! are about themselves — and then only with the registry's own status default
//! (a self-service row can never approve itself).
use crate::authz::{forbidden, AuthUser};
use crate::services::rec_defs;
use crate::services::rec_store;
use loco_rs::prelude::*;
use serde_json::Value;

use super::records_helpers::*;

/// `GET /rec/defs` — every registry kind as a compact summary.
#[debug_handler]
pub async fn list_kinds(
    auth: AuthUser,
    State(_ctx): State<AppContext>,
) -> Result<Json<Vec<KindSummary>>> {
    let _ = auth; // signed-in only: metadata carries no personal data
    let kinds = rec_defs::all_kinds()
        .map(|k| KindSummary {
            key: k.kind().to_string(),
            title: k.title().to_string(),
            one: k.one().to_string(),
            icon: k.icon().to_string(),
            app: k.app().to_string(),
            fields: k.fields().len(),
        })
        .collect();
    Ok(Json(kinds))
}

/// `GET /rec/:kind` — list a kind's records.
#[debug_handler]
pub async fn list_records(
    auth: AuthUser,
    State(ctx): State<AppContext>,
    Path(kind): Path<String>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Value>>> {
    let def = def_for(&kind)?;
    let t = table_for(&kind)?;
    let access = access_of(&ctx, &auth, &kind).await?;
    let mut rows = scope(&access, load_rows(&ctx, def, t, q.archived.unwrap_or(false)).await?);
    if let (Some(page), Some(size)) = (q.page, q.page_size) {
        if page > 0 && size > 0 {
            let skip = usize::try_from((page - 1) * size).unwrap_or(0);
            rows = rows.into_iter().skip(skip).take(size as usize).collect();
        }
    }
    Ok(Json(rows))
}

/// `GET /rec/:kind/:code` — one record, or 404 when it is out of scope.
#[debug_handler]
pub async fn get_record(
    auth: AuthUser,
    State(ctx): State<AppContext>,
    Path((kind, code)): Path<(String, String)>,
) -> Result<Json<Value>> {
    def_for(&kind)?;
    table_for(&kind)?;
    let access = access_of(&ctx, &auth, &kind).await?;
    let record = rec_store::get_by_code(&ctx.db, &kind, &code).await?;
    match record {
        Some(r) if matches!(&access, Access::All) || is_own_ids(&access, &r, READ_KEYS) => {
            Ok(Json(r))
        }
        _ => not_found(),
    }
}

/// `POST /rec/:kind` — create a record.
#[debug_handler]
pub async fn create_record(
    auth: AuthUser,
    State(ctx): State<AppContext>,
    Path(kind): Path<String>,
    Json(rec): Json<Value>,
) -> Result<Json<Value>> {
    let def = def_for(&kind)?;
    let access = access_of(&ctx, &auth, &kind).await?;
    let mut obj = as_object(rec)?;

    // Registry defaults (`status: Pending`, `half: No`, ...) apply first.
    for (k, v) in def.defaults() {
        obj.entry(k.clone())
            .or_insert_with(|| Value::String(v.clone()));
    }

    match &access {
        Access::All => {
            if !manages(&auth, &kind) {
                return Err(forbidden(
                    "records:manage",
                    "You do not have access to this.",
                ));
            }
        }
        Access::Own(ids) => {
            if !is_own(ids, &Value::Object(obj.clone()), SUBJECT_KEYS) {
                return Err(forbidden(
                    "records:manage",
                    "You can only add records about yourself.",
                ));
            }
            // Self-service rows never decide their own outcome.
            if let Some(status) = def.defaults().get("status") {
                obj.insert("status".into(), Value::String(status.clone()));
            }
            obj.remove("approver");
        }
    }

    let created = rec_store::create_rec(&ctx.db, &kind, &obj)
        .await
        .map_err(Error::BadRequest)?;
    Ok(Json(created))
}

/// `PUT|PATCH /rec/:kind/:code` — patch a record and journal the change.
#[debug_handler]
pub async fn update_record(
    auth: AuthUser,
    State(ctx): State<AppContext>,
    Path((kind, code)): Path<(String, String)>,
    Json(rec): Json<Value>,
) -> Result<Json<Value>> {
    let def = def_for(&kind)?;
    let t = table_for(&kind)?;
    let access = access_of(&ctx, &auth, &kind).await?;
    let existing = rec_store::get_by_code(&ctx.db, &kind, &code)
        .await?
        .ok_or(Error::NotFound)?;
    let _ = (def, t);

    match &access {
        Access::All => {
            if !manages(&auth, &kind) {
                return Err(forbidden(
                    "records:manage",
                    "You do not have access to this.",
                ));
            }
        }
        Access::Own(ids) => {
            if !is_own(ids, &existing, SUBJECT_KEYS) {
                return not_found(); // 404, not 403, for other people's rows
            }
        }
    }

    let mut patch = as_object(rec)?;
    if matches!(access, Access::Own(_)) {
        patch.remove("status");
        patch.remove("approver");
    }
    let changed: Vec<String> = patch.keys().cloned().collect();
    let mut history: Vec<Value> = existing
        .get("history")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    history.push(serde_json::json!({
        "at": chrono::Utc::now().to_rfc3339(),
        "by": auth.user.email,
        "fields": changed,
    }));
    let history_json = Value::Array(history).to_string();
    let updated =
        rec_store::update_rec(&ctx.db, &kind, &code, &patch, &history_json)
            .await
            .map_err(Error::BadRequest)?;
    Ok(Json(updated))
}

/// `POST /rec/:kind/:code/archive` — archive (`true`) or restore (`false`).
#[debug_handler]
pub async fn set_archived(
    auth: AuthUser,
    State(ctx): State<AppContext>,
    Path((kind, code)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> Result<Json<Value>> {
    table_for(&kind)?;
    let access = access_of(&ctx, &auth, &kind).await?;
    let existing = rec_store::get_by_code(&ctx.db, &kind, &code)
        .await?
        .ok_or(Error::NotFound)?;
    match &access {
        Access::All => {
            if !manages(&auth, &kind) {
                return Err(forbidden(
                    "records:manage",
                    "You do not have access to this.",
                ));
            }
        }
        Access::Own(ids) => {
            if !is_own(ids, &existing, SUBJECT_KEYS) {
                return not_found();
            }
        }
    }
    let archived = match body {
        Value::Bool(b) => b,
        Value::Object(o) => o
            .get("archived")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        _ => true,
    };
    rec_store::set_archived(&ctx.db, &kind, &code, archived)
        .await
        .map_err(Error::BadRequest)?;
    match rec_store::get_by_code(&ctx.db, &kind, &code).await? {
        Some(r) => Ok(Json(r)),
        None => not_found(),
    }
}

/// `GET /rec/:kind/export` — CSV of every visible row.
#[debug_handler]
pub async fn export_records(
    auth: AuthUser,
    State(ctx): State<AppContext>,
    Path(kind): Path<String>,
    Query(q): Query<ListQuery>,
) -> Result<axum::response::Response> {
    let def = def_for(&kind)?;
    let t = table_for(&kind)?;
    let access = access_of(&ctx, &auth, &kind).await?;
    let rows = scope(
        &access,
        load_rows(&ctx, def, t, q.archived.unwrap_or(false)).await?,
    );

    let mut keys = vec!["code".to_string(), "archived".to_string()];
    keys.extend(rec_store::field_keys(def, t));
    let mut csv = keys.join(",") + "\n";
    for row in &rows {
        let line: Vec<String> = keys
            .iter()
            .map(|k| csv_cell(row.get(k).unwrap_or(&Value::Null)))
            .collect();
        csv.push_str(&line.join(","));
        csv.push('\n');
    }

    let response = axum::response::Response::builder()
        .header("Content-Type", "text/csv; charset=utf-8")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{kind}_export.csv\""),
        )
        .body(axum::body::Body::from(csv))
        .map_err(|e| Error::string(&e.to_string()))?;
    Ok(response)
}

fn build(prefix: &str) -> Routes {
    Routes::new()
        .prefix(prefix)
        .add("/defs", get(list_kinds))
        .add("/{kind}", get(list_records))
        .add("/{kind}", post(create_record))
        .add("/{kind}/export", get(export_records))
        .add("/{kind}/{code}", get(get_record))
        .add("/{kind}/{code}", put(update_record))
        .add("/{kind}/{code}", patch(update_record))
        .add("/{kind}/{code}/archive", post(set_archived))
}

/// `/api/v1/rec/*` — used by the SPA (axios `baseURL = /api/v1`).
pub fn routes() -> Routes {
    build("/api/v1/rec")
}

/// `/api/rec/*` — the path documented in PLAN.md.
pub fn alt_routes() -> Routes {
    build("/api/rec")
}
