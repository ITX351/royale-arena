use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String, // waiting|running|paused|ended
    pub phase: String,  // day|night
    pub player_count: u32,
    pub max_players: u32,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub action_start_time: Option<String>,
    pub action_end_time: Option<String>,
    pub safe_zones: Vec<String>,
    pub weather: f64,
    pub announcements: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = Game {
            id: "game1".to_string(),
            name: "Test Game".to_string(),
            description: "A test game".to_string(),
            status: "waiting".to_string(),
            phase: "day".to_string(),
            player_count: 0,
            max_players: 100,
            start_time: None,
            end_time: None,
            action_start_time: None,
            action_end_time: None,
            safe_zones: vec![],
            weather: 0.0,
            announcements: vec![],
        };

        assert_eq!(game.id, "game1");
        assert_eq!(game.name, "Test Game");
        assert_eq!(game.status, "waiting");
        assert_eq!(game.phase, "day");
        assert_eq!(game.player_count, 0);
        assert_eq!(game.max_players, 100);
    }

    #[test]
    fn test_game_serialization() {
        let game = Game {
            id: "game1".to_string(),
            name: "Test Game".to_string(),
            description: "A test game".to_string(),
            status: "running".to_string(),
            phase: "night".to_string(),
            player_count: 50,
            max_players: 100,
            start_time: Some("2023-01-01T00:00:00Z".to_string()),
            end_time: None,
            action_start_time: Some("2023-01-01T01:00:00Z".to_string()),
            action_end_time: Some("2023-01-01T02:00:00Z".to_string()),
            safe_zones: vec!["zone1".to_string(), "zone2".to_string()],
            weather: 0.5,
            announcements: vec!["Welcome!".to_string()],
        };

        let serialized = serde_json::to_string(&game).unwrap();
        let deserialized: Game = serde_json::from_str(&serialized).unwrap();

        assert_eq!(game.id, deserialized.id);
        assert_eq!(game.name, deserialized.name);
        assert_eq!(game.status, deserialized.status);
        assert_eq!(game.phase, deserialized.phase);
        assert_eq!(game.player_count, deserialized.player_count);
        assert_eq!(game.max_players, deserialized.max_players);
        assert_eq!(game.start_time, deserialized.start_time);
        assert_eq!(game.action_start_time, deserialized.action_start_time);
        assert_eq!(game.action_end_time, deserialized.action_end_time);
        assert_eq!(game.safe_zones, deserialized.safe_zones);
        assert_eq!(game.weather, deserialized.weather);
        assert_eq!(game.announcements, deserialized.announcements);
    }
}