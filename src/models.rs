use std::str::FromStr;
use serde::{Serialize, Deserialize};

// ================================================================================================
// ENUM — Define as operações fixas do processo produtivo (espelho do processo real da HSA)

#[derive(Debug, Serialize, Deserialize)]
pub enum ProximaOperacao {
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

impl ProximaOperacao {
    // Converte a variante do enum para &str sem alocar String — usado ao salvar no banco.
    // Substitui o antigo format!("{:?}", enum), que alocava uma String a cada insert/update.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProximaOperacao::FormarPonta      => "FormarPonta",
            ProximaOperacao::Trefilar         => "Trefilar",
            ProximaOperacao::Endireitar       => "Endireitar",
            ProximaOperacao::Cortar           => "Cortar",
            ProximaOperacao::LimparBlank      => "LimparBlank",
            ProximaOperacao::FormarEndForming => "FormarEndForming",
            ProximaOperacao::Curvar           => "Curvar",
            ProximaOperacao::Recalcar         => "Recalcar",
            ProximaOperacao::Rebarbar         => "Rebarbar",
            ProximaOperacao::Zincar           => "Zincar",
            ProximaOperacao::Calibrar         => "Calibrar",
        }
    }
}

// Converte String vinda do banco ou do frontend ("Recalcar") para a variante do enum
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
            _ => Err(format!("Operação desconhecida: {}", s)),
        }
    }
}

// ================================================================================================
// STRUCTS — Moldes de dados usados no sistema

// Representa uma peça completa conforme salva no banco — usada em SELECT e UPDATE
#[derive(Debug, Serialize, Deserialize)]
pub struct Peca {
    pub id:               i32,             // Gerado automaticamente pelo SQLite (AUTOINCREMENT)
    pub codigo_pi:        String,          // 8 dígitos numéricos — identificador interno da peça
    pub nome:             String,          // Nome descritivo da peça (ex: 31-2298 - Haste Curvada)
    pub proxima_operacao: ProximaOperacao, // Operação seguinte no processo produtivo
    pub data_entrada:     String,          // Data em formato ISO (AAAA-MM-DD), ideal para input type="date"
    pub lote:             String,          // Lote de origem — padrão H + 10 dígitos (ex: H0501001001)
    pub data_saida:       Option<String>,  // None = peça ainda em estoque, Some = data em que saiu
}

// Representa os dados recebidos via JSON do formulário HTML — sem id (gerado pelo banco)
#[derive(Deserialize)]
pub struct PecaJson {
    pub codigo_pi:        String,
    pub nome:             String,
    pub proxima_operacao: String,          // Recebido como String e convertido para enum antes de salvar
    pub lote:             String,
    pub data_entrada:     String,
    pub data_saida:       Option<String>,
}

// Parâmetros de filtro recebidos na query string do GET /pecas
#[derive(Deserialize)]
pub struct FiltrosPeca {
    pub codigo_pi:        Option<String>,
    pub nome:             Option<String>,
    pub lote:             Option<String>,
    pub proxima_operacao: Option<String>,
    pub data_entrada:     Option<String>,
    pub data_saida:       Option<String>,
}