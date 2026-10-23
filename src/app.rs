use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    bgworker::Queue,
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
    Result,
};
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }
}

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(crate::controllers::health::routes())
            .add_route(crate::controllers::shifts::routes())
            .add_route(crate::controllers::holidays::routes())
            .add_route(crate::controllers::leave_types::routes())
            .add_route(crate::controllers::leave_quotas::routes())
            .add_route(crate::controllers::roster::routes())
            .add_route(crate::controllers::attendance::routes())
            .add_route(crate::controllers::leave_calc::routes())
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn before_run(ctx: &AppContext) -> Result<()> {
        let pool = ctx.db.get_postgres_connection_pool();
        crate::db::sync_schema(pool).await.map_err(|e| {
            loco_rs::Error::Message(format!("Database schema synchronization failed: {e}"))
        })?;
        Ok(())
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn truncate(_ctx: &AppContext) -> Result<()> {
        Ok(())
    }

    async fn seed(_ctx: &AppContext, _base: &std::path::Path) -> Result<()> {
        Ok(())
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![])
    }
}
