use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/games", web::get().to(crate::handlers::game::get_games))
            .route("/game/{game_id}", web::get().to(crate::handlers::game::get_game_info))
            .route("/game/{game_id}/rules", web::get().to(crate::handlers::rules::get_game_rules))
            .route("/game/{game_id}/rules", web::put().to(crate::handlers::director::update_game_rules))
            .route("/game/{game_id}/player/{player_id}", web::get().to(crate::handlers::player::get_player_info))
            .route("/game/{game_id}/places", web::get().to(crate::handlers::places::get_places_status))
            .route("/game/{game_id}/players", web::post().to(crate::handlers::director::add_players))
            .route("/game/{game_id}/players", web::get().to(crate::handlers::director::get_players))
            .route("/game/{game_id}/players", web::delete().to(crate::handlers::director::delete_players))
            .route("/game/{game_id}/logs", web::get().to(crate::handlers::logs::get_game_logs))
            .route("/game/{game_id}/stats", web::get().to(crate::handlers::stats::get_game_stats))
            .route("/game/{game_id}/votes", web::get().to(crate::handlers::votes::get_game_votes))
            .route("/game/{game_id}/snapshot", web::get().to(crate::handlers::snapshot::get_game_snapshot))
            .route("/game/{game_id}/reset", web::post().to(crate::handlers::reset::reset_game))
            .route("/game/{game_id}/export", web::get().to(crate::handlers::export::export_game_data))
            .route("/game/{game_id}/ws-auth", web::post().to(crate::handlers::ws_auth::ws_auth))
            .route("/game/{game_id}/ws/{user_type}", web::get().to(crate::handlers::ws_connect::ws_connect))
            .route("/rule-templates", web::get().to(crate::handlers::rule_templates::get_rule_templates))
            .route("/rule-templates/{template_id}", web::get().to(crate::handlers::rule_templates::get_rule_template))
            .route("/admin/login", web::post().to(crate::handlers::admin::admin_login))
            // Admin game management routes
            .route("/admin/games", web::post().to(crate::handlers::admin_game::create_game))
            .route("/admin/game/{game_id}", web::put().to(crate::handlers::admin_game::update_game))
            .route("/admin/game/{game_id}", web::delete().to(crate::handlers::admin_game::delete_game))
            // Admin rule template management routes
            .route("/admin/rule-templates", web::post().to(crate::handlers::admin_game::create_rule_template))
            .route("/admin/rule-templates/{template_id}", web::put().to(crate::handlers::admin_game::update_rule_template))
            .route("/admin/rule-templates/{template_id}", web::delete().to(crate::handlers::admin_game::delete_rule_template))
    );
}