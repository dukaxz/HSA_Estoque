use std::sync::Mutex;
use rusqlite::Connection;

// Compartilha a conexão com o banco entre todas as rotas usando Mutex para acesso seguro entre threads
pub struct AppState {
    pub conn: Mutex<Connection>,
}

// Cria a tabela "pecas" no banco caso ainda não exista — executado uma vez ao iniciar o servidor
pub fn iniciar_banco(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS pecas (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            codigo_pi        TEXT NOT NULL,
            nome             TEXT NOT NULL,
            proxima_operacao TEXT NOT NULL,
            data_entrada     TEXT NOT NULL,
            lote             TEXT NOT NULL,
            data_saida       TEXT
        );"
    ).expect("Erro ao criar tabela");
}