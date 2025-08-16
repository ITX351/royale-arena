use crate::models::actor::Actor;
use crate::services::db_helper::get_db_connection_from_pool;
use actix_web::error::ErrorInternalServerError;
use mysql::prelude::*;

/// 从数据库获取演员信息
pub async fn get_actor_from_db(game_id: &str, actor_id: &str) -> Result<Option<Actor>, actix_web::Error> {
    let mut conn = get_db_connection_from_pool()?;

    let result: Option<(String, String, String, String, u32)> = conn
        .exec_first(
            r"SELECT id, game_id, name, password, team_id
              FROM actors WHERE game_id = ? AND id = ?",
            (game_id, actor_id),
        )
        .map_err(|e| {
            tracing::error!("Failed to query actor from database: {}", e);
            ErrorInternalServerError("Database query error")
        })?;

    match result {
        Some((id, game_id, name, password, team_id)) => {
            let actor = Actor {
                id,
                game_id,
                name,
                password,
                team_id,
            };
            Ok(Some(actor))
        }
        None => Ok(None),
    }
}

/// 验证导演密码
pub async fn verify_director_password(game_id: &str, password: &str) -> Result<bool, actix_web::Error> {
    let mut conn = get_db_connection_from_pool()?;

    let result: Option<(String,)> = conn
        .exec_first(
            r"SELECT director_password FROM games WHERE id = ?",
            (game_id,),
        )
        .map_err(|e| {
            tracing::error!("Failed to query game director password from database: {}", e);
            ErrorInternalServerError("Database query error")
        })?;

    match result {
        Some((director_password,)) => Ok(director_password == password),
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_struct_serialization() {
        let actor = Actor {
            id: "actor1".to_string(),
            game_id: "game1".to_string(),
            name: "Test Actor".to_string(),
            password: "pass123".to_string(),
            team_id: 1,
        };

        let json = serde_json::to_string(&actor).unwrap();
        assert!(json.contains("\"password\":\"pass123\""));

        // 测试反序列化
        let deserialized: Actor = serde_json::from_str(&json).unwrap();
        assert_eq!(actor.id, deserialized.id);
        assert_eq!(actor.game_id, deserialized.game_id);
        assert_eq!(actor.name, deserialized.name);
        assert_eq!(actor.password, deserialized.password);
        assert_eq!(actor.team_id, deserialized.team_id);
    }
}