use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_safe: bool,        // 是否安全区域
    pub is_active: bool,      // 是否仍在游戏中（未被缩圈淘汰）
    pub players: Vec<String>, // 在此地点的玩家ID列表
}

impl Place {
    pub fn new(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            is_safe: true,
            is_active: true,
            players: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaceStatus {
    pub place: Place,
    pub player_count: u32,    // 当前在该地点的玩家数量
    pub is_searched: bool,    // 本轮是否已被搜索
}

impl PlaceStatus {
    pub fn new(place: Place) -> Self {
        Self {
            player_count: place.players.len() as u32,
            is_searched: false,
            place,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place_creation() {
        let place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );
        
        assert_eq!(place.id, "place1");
        assert_eq!(place.name, "测试地点");
        assert_eq!(place.description, "这是一个测试地点");
        assert!(place.is_safe);
        assert!(place.is_active);
        assert!(place.players.is_empty());
    }

    #[test]
    fn test_place_status_creation() {
        let place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );
        let place_status = PlaceStatus::new(place.clone());
        
        assert_eq!(place_status.place.id, place.id);
        assert_eq!(place_status.player_count, 0);
        assert!(!place_status.is_searched);
    }

    #[test]
    fn test_place_with_players() {
        let mut place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );
        place.players.push("player1".to_string());
        place.players.push("player2".to_string());
        
        let place_status = PlaceStatus::new(place.clone());
        
        assert_eq!(place_status.player_count, 2);
        assert_eq!(place.players.len(), 2);
    }

    #[test]
    fn test_place_serialization() {
        let place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );
        let place_status = PlaceStatus::new(place.clone());
        
        let serialized = serde_json::to_string(&place_status).unwrap();
        let deserialized: PlaceStatus = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(place_status.place.id, deserialized.place.id);
        assert_eq!(place_status.player_count, deserialized.player_count);
        assert_eq!(place_status.is_searched, deserialized.is_searched);
    }
}