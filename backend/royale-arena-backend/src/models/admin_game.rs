use crate::models::rules::GameRules;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateGameRequest {
    pub name: String,
    pub description: String,
    pub director_password: String,
    pub max_players: u32,
    pub rules_template_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateGameRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub director_password: Option<String>,
    pub max_players: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateGameResponse {
    pub success: bool,
    pub game_id: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateGameResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeleteGameResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateRuleTemplateRequest {
    pub name: String,
    pub description: String,
    pub rules: GameRules,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateRuleTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rules: Option<GameRules>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateRuleTemplateResponse {
    pub success: bool,
    pub template_id: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateRuleTemplateResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeleteRuleTemplateResponse {
    pub success: bool,
    pub message: Option<String>,
}
