use crate::models::kill_record::KillRecord;
use crate::services::db_helper::get_db_connection_from_pool;
use actix_web::error::ErrorInternalServerError;
use mysql::prelude::*;
use chrono::{DateTime, Utc};

/// 从数据库获取指定游戏的所有击杀记录
pub async fn get_kill_records_from_db(game_id: &str) -> Result<Vec<KillRecord>, actix_web::Error> {
    let mut conn = get_db_connection_from_pool()?;

    let results: Vec<(String, String, Option<String>, String, String, String, Option<String>, Option<String>)> = conn
        .exec(
            r"SELECT id, game_id, killer_id, victim_id, kill_time, cause, weapon, location
              FROM kill_records WHERE game_id = ? ORDER BY kill_time ASC",
            (game_id,),
        )
        .map_err(|e| {
            tracing::error!("Failed to query kill records from database: {}", e);
            ErrorInternalServerError("Database query error")
        })?;

    let mut kill_records = Vec::new();
    for (id, game_id, killer_id, victim_id, kill_time_str, cause, weapon, location) in results {
        // 解析时间字符串
        let kill_time = DateTime::parse_from_rfc3339(&kill_time_str)
            .map_err(|e| {
                tracing::error!("Failed to parse kill time: {}", e);
                ErrorInternalServerError("Data parsing error")
            })?
            .with_timezone(&Utc);
        
        let kill_record = KillRecord::new(
            id,
            game_id,
            killer_id,
            victim_id,
            kill_time,
            cause,
            weapon,
            location,
        );
        kill_records.push(kill_record);
    }

    Ok(kill_records)
}