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

fn main() {
    let conn = Connection::open("pecas.db").expect("Erro ao abrir banco");

    iniciar_banco(&conn);

    let peca = Peca {
        id: 0, // o banco vai gerar o id real
        codigo_pi:        String::from("15570080"),
        nome:             String::from("31-2298 - Haste Curvada"),
        proxima_operacao: ProximaOperacao::Recalcar,
        lote:             String::from("H0501001001"),
        data_entrada:     String::from("01/07/2026"),
        data_saida:       None,
    };

    inserir_peca(&conn, &peca);

    let lista = listar_pecas(&conn);
    for p in &lista {
        let status = match &p.data_saida {
            Some(data) => format!("Saiu em: {}", data),
            None       => String::from("Em estoque"),
        };
        println!("#{} | {} | {} | {} | {} | {}", p.id, p.codigo_pi, p.nome, p.lote, p.data_entrada, status);
    }

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

