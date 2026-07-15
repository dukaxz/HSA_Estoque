use crate::models::{PecaJson, ProximaOperacao};

// Valida os campos recebidos do frontend antes de inserir ou atualizar no banco
pub fn validar_peca(body: &PecaJson) -> Result<ProximaOperacao, String> {
    // Rejeita se codigo_pi não tiver exatamente 8 dígitos numéricos
    if body.codigo_pi.len() != 8 || !body.codigo_pi.chars().all(|c| c.is_ascii_digit()) {
        return Err("Código PI inválido — deve ter exatamente 8 dígitos numéricos.".to_string());
    }

    // Rejeita se lote não seguir o padrão H + 10 dígitos (ex: H0501001001)
    let lote_valido = body.lote.starts_with('H')
        && body.lote.len() == 11
        && body.lote[1..].chars().all(|c| c.is_ascii_digit());

    if !lote_valido {
        return Err("Lote inválido — fora do padrão H + 10 dígitos.".to_string());
    }

    // Rejeita se campos obrigatórios estiverem vazios
    if body.nome.trim().is_empty() || body.data_entrada.trim().is_empty() {
        return Err("Nome e data de entrada são obrigatórios.".to_string());
    }

    // Converte a operação para enum e retorna erro amigável se vier inválida
    body.proxima_operacao.parse()
}