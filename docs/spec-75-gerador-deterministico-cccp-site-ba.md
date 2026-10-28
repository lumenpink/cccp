# Especificação Técnica — Ticket #75
## feat(site): gerador deterministico cccp site banner a partir de gost metadata

### 1. Contexto e Motivação
Em conformidade com as regras GOST-20, GOST-26 e a Seção 6.2 da norma Kollontai 2.0, todo repositório do ecossistema deve possuir um banner minimalista de documentação Open Graph em `site/assets/og_preview.png`.
Este banner é utilizado por indexadores, mensageiros e forjas soberanas como imagem de pré-visualização de link (1200x630 px com Safe Zone centralizada 4:3).
A geração deve ser estritamente determinística, nativa e sem dependências externas pesadas (como browsers headless ou Node.js/Puppeteer).

### 2. Requisitos e Subcomando

#### 2.1 Subcomando `cccp site banner`
- Gera `site/assets/og_preview.png` na raiz do repositório alvo (ou diretório atual).
- Cria automaticamente os diretórios `site/` e `site/assets/` se não existirem.
- Flags opcionais:
  - `--title <str>`: Sobrescreve o título.
  - `--tagline <str>`: Sobrescreve a tagline (truncada em até 65 caracteres).
  - `--output <path>`: Caminho de saída customizado (padrão: `site/assets/og_preview.png`).

#### 2.2 Layout e Desenho
- **Dimensões**: 1200 x 630 pixels.
- **Safe Zone**: Área centralizada 4:3 (840 x 630 ou margem segura interna).
- **Cores**:
  - `bg_color`: Fundo escuro derivado do tema ou `.gost/metadata.json` (padrão: `#111317`).
  - `accent_color`: Acento soviético/canônico (padrão: `#ef4444` ou configurado no metadata).
  - `text_color`: Branco/cinza claro `#f4f4f5`.
- **Conteúdo Visual**:
  - Emblema/estrela ou badge Gosplan superior.
  - Título do projeto central em fonte legível.
  - Tagline descritiva ($\le 65$ chars) e versão GOST ativa no rodapé da safe zone.
- **Formato**: Imagem PNG padrão RFC 2083 válida com chunks `IHDR`, `IDAT` e `IEND`.

### 3. Critérios de Aceitação
- Subcomando `cccp site banner` executável via CLI.
- Arquivo `site/assets/og_preview.png` gerado com dimensões exatas de 1200x630 pixels e assinatura PNG válida.
- Leitura transparente de `.gost/metadata.json` com fallback gracioso para nomes locais.
- Testes automatizados cobrindo geração, dimensões, integridade de PNG e sobreposição de opções.
