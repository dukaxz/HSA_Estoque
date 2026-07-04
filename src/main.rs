// Importações para servir o arquivo HTML estático
use actix_files::NamedFile;
use std::path::PathBuf;

// Importações de concorrência, banco de dados, serialização e servidor web
use std::str::FromStr;
use std::sync::Mutex;
use rusqlite::Connection;
use serde::{Serialize, Deserialize};
use actix_web::{web, App, HttpServer, HttpResponse, Responder};

// ================================================================================================
// ENUM — Define as operações fixas do processo produtivo (espelho do processo real da HSA)


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

// Converte String vinda do banco ("Recalcar") de volta para a variante do enum (ProximaOperacao::Recalcar)
impl FromStr for ProximaOperacao {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FormarPonta"      => Ok(ProximaOperacao::FormarPonta),
            "Trefilar"         => Ok(ProximaOperacao::Trefilar),
            "Endireitar"       => Ok(ProximaOperacao::Endireitar),
            "Cortar"           => Ok(ProximaOperacao::Cortar),
            "LimparBlank"      => Ok(ProximaOperacao::LimparBlank),
            "FormarEndForming" => Ok(ProximaOperacao::FormarEndForming),
            "Curvar"           => Ok(ProximaOperacao::Curvar),
            "Recalcar"         => Ok(ProximaOperacao::Recalcar),
            "Rebarbar"         => Ok(ProximaOperacao::Rebarbar),
            "Zincar"           => Ok(ProximaOperacao::Zincar),
            "Calibrar"         => Ok(ProximaOperacao::Calibrar),
            // Qualquer valor fora da lista causa erro imediato — dado inválido nunca entra no banco
            _  => Err(format!("Operação desconhecida: {}", s)),
        }
    }
}

// ================================================================================================
// STRUCTS — Moldes de dados usados no sistema


// Representa uma peça completa conforme salva no banco — usada em SELECT e UPDATE
#[derive(Debug, Serialize, Deserialize)]
struct Peca {
    id:               i32,            // Gerado automaticamente pelo SQLite (AUTOINCREMENT)
    codigo_pi:        String,         // 8 dígitos numéricos — identificador interno da peça
    nome:             String,         // Nome descritivo da peça (ex: 31-2298 - Haste Curvada)
    proxima_operacao: ProximaOperacao,// Operação seguinte no processo produtivo
    data_entrada:     String,         // Data em que a peça entrou no estoque
    lote:             String,         // Lote de origem — padrão H + 10 dígitos (ex: H0501001001)
    data_saida:       Option<String>, // None = peça ainda em estoque, Some = data em que saiu
}

// Representa os dados recebidos via JSON do formulário HTML — sem id (gerado pelo banco)
#[derive(Deserialize)]
struct PecaJson {
    codigo_pi:        String,
    nome:             String,
    proxima_operacao: String,         // Recebido como String e convertido para enum antes de salvar
    lote:             String,
    data_entrada:     String,
    data_saida:       Option<String>,
}

// Compartilha a conexão com o banco entre todas as rotas usando Mutex para acesso seguro entre threads
struct AppState {
    conn: Mutex<Connection>,
}

// ================================================================================================
// FUNÇÕES DE BANCO — Executam as operações SQL no SQLite


// Cria a tabela "pecas" no banco caso ainda não exista — executado uma vez ao iniciar o servidor
fn iniciar_banco(conn: &Connection) {
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

// Insere uma nova peça no banco — chamada pela rota POST /pecas após validação
fn inserir_peca(conn: &Connection, peca: &Peca) {
    conn.execute(
        "INSERT INTO pecas (codigo_pi, nome, proxima_operacao, lote, data_entrada, data_saida)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &peca.codigo_pi,
            &peca.nome,
            format!("{:?}", peca.proxima_operacao), // Converte enum para String antes de salvar
            &peca.lote,
            &peca.data_entrada,
            &peca.data_saida,
        )
    ).expect("Erro ao inserir peça");
}

// Remove uma peça pelo id — chamada pela rota DELETE /pecas/{id}
fn deletar_peca(conn: &Connection, id: i32) {
    conn.execute(
        "DELETE FROM pecas WHERE id = ?1",
        [id],
    ).expect("Erro ao deletar peça");

    println!("Peça #{} removida.", id);
}

// Atualiza todos os campos de uma peça existente pelo id — chamada pela rota PUT /pecas/{id}
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
            format!("{:?}", peca.proxima_operacao), // Converte enum para String antes de salvar
            &peca.lote,
            &peca.data_entrada,
            &peca.data_saida,
            &peca.id,
        ),
    ).expect("Erro ao atualizar peça");

    println!("Peça #{} atualizada.", peca.id);
}

// Busca todas as peças do banco e retorna como Vec<Peca> — chamada pela rota GET /pecas
fn listar_pecas(conn: &Connection) -> Vec<Peca> {
    let mut stmt = conn
        .prepare("SELECT id, codigo_pi, nome, proxima_operacao, lote, data_entrada, data_saida FROM pecas")
        .expect("Erro ao preparar consulta");

    let pecas: Vec<Peca> = stmt
        .query_map([], |row| {
            Ok(Peca {
                id:               row.get(0)?,
                codigo_pi:        row.get(1)?,
                nome:             row.get(2)?,
                proxima_operacao: {
                    // Lê como String do banco e converte para o enum — erro explode se valor for inválido
                    let s: String = row.get(3)?;
                    s.parse().expect("Operação inválida no banco")
                },
                lote:             row.get(4)?,
                data_entrada:     formatar_data(&row.get::<_, String>(5)?),
                data_saida:       row.get::<_, Option<String>>(6)?.map(|d| formatar_data(&d)),
            })
        })
        .expect("Erro ao consultar peças")
        .filter_map(|r| r.ok())
        .collect();

    pecas
}

