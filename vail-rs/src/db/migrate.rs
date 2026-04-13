use sqlx::PgPool;

pub async fn ensure_partitions(pool: &PgPool) {
    sqlx::query("SELECT create_login_log_partition()")
        .execute(pool)
        .await
        .ok();

    sqlx::query("SELECT create_operator_log_partition()")
        .execute(pool)
        .await
        .ok();
}
