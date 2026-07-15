use actix_web::{web, HttpResponse, Responder};

use crate::db::AppState;
use crate::models::{Peca, PecaJson, FiltrosPeca};
use crate::repository::{inserir_peca, deletar_peca, atualizar_peca, listar_pecas_filtradas};
use crate::validation::validar_peca;

// Rota de verificação — GET /health — confirma que o servidor está no ar
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Servidor rodando!")
}

// Retorna peças filtradas em JSON — GET /pecas?codigo_pi=...&nome=... — consumido pelo JS da tabela
pub async fn get_pecas(
    data:    web::Data<AppState>,
    filtros: web::Query<FiltrosPeca>,
) -> impl Responder {
    let conn = data.conn.lock().expect("Erro ao acessar banco");
    let lista = listar_pecas_filtradas(&conn, &filtros);
    HttpResponse::Ok().json(lista)
}

// Valida e insere uma nova peça — POST /pecas — chamada pelo formulário de cadastro
pub async fn post_peca(
    data: web::Data<AppState>,
    body: web::Json<PecaJson>,
) -> impl Responder {
    let proxima_operacao = match validar_peca(&body) {
        Ok(op) => op,
        Err(msg) => return HttpResponse::BadRequest().body(msg),
    };

    let conn = data.conn.lock().expect("Erro ao acessar banco");

    // Converte PecaJson (sem id) para Peca completa — id=0 pois o banco gera o valor real
    let peca = Peca {
        id:               0,
        codigo_pi:        body.codigo_pi.clone(),
        nome:             body.nome.clone(),
        proxima_operacao,
        lote:             body.lote.clone(),
        data_entrada:     body.data_entrada.clone(),
        data_saida:       body.data_saida.clone(),
    };

    inserir_peca(&conn, &peca);
    HttpResponse::Ok().body("Peça cadastrada com sucesso!")
}

// Valida e atualiza uma peça existente — PUT /pecas/{id} — chamada pelo modal de edição
pub async fn put_peca(
    data: web::Data<AppState>,
    path: web::Path<i32>,
    body: web::Json<PecaJson>,
) -> impl Responder {
    let proxima_operacao = match validar_peca(&body) {
        Ok(op) => op,
        Err(msg) => return HttpResponse::BadRequest().body(msg),
    };

    let conn = data.conn.lock().expect("Erro ao acessar banco");

    // Usa o id da URL para identificar qual peça atualizar no banco
    let peca = Peca {
        id:               path.into_inner(),
        codigo_pi:        body.codigo_pi.clone(),
        nome:             body.nome.clone(),
        proxima_operacao,
        lote:             body.lote.clone(),
        data_entrada:     body.data_entrada.clone(),
        data_saida:       body.data_saida.clone(),
    };

    atualizar_peca(&conn, &peca);
    HttpResponse::Ok().body("Peça atualizada.")
}

// Remove uma peça pelo id — DELETE /pecas/{id} — chamada pelo modal de exclusão
pub async fn delete_peca(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();
    let conn = data.conn.lock().expect("Erro ao acessar banco");
    deletar_peca(&conn, id);
    HttpResponse::Ok().body("Peça deletada.")
}