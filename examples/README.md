# haygeom Examples

Examples of using the haygeom plugin for creating geographic maps.

## Organization

### basic/
Basic examples to test plugin functionality:
- `simple.hay` - Basic map creation
- `test.hay` - Rendering test with haygeobr data
- `debug.hay` - Debug issues

### brazil/
Brazil maps:
- `map_brazil.hay` - Map of Brazilian states
- `map_brazil_municipalities.hay` - Map of all Brazilian municipalities (5,570 municipalities)

### rs/
Rio Grande do Sul maps with various haygeobr data layers:
- `map_rs_municipalities.hay` - Map of 497 RS municipalities
- `map_rs_municipalities_pelotas.hay` - RS municipalities with Pelotas highlighted
- `map_rs_quilombolas.hay` - 27 quilombola lands in RS
- `map_rs_indigenous.hay` - 21 indigenous lands in RS
- `map_rs_favelas.hay` - 481 favelas/urban communities in RS
- `map_rs_pop_arrangement.hay` - 106 population arrangements in RS

### cleanup/
Temporary scripts for data verification:
- `check_favelas.hay` - Favelas data verification
- `check_population.hay` - Population data verification
- `find_pelotas.hay` - Search for Pelotas IBGE code

## Running the examples

```bash
# Navigate to the example directory
cd examples/brazil

# Run the script
hay map_brazil.hay
```

## Available haygeobr layers

haygeobr provides the following spatial data layers:

1. **Administrative Geography**
   - `read_country` - Brazil
   - `read_state` - States
   - `read_municipality` - Municipalities
   - `read_region` - Regions (N, NE, SE, S, CO)
   - `read_meso_region` - Mesoregions
   - `read_micro_region` - Microregions

2. **Special Lands**
   - `read_indigenous_land` - Indigenous lands
   - `read_quilombola_land` - Quilombola lands
   - `read_conservation_unit` - Conservation units

3. **Urban and Infrastructure**
   - `read_favelas` - Favelas and urban communities
   - `read_neighborhood` - Neighborhoods
   - `read_urban_area` - Urbanized areas
   - `read_municipal_seat` - Municipal seats
   - `read_metro_area` - Metropolitan areas
   - `read_pop_arrangement` - Population arrangements

4. **Health and Education**
   - `read_health_facilities` - Health facilities
   - `read_health_region` - Health regions
   - `read_schools` - Schools

5. **Other**
   - `read_biomes` - Biomes
   - `read_census_tract` - Census tracts
   - `read_polling_place` - Polling places
   - `read_semi_arid` - Semiarid
   - `read_amazonia_legal` - Legal Amazon
   - `read_disaster_risk_area` - Disaster risk areas
   - `read_immediate_region` - Immediate regions
   - `read_intermediate_region` - Intermediate regions
   - `read_statistical_grid` - Statistical grid
   - `read_weighting_area` - Weighting areas

**Note:** haygeobr provides only geographic data (geometries), not population, GDP, HDI, etc. To color by socioeconomic variables, you would need to integrate data from other sources (IBGE SIDRA, IPEA, etc.).
