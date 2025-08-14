use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/games", web::get().to(crate::handlers::game::get_games))
            .route("/game/{game_id}", web::get().to(crate::handlers::game::get_game_info))
            .route("/game/{game_id}/rules", web::get().to(crate::handlers::rules::get_game_rules))
            .route("/game/{game_id}/player/{player_id}", web::get().to(crate::handlers::player::get_player_info))
            .route("/admin/login", web::post().to(crate::handlers::admin::admin_login))
    );
}