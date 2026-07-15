use rusqlite::Connection;

use crate::models::{Peca, FiltrosPeca};

// Insere uma nova peça no banco — chamada pela rota POST /pecas após validação
pub fn inserir_peca(conn: &Connection, peca: &Peca) {
    conn.execute(
        "INSERT INTO pecas (codigo_pi, nome, proxima_operacao, lote, data_entrada, data_saida)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &peca.codigo_pi,
            &peca.nome,
            peca.proxima_operacao.as_str(), // &str direto — sem alocar String a cada insert
            &peca.lote,
            &peca.data_entrada,
            &peca.data_saida,
        )
    ).expect("Erro ao inserir peça");
}

// Remove uma peça pelo id — chamada pela rota DELETE /pecas/{id}
pub fn deletar_peca(conn: &Connection, id: i32) {
    conn.execute(
        "DELETE FROM pecas WHERE id = ?1",
        [id],
    ).expect("Erro ao deletar peça");

    println!("Peça #{} removida.", id);
}

// Atualiza todos os campos de uma peça existente pelo id — chamada pela rota PUT /pecas/{id}
pub fn atualizar_peca(conn: &Connection, peca: &Peca) {
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
            peca.proxima_operacao.as_str(),
            &peca.lote,
            &peca.data_entrada,
            &peca.data_saida,
            &peca.id,
        ),
    ).expect("Erro ao atualizar peça");

    println!("Peça #{} atualizada.", peca.id);
}

// Monta SELECT com WHERE dinâmico — só passa parâmetros das colunas filtradas
pub fn listar_pecas_filtradas(conn: &Connection, f: &FiltrosPeca) -> Vec<Peca> {
    let mut sql = String::from(
        "SELECT id, codigo_pi, nome, proxima_operacao, data_entrada, lote, data_saida FROM pecas WHERE 1=1"
    );
    let mut params: Vec<String> = Vec::new();

    // Adiciona cláusula e parâmetro apenas se o filtro foi preenchido
    if let Some(v) = &f.codigo_pi        { sql.push_str(" AND codigo_pi LIKE ?");        params.push(format!("%{}%", v)); }
    if let Some(v) = &f.nome             { sql.push_str(" AND nome LIKE ?");             params.push(format!("%{}%", v)); }
    if let Some(v) = &f.lote             { sql.push_str(" AND lote LIKE ?");             params.push(format!("%{}%", v)); }
    if let Some(v) = &f.proxima_operacao { sql.push_str(" AND proxima_operacao LIKE ?"); params.push(format!("%{}%", v)); }
    if let Some(v) = &f.data_entrada     { sql.push_str(" AND data_entrada LIKE ?");     params.push(format!("%{}%", v)); }
    if let Some(v) = &f.data_saida       { sql.push_str(" AND data_saida LIKE ?");       params.push(format!("%{}%", v)); }

    // Mostra as peças mais recentes primeiro
    sql.push_str(" ORDER BY id DESC");

    let mut stmt = conn.prepare(&sql).expect("Erro ao preparar filtro");

    // Converte Vec<String> para o formato que o rusqlite aceita.
    // Número de "?" no SQL sempre bate com params.len() pois cada push_str vem
    // acompanhado do params.push() correspondente — é isso que evita o InvalidParameterCount.
    let params_ref: Vec<&dyn rusqlite::ToSql> = params
        .iter()
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();

    let pecas: Vec<Peca> = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(Peca {
                id:               row.get(0)?,
                codigo_pi:        row.get(1)?,
                nome:             row.get(2)?,
                proxima_operacao: {
                    let s: String = row.get(3)?;
                    s.parse().expect("Operação inválida no banco")
                },
                // Mantém data em ISO para o frontend conseguir preencher input type="date"
                data_entrada:     row.get(4)?,
                lote:             row.get(5)?,
                data_saida:       row.get(6)?,
            })
        })
        .expect("Erro ao filtrar peças")
        .filter_map(|r| r.ok())
        .collect();

    pecas
}