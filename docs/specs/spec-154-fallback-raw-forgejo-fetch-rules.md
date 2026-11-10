# Especificação do Ticket #154: Fallback Raw Forgejo em fetch_rules_for_layer

## Objetivo
Garantir a montagem hermética e resiliente de `rules.toml` (`assemble_gost_specs`) mesmo quando a CDN ou o domínio de publicação HTTP do perfil estiver indisponível ou retornar 404 (ex.: em ambientes de CI isolados ou CDN fria).

## Requisitos
1. Na função `fetch_rules_for_layer` (`src/profile.rs`), após montar a URL pretendida (`rules_url`), deve existir uma lista ordenada de URLs candidatas contendo `rules_url` e o fallback canônico via Forgejo raw (`https://gambi.2lp.in/lumen/{repo}/raw/branch/main/rules.toml`).
2. Se a primeira URL retornar erro HTTP (como 404) ou HTML/vazio, a função tenta a próxima candidata antes de desistir.
3. Repositórios suportados no fallback canônico:
   - `lilya` -> `https://gambi.2lp.in/lumen/gost-profile-lilya/raw/branch/main/rules.toml`
   - `kollontai` -> `https://gambi.2lp.in/lumen/gost-profile-kollontai/raw/branch/main/rules.toml`
   - `gosplan` -> `https://gambi.2lp.in/lumen/gosplan/raw/branch/main/rules.toml`
   - `gost` -> `https://gambi.2lp.in/lumen/gost/raw/branch/main/rules.toml`
4. Teste unitário/integrado deve validar que o fallback é acionado com sucesso quando a URL direta é inválida.
