# Especificação Técnica: Purgar Gerador de Site Banner (Ticket #167)

## 1. Contexto e Motivação
O `cccp` continha um módulo (`src/site_banner.rs`) para gerar imagens PNG estáticas (1200x630) de pré-visualização Open Graph (`og_preview.png`) a partir dos metadados GOST do repositório, invocado via subcomando `cccp site banner` e implicitamente durante o deploy de site em `ci_pipeline.rs`.

A renderização crua de imagens bitmap e geração de Open Graph preview em nada diz respeito às responsabilidades de um orquestrador de ciclo de vida de desenvolvimento, commits e auditoria de repositório. O módulo trazia complexidade gráfica acessória, alocações de buffer desnecessárias e mais de 440 linhas de código sem uso prático.

## 2. Decisões Arquiteturais e Alternativas Descartadas (Para a Natasha)
- **Expurgo Completo do Módulo e Subcomando `site`:**
  - *Decisão:* Remover `src/site_banner.rs`, o subcomando `Site` (`Banner`) em `src/cli.rs`, o despacho correspondente em `src/main.rs`, o gancho implícito de geração em `src/ci_pipeline.rs` e o arquivo de teste `tests/site_banner_test.rs`.
  - *Alternativa descartada:* Manter a geração de banner delegada a um binário externo durante o deploy de páginas. Descartada pois a geração de peças gráficas para sites estáticos deve residir em ferramentas de design ou geradores de documentação/SSG (ex.: Zola, MkDocs, Astro), e não embutida na ferramenta de governança do repositório.

## 3. Resultados da Execução
- Eliminação de mais de 440 linhas de código gráfico e 140 linhas de testes associados.
- Simplificação da esteira de deploy de documentação (`deploy_site` em `ci_pipeline.rs`).
- CLI do `cccp` mais limpa e focada em seu propósito essencial.
