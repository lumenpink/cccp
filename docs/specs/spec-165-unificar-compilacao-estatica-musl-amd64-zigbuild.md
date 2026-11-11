# Especificação Técnica: Unificar Compilação Estática Musl AMD64 com Zigbuild (Ticket #165)

## 1. Contexto e Motivação
O job `Build & Publish CCCP Multiarch to Banca` na esteira de CI (`.forgejo/workflows/ci.yml`) falhava na etapa de compilação estática do target `x86_64-unknown-linux-musl`.

Ao compilar dependências em C (especificamente o interpretador Lua do `mlua_sys`), a presença da variável de ambiente `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER: "x86_64-linux-gnu-gcc"` forçava o uso da toolchain glibc host do Ubuntu. Sob essa toolchain, macros LFS ativavam referências a símbolos com sufixo 64 (`fopen64`, `fseeko64`, `tmpfile64`, `mkstemp64`), os quais não existem na musl libc estática.

## 2. Decisões Arquiteturais e Alternativas Descartadas (Para a Natasha)
- **Unificação com `cargo zigbuild`:**
  - *Decisão:* Remover a sobrescrita de linker glibc host e adotar `cargo zigbuild --release --target x86_64-unknown-linux-musl`, alinhando a arquitetura AMD64 com as arquiteturas ARM64 e Windows que já compilam com sucesso usando zigbuild.
  - *Alternativa descartada:* Instalar `musl-tools` / `musl-gcc` adicional no contêiner ou adicionar stubs de compatibilidade C. Descartada pois o zigbuild já se encontra instalado e homologado no contêiner `gost-ci-cross`.

## 3. Resultados da Execução
- Padronização completa do pipeline de release multiarch sob o `zigbuild`.
