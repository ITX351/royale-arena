use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

    /// 验证地点字段的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("地点ID不能为空".to_string());
        }

        if self.name.is_empty() {
            return Err("地点名称不能为空".to_string());
        }

        if self.name.len() > 100 {
            return Err("地点名称不能超过100个字符".to_string());
        }

        if self.description.len() > 500 {
            return Err("地点描述不能超过500个字符".to_string());
        }

        // 验证玩家ID列表中的每个ID
        for player_id in &self.players {
            if player_id.is_empty() {
                return Err("玩家ID不能为空".to_string());
            }

            if player_id.len() > 50 {
                return Err("玩家ID不能超过50个字符".to_string());
            }
        }

        // 检查玩家ID是否重复
        let mut player_set = HashSet::new();
        for player_id in &self.players {
            if !player_set.insert(player_id) {
                return Err(format!("玩家ID重复: {}", player_id));
            }
        }

        Ok(())
    }

    /// 添加玩家到地点
    pub fn add_player(&mut self, player_id: String) -> Result<(), String> {
        if player_id.is_empty() {
            return Err("玩家ID不能为空".to_string());
        }

        if player_id.len() > 50 {
            return Err("玩家ID不能超过50个字符".to_string());
        }

        if !self.players.contains(&player_id) {
            self.players.push(player_id);
        }

        Ok(())
    }

    /// 从地点移除玩家
    pub fn remove_player(&mut self, player_id: &str) -> bool {
        if let Some(index) = self.players.iter().position(|x| x == player_id) {
            self.players.remove(index);
            true
        } else {
            false
        }
    }

    /// 获取地点的玩家数量
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// 检查地点是否包含特定玩家
    pub fn has_player(&self, player_id: &str) -> bool {
        self.players.contains(&player_id.to_string())
    }

    /// 设置地点为安全区域
    pub fn set_safe(&mut self, is_safe: bool) {
        self.is_safe = is_safe;
    }

    /// 设置地点为活跃状态（仍在游戏中）
    pub fn set_active(&mut self, is_active: bool) {
        self.is_active = is_active;
    }

    /// 检查地点是否可以被搜索
    pub fn can_be_searched(&self) -> bool {
        self.is_active
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaceStatus {
    pub place: Place,
    pub player_count: u32, // 当前在该地点的玩家数量
    pub is_searched: bool, // 本轮是否已被搜索
}

impl PlaceStatus {
    pub fn new(place: Place) -> Self {
        Self {
            player_count: place.players.len() as u32,
            is_searched: false,
            place,
        }
    }

    /// 标记地点为已搜索
    pub fn mark_as_searched(&mut self) {
        self.is_searched = true;
    }

    /// 重置搜索状态
    pub fn reset_search_status(&mut self) {
        self.is_searched = false;
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

    #[test]
    fn test_place_validation() {
        // 测试有效的地点
        let place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );
        assert!(place.validate().is_ok());

        // 测试空ID
        let mut place = Place::new(
            "".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );
        assert!(place.validate().is_err());

        // 测试空名称
        let place = Place::new(
            "place1".to_string(),
            "".to_string(),
            "这是一个测试地点".to_string(),
        );
        assert!(place.validate().is_err());

        // 测试过长的名称
        let long_name = "A".repeat(101);
        let place = Place::new(
            "place1".to_string(),
            long_name,
            "这是一个测试地点".to_string(),
        );
        assert!(place.validate().is_err());

        // 测试过长的描述
        let long_description = "A".repeat(501);
        let place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            long_description,
        );
        assert!(place.validate().is_err());

        // 测试空玩家ID
        let mut place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );
        place.players.push("".to_string());
        assert!(place.validate().is_err());

        // 测试过长的玩家ID
        let mut place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );
        let long_player_id = "A".repeat(51);
        place.players.push(long_player_id);
        assert!(place.validate().is_err());

        // 测试重复的玩家ID
        let mut place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );
        place.players.push("player1".to_string());
        place.players.push("player1".to_string());
        assert!(place.validate().is_err());
    }

    #[test]
    fn test_player_management() {
        let mut place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );

        // 测试添加玩家
        assert!(place.add_player("player1".to_string()).is_ok());
        assert_eq!(place.players.len(), 1);
        assert_eq!(place.players[0], "player1");

        // 测试添加重复玩家（应该不会重复添加）
        assert!(place.add_player("player1".to_string()).is_ok());
        assert_eq!(place.players.len(), 1);

        // 测试添加更多玩家
        assert!(place.add_player("player2".to_string()).is_ok());
        assert_eq!(place.players.len(), 2);
        assert_eq!(place.players[1], "player2");

        // 测试添加空玩家ID
        assert!(place.add_player("".to_string()).is_err());
        assert_eq!(place.players.len(), 2);

        // 测试添加过长的玩家ID
        let long_player_id = "A".repeat(51);
        assert!(place.add_player(long_player_id).is_err());
        assert_eq!(place.players.len(), 2);

        // 测试移除存在的玩家
        assert!(place.remove_player("player1"));
        assert_eq!(place.players.len(), 1);
        assert_eq!(place.players[0], "player2");

        // 测试移除不存在的玩家
        assert!(!place.remove_player("player3"));
        assert_eq!(place.players.len(), 1);

        // 测试获取玩家数量
        assert_eq!(place.player_count(), 1);

        // 测试检查是否包含特定玩家
        assert!(place.has_player("player2"));
        assert!(!place.has_player("player1"));
    }

    #[test]
    fn test_place_status_management() {
        let place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );
        let mut place_status = PlaceStatus::new(place);

        // 测试初始状态
        assert!(!place_status.is_searched);

        // 测试标记为已搜索
        place_status.mark_as_searched();
        assert!(place_status.is_searched);

        // 测试重置搜索状态
        place_status.reset_search_status();
        assert!(!place_status.is_searched);
    }

    #[test]
    fn test_place_state_management() {
        let mut place = Place::new(
            "place1".to_string(),
            "测试地点".to_string(),
            "这是一个测试地点".to_string(),
        );

        // 测试设置安全状态
        assert!(place.is_safe);
        place.set_safe(false);
        assert!(!place.is_safe);
        place.set_safe(true);
        assert!(place.is_safe);

        // 测试设置活跃状态
        assert!(place.is_active);
        place.set_active(false);
        assert!(!place.is_active);
        place.set_active(true);
        assert!(place.is_active);

        // 测试可搜索性
        place.set_active(true);
        assert!(place.can_be_searched());
        place.set_active(false);
        assert!(!place.can_be_searched());
    }
}
