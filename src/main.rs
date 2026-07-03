//index html
use actix_files::NamedFile;
use std::path::PathBuf;

//=================================================================================================================

use std::str::FromStr;
use std::sync::Mutex;

use rusqlite::Connection;
use serde::{Serialize, Deserialize};
use actix_web::{web, App, HttpServer, HttpResponse, Responder};

//=================================================================================================================

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

//=================================================================================================================

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

// Pesquisar mais afundo 
struct AppState {
    conn: Mutex<Connection>,
}

//=================================================================================================================

// funções para ações

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
//=================================================================================================================

#[derive(Deserialize)]
struct PecaJson {
    codigo_pi:        String,
    nome:             String,
    proxima_operacao: String,
    lote:             String,
    data_entrada:     String,
    data_saida:       Option<String>,
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Servidor rodando!")
}

async fn get_pecas(data: web::Data<AppState>) -> impl Responder {
    let conn = data.conn.lock().expect("Erro ao acessar banco");
    let lista = listar_pecas(&conn);
    HttpResponse::Ok().json(lista)
}

async fn post_peca(
    data: web::Data<AppState>,
    body: web::Json<PecaJson>,
) -> impl Responder {

    if body.codigo_pi.len() != 8 || !body.codigo_pi.chars().all(|c| c.is_ascii_digit()) {
        return HttpResponse::BadRequest().body("Código PI inválido — deve ter exatamente 8 dígitos numéricos.");
    }

    let lote_valido = body.lote.starts_with('H')
    && body.lote.len() == 11
    && body.lote[1..].chars().all(|c| c.is_ascii_digit());

    if !lote_valido {
        return HttpResponse::BadRequest().body("Lote invalido - fora do padrão.")
    }
    
    let conn = data.conn.lock().expect("Erro ao acessar banco");

    let peca = Peca {
        id:               0,
        codigo_pi:        body.codigo_pi.clone(),
        nome:             body.nome.clone(),
        proxima_operacao: body.proxima_operacao.parse()
                            .expect("Operação inválida"),
        lote:             body.lote.clone(),
        data_entrada:     body.data_entrada.clone(),
        data_saida:       body.data_saida.clone(),
    };

    inserir_peca(&conn, &peca);
    HttpResponse::Ok().body("Peça cadastrada com sucesso!")
}

async fn delete_peca(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();
    let conn = data.conn.lock().expect("Erro ao acessar banco");
    deletar_peca(&conn, id);
    HttpResponse::Ok().body("Peça deletada.")
}

//=================================================================================================================

async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open(PathBuf::from("static/index.html"))?)
}
// Pesquisar afundo async api ("Main")
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
            .route("/health", web::get().to(health_check))
            .route("/pecas", web::get().to(get_pecas))
            .route("/pecas", web::post().to(post_peca))
            .route("/", web::get().to(index))
            .route("/pecas/{id}", web::delete().to(delete_peca))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

//=================================================================================================================