// Converte data do formato ISO (AAAA-MM-DD) para BR (DD/MM/AAAA) para exibição no frontend
fn formatar_data(data: &str) -> String {
    let partes: Vec<&str> = data.split('-').collect();
    if partes.len() == 3 {
        format!("{}/{}/{}", partes[2], partes[1], partes[0])
    } else {
        data.to_string() // retorna original se o formato for inesperado
    }
}

// ================================================================================================
// ROTAS HTTP — Recebem requisições do navegador e chamam as funções de banco


// Rota de verificação — GET /health — confirma que o servidor está no ar
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Servidor rodando!")
}

// Serve o arquivo index.html — GET / — ponto de entrada da interface web
async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open(PathBuf::from("static/index.html"))?)
}

// Retorna todas as peças do banco em JSON — GET /pecas — consumido pelo JS da tabela
async fn get_pecas(data: web::Data<AppState>) -> impl Responder {
    let conn = data.conn.lock().expect("Erro ao acessar banco");
    let lista = listar_pecas(&conn);
    HttpResponse::Ok().json(lista)
}

// Valida e insere uma nova peça — POST /pecas — chamada pelo formulário de cadastro
async fn post_peca(
    data: web::Data<AppState>,
    body: web::Json<PecaJson>,
) -> impl Responder {

    // Rejeita se codigo_pi não tiver exatamente 8 dígitos numéricos
    if body.codigo_pi.len() != 8 || !body.codigo_pi.chars().all(|c| c.is_ascii_digit()) {
        return HttpResponse::BadRequest().body("Código PI inválido — deve ter exatamente 8 dígitos numéricos.");
    }

    // Rejeita se lote não seguir o padrão H + 10 dígitos (ex: H0501001001)
    let lote_valido = body.lote.starts_with('H')
        && body.lote.len() == 11
        && body.lote[1..].chars().all(|c| c.is_ascii_digit());

    if !lote_valido {
        return HttpResponse::BadRequest().body("Lote inválido — fora do padrão H + 10 dígitos.");
    }

    let conn = data.conn.lock().expect("Erro ao acessar banco");

    // Converte PecaJson (sem id) para Peca completa — id=0 pois o banco gera o valor real
    let peca = Peca {
        id:               0,
        codigo_pi:        body.codigo_pi.clone(),
        nome:             body.nome.clone(),
        proxima_operacao: body.proxima_operacao.parse().expect("Operação inválida"),
        lote:             body.lote.clone(),
        data_entrada:     body.data_entrada.clone(),
        data_saida:       body.data_saida.clone(),
    };

    inserir_peca(&conn, &peca);
    HttpResponse::Ok().body("Peça cadastrada com sucesso!")
}

// Valida e atualiza uma peça existente — PUT /pecas/{id} — chamada pelo modal de edição
async fn put_peca(
    data: web::Data<AppState>,
    path: web::Path<i32>,
    body: web::Json<PecaJson>,
) -> impl Responder {

    // Mesmas validações do POST — garante integridade mesmo em edições
    if body.codigo_pi.len() != 8 || !body.codigo_pi.chars().all(|c| c.is_ascii_digit()) {
        return HttpResponse::BadRequest().body("Código PI inválido.");
    }

    let lote_valido = body.lote.starts_with('H')
        && body.lote.len() == 11
        && body.lote[1..].chars().all(|c| c.is_ascii_digit());

    if !lote_valido {
        return HttpResponse::BadRequest().body("Lote inválido.");
    }

    let conn = data.conn.lock().expect("Erro ao acessar banco");

    // Usa o id da URL para identificar qual peça atualizar no banco
    let peca = Peca {
        id:               path.into_inner(),
        codigo_pi:        body.codigo_pi.clone(),
        nome:             body.nome.clone(),
        proxima_operacao: body.proxima_operacao.parse().expect("Operação inválida"),
        lote:             body.lote.clone(),
        data_entrada:     body.data_entrada.clone(),
        data_saida:       body.data_saida.clone(),
    };

    atualizar_peca(&conn, &peca);
    HttpResponse::Ok().body("Peça atualizada.")
}

// Remove uma peça pelo id — DELETE /pecas/{id} — chamada pelo botão 🗑 da tabela
async fn delete_peca(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();
    let conn = data.conn.lock().expect("Erro ao acessar banco");
    deletar_peca(&conn, id);
    HttpResponse::Ok().body("Peça deletada.")
}

// ================================================================================================
// MAIN — Inicializa o banco e sobe o servidor com todas as rotas registradas


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Abre (ou cria) o arquivo de banco de dados SQLite na raiz do projeto
    let conn = Connection::open("pecas.db")
        .expect("Erro ao abrir banco");

    // Garante que a tabela existe antes de qualquer requisição chegar
    iniciar_banco(&conn);

    // Empacota a conexão no AppState com Mutex para compartilhar entre as rotas com segurança
    let app_state = web::Data::new(AppState {
        conn: Mutex::new(conn),
    });

    println!("Servidor rodando em http://localhost:8080");

    // Registra todas as rotas e sobe o servidor na porta 8080
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/",            web::get().to(index))       // Serve a página HTML
            .route("/health",      web::get().to(health_check))// Verifica se o servidor está no ar
            .route("/pecas",       web::get().to(get_pecas))   // Lista todas as peças em JSON
            .route("/pecas",       web::post().to(post_peca))  // Cadastra nova peça
            .route("/pecas/{id}",  web::put().to(put_peca))    // Atualiza peça pelo id
            .route("/pecas/{id}",  web::delete().to(delete_peca)) // Deleta peça pelo id
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}