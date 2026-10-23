use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    bgworker::{BackgroundWorker, Queue},
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    db::truncate_table,
    environment::Environment,
    task::Tasks,
    Result,
};
use migration::Migrator;
use std::path::Path;

#[allow(unused_imports)]
use crate::{
    controllers, initializers::seed::SeedInitializer, models::_entities::users, tasks,
    workers::downloader::DownloadWorker,
};

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![
            Box::new(crate::initializers::schema_sync::SchemaSyncInitializer),
            Box::new(SeedInitializer),
        ])
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::auth::routes())
            .add_route(controllers::auth::v1_routes())
            .add_route(controllers::dashboard::routes())
            .add_route(controllers::employees::routes())
            .add_route(controllers::departments::routes())
            .add_route(controllers::jobs::routes())
            .add_route(controllers::candidates::routes())
            .add_route(controllers::applications::routes())
            .add_route(controllers::interviews::routes())
            .add_route(controllers::offers::routes())
            .add_route(controllers::referrals::routes())
            .add_route(controllers::performance::routes())
            .add_route(controllers::leave::routes())
            .add_route(controllers::attendance::routes())
            .add_route(controllers::helpdesk::routes())
            .add_route(controllers::resignation::routes())
            .add_route(controllers::assets::routes())
            .add_route(controllers::documents::routes())
            .add_route(controllers::signature_templates::routes())
            .add_route(controllers::signature_templates::v1_routes())
            .add_route(controllers::learn::routes())
            .add_route(controllers::shifts::routes())
            .add_route(controllers::shifts::legacy_routes())
            .add_route(controllers::holidays::routes())
            .add_route(controllers::holidays::legacy_routes())
            .add_route(controllers::roster::routes())
            .add_route(controllers::roster::legacy_routes())
    }

    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        queue.register(DownloadWorker::build(ctx)).await?;
        queue.register(crate::workers::course_reminder::CourseReminderWorker::build(ctx)).await?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::user_create::UserCreate);
        tasks.register(tasks::user_delete::UserDelete);
    }

    async fn truncate(ctx: &AppContext) -> Result<()> {
        truncate_table(&ctx.db, users::Entity).await?;
        Ok(())
    }

    async fn seed(ctx: &AppContext, _base: &Path) -> Result<()> {
        crate::services::seed::run_seed(&ctx.db)
            .await
            .map_err(|e| loco_rs::Error::string(&e.to_string()))?;
        Ok(())
    }
}
