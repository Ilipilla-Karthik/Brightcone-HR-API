use crate::models::_entities::users::{self, ActiveModel as UserActiveModel};
use loco_rs::hash;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

/// Seed only the two specification accounts:
/// 1. Ananya Rao - HR Admin, password: Admin@123
/// 2. Rahul Varma - Employee, password: Welcome@123
/// Any employee with matching work_email auto-provisions as Employee role.
pub async fn seed_users_and_org(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Seed the two spec accounts
    let spec_accounts = vec![
        (
            "ananya.rao@brightcone.ai",
            "Ananya Rao",
            "HR Admin",
            "Admin@123",
        ),
        (
            "rahul.varma@brightcone.ai",
            "Rahul Varma",
            "Employee",
            "Welcome@123",
        ),
    ];

    for (email, name, role, pwd) in spec_accounts {
        let exists = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await?;

        if exists.is_none() {
            let password_hash = hash::hash_password(pwd)?;
            let pid = Uuid::new_v4();
            let user = UserActiveModel {
                pid: Set(pid),
                email: Set(email.to_string()),
                password: Set(password_hash),
                api_key: Set(format!("lo-{}", Uuid::new_v4())),
                name: Set(name.to_string()),
                role: Set(role.to_string()),
                email_verified_at: Set(None),
                ..Default::default()
            };
            user.insert(db).await?;
            tracing::info!("Seeded spec account: {}", email);
        }
    }

    // 2. Auto-provision Employee account for any employee with matching work_email
    // This is handled by the login handler, not seed

    Ok(())
}