use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/games", web::get().to(crate::handlers::game::get_games))
            .route("/game/{game_id}", web::get().to(crate::handlers::game::get_game_info))
            .route("/game/{game_id}/rules", web::get().to(crate::handlers::rules::get_game_rules))
            .route("/game/{game_id}/player/{player_id}", web::get().to(crate::handlers::player::get_player_info))
            .route("/game/{game_id}/places", web::get().to(crate::handlers::places::get_places_status))
            .route("/rule-templates", web::get().to(crate::handlers::rule_templates::get_rule_templates))
            .route("/rule-templates/{template_id}", web::get().to(crate::handlers::rule_templates::get_rule_template))
            .route("/admin/login", web::post().to(crate::handlers::admin::admin_login))
    );
}