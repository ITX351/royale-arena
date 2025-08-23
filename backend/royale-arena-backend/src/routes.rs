use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Admin routes
    cfg.service(
        web::scope("/api/admin")
            .route("/login", web::post().to(crate::handlers::admin::admin_login))
            .route("/games", web::post().to(crate::handlers::admin_game::create_game))
            .route("/game/{game_id}", web::put().to(crate::handlers::admin_game::update_game))
            .route("/game/{game_id}", web::delete().to(crate::handlers::admin_game::delete_game))
            .route("/rule-templates", web::post().to(crate::handlers::admin_game::create_rule_template))
            .route("/rule-templates/{template_id}", web::put().to(crate::handlers::admin_game::update_rule_template))
            .route("/rule-templates/{template_id}", web::delete().to(crate::handlers::admin_game::delete_rule_template)),
    );

    // Game routes
    cfg.service(
        web::scope("/api/game")
            .route("", web::get().to(crate::handlers::game::get_games))
            .route("/{game_id}", web::get().to(crate::handlers::game::get_game_info))
            .route("/{game_id}/rules", web::get().to(crate::handlers::rules::get_game_rules))
            .route("/{game_id}/player/{player_id}", web::get().to(crate::handlers::player::get_player_info))
            .route("/{game_id}/player/{player_id}/login", web::post().to(crate::handlers::player::player_login))
            .route("/{game_id}/places", web::get().to(crate::handlers::places::get_places_status))
            .route("/{game_id}/places/{place_id}", web::put().to(crate::handlers::places::update_place_status))
            .route("/{game_id}/places/{place_id}/add-player/{player_id}", web::post().to(crate::handlers::places::add_player_to_place))
            .route("/{game_id}/places/{place_id}/remove-player/{player_id}", web::post().to(crate::handlers::places::remove_player_from_place))
            .route("/{game_id}/vote", web::post().to(crate::handlers::votes::get_game_votes))
            .route("/{game_id}/ws-auth", web::post().to(crate::handlers::ws_auth::ws_auth)),
    );

    // Director routes
    cfg.service(
        web::scope("/api/director")
            .route("/{game_id}/players/{password}", web::post().to(crate::handlers::director::add_players))
            .route("/{game_id}/players/{password}", web::get().to(crate::handlers::director::get_players))
            .route("/{game_id}/players/{password}", web::delete().to(crate::handlers::director::delete_players))
            .route("/{game_id}/rules/{password}", web::put().to(crate::handlers::director::update_game_rules))
            .route("/{game_id}/kills/{password}", web::get().to(crate::handlers::kill_records::get_game_kills))
            .route("/{game_id}/logs/{password}", web::get().to(crate::handlers::logs::get_game_logs))
            .route("/{game_id}/snapshot/{password}", web::get().to(crate::handlers::snapshot::get_game_snapshot))
            .route("/{game_id}/reset/{password}", web::post().to(crate::handlers::reset::reset_game))
            .route("/{game_id}/export/{password}", web::get().to(crate::handlers::export::export_game_data)),
    );

    // WebSocket routes
    cfg.service(
        web::scope("/api/ws")
            .route("/{game_id}/{user_type}", web::get().to(crate::handlers::ws_connect::ws_connect)),
    );
    
    // Rule templates routes
    cfg.service(
        web::scope("/api/rule-templates")
            .route("", web::get().to(crate::handlers::rule_templates::get_rule_templates))
            .route("/{template_id}", web::get().to(crate::handlers::rule_templates::get_rule_template)),
    );
}
