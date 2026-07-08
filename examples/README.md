# haygeom Examples

Exemplos de uso do plugin haygeom para criação de mapas geográficos.

## Organização

### basic/
Exemplos básicos para testar a funcionalidade do plugin:
- `simple.hay` - Criação básica de um mapa
- `test.hay` - Teste de renderização com dados do haygeobr
- `debug.hay` - Debug de problemas

### brazil/
Mapas do Brasil:
- `map_brazil.hay` - Mapa dos estados brasileiros
- `map_brazil_municipalities.hay` - Mapa de todos os municípios do Brasil (5.570 municípios)

### rs/
Mapas do Rio Grande do Sul com diferentes camadas de dados do haygeobr:
- `map_rs_municipalities.hay` - Mapa dos 497 municípios do RS
- `map_rs_municipalities_pelotas.hay` - Municípios do RS com Pelotas destacada
- `map_rs_quilombolas.hay` - 27 terras quilombolas no RS
- `map_rs_indigenous.hay` - 21 terras indígenas no RS
- `map_rs_favelas.hay` - 481 favelas/comunidades urbanas no RS
- `map_rs_pop_arrangement.hay` - 106 arranjos populacionais no RS

### cleanup/
Scripts temporários para verificação de dados:
- `check_favelas.hay` - Verificação de dados de favelas
- `check_population.hay` - Verificação de dados de população
- `find_pelotas.hay` - Busca de código IBGE de Pelotas

## Executando os exemplos

```bash
# Navegue para o diretório do exemplo
cd examples/brazil

# Execute o script
hay map_brazil.hay
```

## Camadas disponíveis no haygeobr

O haygeobr fornece as seguintes camadas de dados espaciais:

1. **Geografia Administrativa**
   - `read_country` - Brasil
   - `read_state` - Estados
   - `read_municipality` - Municípios
   - `read_region` - Regiões (N, NE, SE, S, CO)
   - `read_meso_region` - Mesorregiões
   - `read_micro_region` - Microrregiões

2. **Terras Especiais**
   - `read_indigenous_land` - Terras indígenas
   - `read_quilombola_land` - Terras quilombolas
   - `read_conservation_unit` - Unidades de conservação

3. **Urbano e Infraestrutura**
   - `read_favelas` - Favelas e comunidades urbanas
   - `read_neighborhood` - Bairros
   - `read_urban_area` - Áreas urbanizadas
   - `read_municipal_seat` - Sedes municipais
   - `read_metro_area` - Áreas metropolitanas
   - `read_pop_arrangement` - Arranjos populacionais

4. **Saúde e Educação**
   - `read_health_facilities` - Estabelecimentos de saúde
   - `read_health_region` - Regiões de saúde
   - `read_schools` - Escolas

5. **Outros**
   - `read_biomes` - Biomas
   - `read_census_tract` - Setores censitários
   - `read_polling_place` - Locais de votação
   - `read_semi_arid` - Semiárido
   - `read_amazonia_legal` - Amazônia Legal
   - `read_disaster_risk_area` - Áreas de risco de desastres
   - `read_immediate_region` - Regiões imediatas
   - `read_intermediate_region` - Regiões intermediárias
   - `read_statistical_grid` - Grade estatística
   - `read_weighting_area` - Áreas de ponderação

**Nota:** O haygeobr fornece apenas dados geográficos (geometrias), não dados de população, PIB, IDH, etc. Para colorir por variáveis socioeconômicas, seria necessário integrar dados de outras fontes (IBGE SIDRA, IPEA, etc.).
