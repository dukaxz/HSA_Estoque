mod models;
mod db;
mod repository;
mod validation;
mod handlers;

use std::sync::Mutex;
use rusqlite::Connection;
use actix_web::{web, App, HttpServer};

use db::AppState;

// ================================================================================================
// MAIN — Inicializa o banco e sobe o servidor com todas as rotas registradas

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Abre (ou cria) o arquivo de banco de dados SQLite na raiz do projeto
    let conn = Connection::open("pecas.db")
        .expect("Erro ao abrir banco");

    // Garante que a tabela existe antes de qualquer requisição chegar
    db::iniciar_banco(&conn);

    // Empacota a conexão no AppState com Mutex para compartilhar entre as rotas com segurança
    let app_state = web::Data::new(AppState {
        conn: Mutex::new(conn),
    });

    println!("Servidor rodando em http://localhost:8080");

    // Registra todas as rotas e sobe o servidor na porta 8080
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health",     web::get().to(handlers::health_check))   // Verifica se o servidor está no ar
            .route("/pecas",      web::get().to(handlers::get_pecas))      // Lista peças filtradas em JSON
            .route("/pecas",      web::post().to(handlers::post_peca))     // Cadastra nova peça
            .route("/pecas/{id}", web::put().to(handlers::put_peca))       // Atualiza peça pelo id
            .route("/pecas/{id}", web::delete().to(handlers::delete_peca)) // Deleta peça pelo id
            // Serve toda a pasta static/ (index.html, style.css, app.js) — precisa ser
            // registrado por último, pois "/" é um prefixo que "engole" o resto se vier antes
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}