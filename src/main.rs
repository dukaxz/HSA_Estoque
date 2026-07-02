

use rusqlite::Connection;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
enum ProximaOperacao {
    FormarPonta,
    Trefilar,
    Endireitar,
    Cortar,
    LimparBlank,
    FormarEndForming,
    Curvar,
    Recalcar,
    Rebarbar,
    Zincar,
    Calibrar,
}

#[derive(Debug, Serialize, Deserialize)]

struct Peca {
    id: i32,
    codigo_pi: String,
    nome: String,
    proxima_operacao: ProximaOperacao,
    data_entrada: String,
    lote: String,
    data_saida: Option<String>,
}

fn iniciar_banco(conn: &Connection) {
    conn.execute_batch(
        "CREATE A TABLE IF NOT EXISTIS pecas (
            id  INTEGER PRIMARY KEY AUTOINCREMENT,
            codigo_pi   TEXT NOT NULL,
            nome    TEXT NOT NULL,
            proxima_operacao TEXT NOT NULL,
            data_entrada TEXT NOT NULL,
            lote    TEXT NOT NULL,
            data_saida  TEXT,
            
        );"
    ).expect("Erro ao criar tabela");
}

fn main() {
    let conn = Connection::open("pecas.db").expect("Erro ao abrir banco");

    iniciar_banco(&conn);

    println!("Banco iniciado com sucesso!");
}

fn inserir_peca(conn: &Connection, peca: &Peca) {
    conn.execute(
        "INSERT INTO pecas (codigo_pi, nome, proxima_operacao, lote, data_entrada, data_saida)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    (
        &peca.codigo_pi,
        &peca.nome,
        format!("{:?}", peca.proxima_operacao), // converte enum para texto
        &peca.lote,
        &peca.data_entrada,
        &peca.data_saida,
        )
    ).expect("Erro ao inserir peça");
}