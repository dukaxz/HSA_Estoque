// Guarda o id da peça sendo editada — null significa modo cadastro novo
let pecaEditandoId = null;

// Guarda a última lista carregada para permitir editar sem colocar JSON dentro do HTML
let pecasCarregadas = [];

// Guarda o timer do debounce dos filtros de texto
let debounceTimer = null;

function formatarDataBr(data) {
    if (!data) return "";
    const partes = data.split("-");
    if (partes.length !== 3) return data;
    return `${partes[2]}/${partes[1]}/${partes[0]}`;
}

// Adia a chamada de carregarPecas() em 300ms — evita disparar uma requisição a cada tecla digitada
function carregarPecasComDebounce() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(carregarPecas, 300);
}

// Localiza a peça pelo id e abre o modal de edição com os dados dela
function abrirModalPorId(id) {
    const peca = pecasCarregadas.find(p => p.id === id);
    if (peca) abrirModal(peca);
}

function abrirModal(peca = null) {
    pecaEditandoId = peca ? peca.id : null;

    // Muda o título do modal conforme o modo
    document.getElementById("modal-titulo").textContent = peca ? "Editar Peça" : "Cadastrar Nova Peça";

    // Preenche cada campo com o valor da peça ou vazio para novo cadastro
    document.getElementById("codigo_pi").value        = peca ? peca.codigo_pi : "";
    document.getElementById("nome").value             = peca ? peca.nome : "";
    document.getElementById("proxima_operacao").value = peca ? peca.proxima_operacao : "FormarPonta";
    document.getElementById("lote").value             = peca ? peca.lote : "";
    document.getElementById("data_entrada").value     = peca ? peca.data_entrada : "";
    document.getElementById("data_saida").value       = peca ? (peca.data_saida || "") : "";

    document.getElementById("modal-fundo").classList.add("aberto");
}

function fecharModal() {
    pecaEditandoId = null;
    document.getElementById("modal-fundo").classList.remove("aberto");
}

function abrirModalDeletar() {
    document.getElementById("id-deletar").value = "";
    document.getElementById("modal-deletar-fundo").classList.add("aberto");
}

function fecharModalDeletar() {
    document.getElementById("modal-deletar-fundo").classList.remove("aberto");
}

async function confirmarDelecao() {
    const id = document.getElementById("id-deletar").value;

    // Rejeita se o campo estiver vazio ou com valor inválido
    if (!id || id <= 0) {
        alert("Digite um ID válido.");
        return;
    }

    if (!confirm(`Tem certeza que deseja excluir a peça #${id}?`)) return;

    const res = await fetch(`/pecas/${id}`, { method: "DELETE" });

    if (!res.ok) {
        alert("Erro ao excluir peça.");
        return;
    }

    fecharModalDeletar();
    carregarPecas(); // Atualiza a tabela após a exclusão
}

// Monta o HTML de uma linha da tabela para a peça informada
function montarLinhaTabela(p) {
    const status = p.data_saida ? `Saiu em: ${formatarDataBr(p.data_saida)}` : "Em estoque";

    return `
        <tr>
            <td>${p.id}</td>
            <td>${p.codigo_pi}</td>
            <td>${p.nome}</td>
            <td>${p.proxima_operacao}</td>
            <td>${p.lote}</td>
            <td>${formatarDataBr(p.data_entrada)}</td>
            <td>${status}</td>
            <td>
                <button class="btn-acao" onclick="abrirModalPorId(${p.id})">Editar</button>
            </td>
        </tr>`;
}

// Busca peças no backend com os filtros ativos e reconstrói a tabela
async function carregarPecas() {
    // Lê o valor de cada campo de filtro
    const params = new URLSearchParams();
    const codigo   = document.getElementById("f-codigo").value.trim();
    const nome     = document.getElementById("f-nome").value.trim();
    const lote     = document.getElementById("f-lote").value.trim();
    const operacao = document.getElementById("f-operacao").value;
    const entrada  = document.getElementById("f-entrada").value;
    const saida    = document.getElementById("f-saida").value;

    // Só adiciona o parâmetro se o campo foi preenchido
    if (codigo)   params.append("codigo_pi", codigo);
    if (nome)     params.append("nome", nome);
    if (lote)     params.append("lote", lote);
    if (operacao) params.append("proxima_operacao", operacao);
    if (entrada)  params.append("data_entrada", entrada);
    if (saida)    params.append("data_saida", saida);

    const res  = await fetch(`/pecas?${params.toString()}`);
    const list = await res.json();

    pecasCarregadas = list;

    const corpo = document.getElementById("corpo-tabela");

    if (list.length === 0) {
        corpo.innerHTML = `<tr><td colspan="8">Nenhuma peça encontrada.</td></tr>`;
        return;
    }

    // Monta todas as linhas num array e injeta uma única vez —
    // evita reparse/reflow do DOM a cada iteração como acontecia com innerHTML += no loop
    corpo.innerHTML = list.map(montarLinhaTabela).join("");
}

function limparFiltros() {
    document.getElementById("f-codigo").value   = "";
    document.getElementById("f-nome").value     = "";
    document.getElementById("f-lote").value     = "";
    document.getElementById("f-operacao").value = "";
    document.getElementById("f-entrada").value  = "";
    document.getElementById("f-saida").value    = "";
    carregarPecas();
}

// Valida codigo_pi (8 dígitos) e lote (H + 10 dígitos) antes de enviar ao servidor
function validarCampos(peca) {
    const regexCodigo = /^\d{8}$/;
    const regexLote   = /^H\d{10}$/;

    if (!regexCodigo.test(peca.codigo_pi)) {
        alert("Código PI inválido — deve ter exatamente 8 dígitos numéricos.");
        return false;
    }
    if (!regexLote.test(peca.lote)) {
        alert("Lote inválido — deve começar com H seguido de 10 dígitos (ex: H0501001001).");
        return false;
    }
    if (!peca.nome || !peca.data_entrada) {
        alert("Preencha todos os campos obrigatórios.");
        return false;
    }
    return true;
}

// Decide entre POST (cadastro) ou PUT (edição) com base no pecaEditandoId
async function salvarPeca() {
    const data_saida = document.getElementById("data_saida").value;
    const peca = {
        codigo_pi:        document.getElementById("codigo_pi").value.trim(),
        nome:             document.getElementById("nome").value.trim(),
        proxima_operacao: document.getElementById("proxima_operacao").value,
        lote:             document.getElementById("lote").value.trim(),
        data_entrada:     document.getElementById("data_entrada").value,
        data_saida:       data_saida || null,
    };

    if (!validarCampos(peca)) return;

    const url = pecaEditandoId !== null ? `/pecas/${pecaEditandoId}` : "/pecas";
    const method = pecaEditandoId !== null ? "PUT" : "POST";

    const res = await fetch(url, {
        method:  method,
        headers: { "Content-Type": "application/json" },
        body:    JSON.stringify(peca),
    });

    if (!res.ok) {
        const msg = await res.text();
        alert(msg || "Erro ao salvar peça.");
        return;
    }

    fecharModal();
    carregarPecas(); // Recarrega a tabela para refletir a alteração
}

// Carrega a tabela automaticamente ao abrir a página
carregarPecas();