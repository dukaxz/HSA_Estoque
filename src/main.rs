use std::sync::Mutex;

use actix_web::{web, App, HttpServer, HttpResponse, Responder};

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Servidor rodando!")
}

async fn get_pecas(data: web::Data<AppState>) -> impl Responder {
    let conn = data.conn.lock().expect("Erro ao acessar banco");
    let lista = listar_pecas(&conn);
    HttpResponse::Ok().json(lista)
}

use std::str::FromStr;

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

impl FromStr for ProximaOperacao {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FormarPonta" =>        Ok(ProximaOperacao::FormarPonta),
            "Trefilar" =>           Ok(ProximaOperacao::Trefilar),
            "Endireitar" =>         Ok(ProximaOperacao::Endireitar),
            "Cortar" =>             Ok(ProximaOperacao::Cortar),
            "LimparBlank" =>        Ok(ProximaOperacao::LimparBlank),
            "FormarEndForming" =>   Ok(ProximaOperacao::FormarEndForming),
            "Curvar" =>             Ok(ProximaOperacao::Curvar),
            "Recalcar" =>           Ok(ProximaOperacao::Recalcar),
            "Rebarbar" =>           Ok(ProximaOperacao::Rebarbar),
            "Zincar" =>             Ok(ProximaOperacao::Zincar),
            "Calibrar" =>           Ok(ProximaOperacao::Calibrar),
            _ => Err(format!("Operação desconhecida: {}", s)),
        }
    }
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

struct AppState {
    conn: Mutex<Connection>,
}

fn iniciar_banco(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS pecas (
            id  INTEGER PRIMARY KEY AUTOINCREMENT,
            codigo_pi   TEXT NOT NULL,
            nome    TEXT NOT NULL,
            proxima_operacao TEXT NOT NULL,
            data_entrada TEXT NOT NULL,
            lote    TEXT NOT NULL,
            data_saida  TEXT
            
        );"
    ).expect("Erro ao criar tabela");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn = Connection::open("pecas.db")
        .expect("Erro ao abrir banco");

    iniciar_banco(&conn);

    let app_state = web::Data::new(AppState {
        conn: Mutex::new(conn),
    });

    println!("Servidor rodando em http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health",      web::get().to(health_check))
            .route("/pecas",       web::get().to(get_pecas))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
    
fn inserir_peca(conn: &Connection, peca: &Peca) {
    conn.execute(
        "INSERT INTO pecas (codigo_pi, nome, proxima_operacao, lote, data_entrada, data_saida)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    (
        &peca.codigo_pi,
        &peca.nome,
        format!("{:?}", peca.proxima_operacao),
        &peca.lote,
        &peca.data_entrada,
        &peca.data_saida,
        )
    ).expect("Erro ao inserir peça");
}


fn deletar_peca(conn: &Connection, id: i32) {
    conn.execute(
        "DELETE FROM pecas WHERE id = ?1",
        [id],
    ).expect("Erro ao deletar peça");

    println!("Peça #{} removida.", id);
}


fn atualizar_peca(conn: &Connection, peca: &Peca) {
    conn.execute(
        "UPDATE pecas SET
            codigo_pi        = ?1,
            nome             = ?2,
            proxima_operacao = ?3,
            lote             = ?4,
            data_entrada     = ?5,
            data_saida       = ?6
         WHERE id = ?7",
        (
            &peca.codigo_pi,
            &peca.nome,
            format!("{:?}", peca.proxima_operacao),
            &peca.lote,
            &peca.data_entrada,
            &peca.data_saida,
            &peca.id,
        ),
    ).expect("Erro ao atualizar peça");

    println!("Peça #{} atualizada.", peca.id);
}


fn listar_pecas(conn: &Connection) -> Vec<Peca> {
    
    let mut stmt = conn
    .prepare("SELECT id, codigo_pi, nome, proxima_operacao, lote, data_entrada, data_saida FROM pecas").expect("Erro ao preparar consulta");

    let pecas: Vec<Peca> = stmt
        .query_map([], |row| {
            Ok(Peca {
                id:               row.get(0)?,
                codigo_pi:        row.get(1)?,
                nome:             row.get(2)?,
                proxima_operacao: {
                    let s: String = row.get(3)?;
                    s.parse().expect("Operação invalida no banco")
                },
                lote:             row.get(4)?,
                data_entrada:     row.get(5)?,
                data_saida:       row.get(6)?,
        })
    })
    .expect("Erro ao consultar peça").filter_map(|r| r.ok()).collect();
    
    pecas
}   

