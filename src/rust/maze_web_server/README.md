# `maze_web_server` Crate

## Introduction

The `maze_web_server` crate is written in `Rust` and is a web server console application that hosts the `REST`-compliant `Maze Web API` and the React [`maze_web_server`](../../react/maze_web_server/README.md) front-end.

It leverages the `Rust` library crates for calculation and generation ([`maze`](../maze/README.md)) and storage ([`storage`](../storage/README.md)). It then exposes them using [`actix`](https://actix.rs/) to serve the API and [`utoipa`](https://docs.rs/utoipa/latest/utoipa/) to publish it as an [`OpenAPI`](https://www.openapis.org/)-compliant interface for use in third party products such as [`Swagger`](https://swagger.io/). 

In addition to the API interfaces, it also supports the following documentation and game endpoints:

| EndPoint                  | Description
|:--------------------------|:------------
| `/api-docs/v1/rapidoc`    | [RapiDoc](https://github.com/rapi-doc/RapiDoc) 
| `/api-docs/v1/redoc`      | [ReDoc](https://redocly.com/)
| `/api-docs/v1/swagger-ui/`| [Swagger UI](https://swagger.io/tools/swagger-ui/)
| `/game/`                  | First-person 3D maze game — [`Bevy`](https://bevyengine.org/) WASM binary compiled from [`maze_game_bevy_wasm`](../maze_game_bevy_wasm/README.md); loads maze via `/api/v1/mazes/{id}` with bearer token; touch D-pad on mobile

These pages provide interactive documentation and, in the case of the `RapiDoc` and `Swagger UI` interfaces, the ability to manually tests the API as well.

## Getting Started

### Build
To build the `maze_web_server` application, run the following from within the `maze_web_server` directory:
```
cargo build
```

### Testing
To test the `maze_web_server` application, run the following from within the `maze_web_server` directory:
```
cargo test
```

### Running

Run with:
```
cargo run
```

This will utilise the following self-signed certificate files:

|  Name         | Description         | Format
|:--------------|:--------------------|:------
| `cert.pem`    | Certficate file     | `PKCS#8`
| `key.pem`     | Private key file    | `PKCS#8`

These curremtly have an expiry of `07-APR-2027`. Hence, they will need to be renewed after this time has elapsed by using tools such as `openssl` or, for production, a trusted Certificate Authority (e.g. Let's Encrypt). 

Any new files must be generated in `PKCS#8` format. The following command using `openssl` (1.11 and later) will regenerate these files with a `365` day expiry in this format:

```
openssl req -x509 -nodes -newkey rsa:2048 -keyout key.pem -out cert.pem -days 365
```

In addition, the following files are included for development/testing purposes:

|  Name             | Description             | Format
|:------------------|:------------------------|:------
| `empty_cert.pem`  | Empty certficate file   | `Text`
| `empty_key.pem`   | Empty private key file  | `Text`

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `maze_web_server` directory:
```
cargo doc --open
```

### Configuration

The following configuration settings exist:

| Type     | Name         | Type    | Default Value    | Environment Variable Override
|:---------|:-------------|:--------|:-----------------|:------------
| Global   | `port`             | Integer | `8443`   | `MAZE_WEB_SERVER_PORT`
| Security | `cert_file`        | Text    | `cert.pem` | `MAZE_WEB_SERVER_SECURITY_CERT_FILE`
|          | `key_file`         | Text    | `key.pem`  | `MAZE_WEB_SERVER_SECURITY_KEY_FILE`
|          | `security.login_expiry_hours` | Integer | `24`  | (config-file only)
|          | `security.password_hash.mem_cost`    | Integer | `65536` | (config-file only)
|          | `security.password_hash.time_cost`   | Integer | `3`     | (config-file only)
|          | `security.password_hash.lanes`       | Integer | `4`     | (config-file only)
|          | `security.password_hash.hash_length` | Integer | `32`    | (config-file only)
| Static   | `static_dir`       | Text    | `static`          | `MAZE_WEB_SERVER_STATIC_DIR`
| Logging  | `log_dir`          | Text    | `logs`            | `MAZE_WEB_SERVER_LOGGING_LOG_DIR`
|          | `log_level`        | Text    | `info`            | `MAZE_WEB_SERVER_LOGGING_LOG_LEVEL`
|          | `log_file_prefix`  | Text    | `maze_web_server_`| `MAZE_WEB_SERVER_LOGGING_LOG_FILE_PREFIX`
| Features | `allow_signup`     | Boolean | `true`            | `MAZE_WEB_SERVER_FEATURES_ALLOW_SIGNUP`
| Game (Play 3D) | `game.play3d.title`                         | Text    | `Maze 3D`  | (config-file only)
|                | `game.play3d.<difficulty>.rows`             | Integer | `8`        | (config-file only)
|                | `game.play3d.<difficulty>.cols`             | Integer | `8`        | (config-file only)
|                | `game.play3d.<difficulty>.timer_seconds`    | Integer | `120`      | (config-file only)
|                | `game.play3d.<difficulty>.seed`             | Integer | `0`        | (config-file only — fixed per difficulty for leaderboard fairness)
|                | `game.play3d.<difficulty>.min_solution_length` | Integer | `0`     | (config-file only — `0` = no minimum; maps to the maze crate's `min_spine_length`)
|                | `game.play3d.<difficulty>.minimap_cell_px`  | Integer | `10`    | (config-file only — on-screen pixel size of each minimap cell)
|                | `game.play3d.<difficulty>.minimap_radius`   | Integer | `5`     | (config-file only — cells visible each direction from the player; minimap shows a 2r+1 square)
|                | `game.play3d.<difficulty>.title`            | Text (optional) | (falls back to `game.play3d.title`) | (config-file only)
|                | `game.play3d.<difficulty>.mode`             | Text    | `Play`     | (config-file only — free-text label shown in the in-game status bar, e.g. `Easy` / `Tricky` / `Hard`)
|                | `game.play3d.<difficulty>.landmarks.wall_tint` | Boolean | `true` | (config-file only — when `true`, add random wall tinting; bypassed when `wall_material_variation` is `true`)
|                | `game.play3d.<difficulty>.landmarks.dead_end_objects` | Boolean | `true` | (config-file only — when `true`, place random objects in dead-end cells)
|                | `game.play3d.<difficulty>.landmarks.wall_decorations` | Boolean | `true` | (config-file only — when `true`, add random wall decorations )
|                | `game.play3d.<difficulty>.landmarks.floor_accents` | Boolean | `true` | (config-file only — when `true`, place flat accents on the floor of 3- and 4-way junction cells)
|                | `game.play3d.<difficulty>.landmarks.wall_material_variation` | Boolean | `true` | (config-file only — when `true`, split the maze into a 2×2 NW/NE/SW/SE grid and render each quadrant with its own wall material (brick / dressed stone / wood / cobblestone); supersedes `wall_tint`)
|                | `game.play3d.<difficulty>.sky_type` | Text (`night` / `sunrise` / `day` / `sunset` / `dungeon` / `chamber`) | `night` | (config-file only — atmospheric sky mode; `dungeon` caps the maze with a dark-rock ceiling and `chamber` with a ceiling in the wall material, instead of an open sky; unknown values fall back to `night`)
|                | `game.play3d.<difficulty>.wall_type` | Text (`brick` / `dressed_stone` / `wood` / `cobblestone` / `water` / `lava` / `iron_fence` / `random`) | `brick` | (config-file only — per-maze wall type. The solid textures use the per-cell tinted path (bypassed when `wall_material_variation` is `true`); the non-occluding types `water` / `lava` / `iron_fence` turn every wall cell into a floor-level pool or see-through bars. `random` rolls one of the seven types **per level** (seeded off the maze), so each level reads as one coherent style. Unknown values fall back to `brick`)
|                | `game.play3d.<difficulty>.perimeter_walls` | Boolean | `true` | (config-file + Play 3D launch modal — whether the maze perimeter is walled at the grid edge under an open sky. Enclosed skies (`dungeon` / `chamber`) always wall it regardless; `false` shows the skybox past an open-sky edge)
|                | `game.play3d.<difficulty>.door_style` | Text (`swing` / `slide` / `portcullis` / `dissolve`) | `swing` | (config-file only — door open-animation style; applies to authored mazes containing door cells; unknown values fall back to `swing`)
|                | `game.play3d.<difficulty>.key_holder` | Text (`pedestal` / `chest` / `floating_key` / `random`) | `pedestal` | (config-file only — key-holder style for key cells; `random` rolls a rig **per cell** (seeded off the maze); unknown values fall back to `pedestal`)
|                | `game.play3d.<difficulty>.door_count` | Integer | `0` | (config-file only — number of real path doors (each paired with one key) the generator auto-places on the maze's spine; clamped to 8 and to what the maze can hold; `0` = a lock-free maze; combined with `spare_doors` and `spare_keys` so that `2*door_count + spare_doors + spare_keys ≤ 16`)
|                | `game.play3d.<difficulty>.spare_doors` | Integer | `0` | (config-file only — number of decoy doors planted on off-spine branches; visually indistinguishable from real path doors so opening one burns a key the player may have needed for a real door, potentially stranding them; clamped to 8 and to feasibility; capped jointly with `door_count` and `spare_keys` at `2*door_count + spare_doors + spare_keys ≤ 16`)
|                | `game.play3d.<difficulty>.spare_keys` | Integer | `0` | (config-file only — number of spare keys planted on off-spine branches, giving the player a budget to spend on decoys before they risk stranding; capped jointly with `door_count` and `spare_doors` at `2*door_count + spare_doors + spare_keys ≤ 16`)
|                | `game.play3d.<difficulty>.enemy_count` | Integer | `0` | (config-file only — number of enemies (`'E'` cells) the generator auto-places on this difficulty's maze; clamped to 8 and to the available eligible cells; `0` = no enemies)
|                | `game.play3d.<difficulty>.health_count` | Integer | `0` | (config-file only — number of health pickups (`'H'` cells) the generator auto-places; clamped to 8 and to the available eligible cells; `0` = none)
|                | `game.play3d.<difficulty>.treasure_count` | Integer | `0` | (config-file only — number of treasure cells (`'T'`) the generator auto-places, dead-end-first and type-weighted; clamped to 12 and to the available eligible cells; `0` = none)
|                | `game.play3d.<difficulty>.enemy_type` | Text (`goblin` / `ghost` / `random`) | `goblin` | (config-file only — enemy rig kind to spawn at every `'E'` cell; `random` rolls a rig **per cell** (seeded off the maze); unknown values fall back to `goblin`)
|                | `game.play3d.<difficulty>.health_style` | Text (`heart` / `potion` / `random`) | `heart` | (config-file only — health-pickup rig kind to spawn at every `'H'` cell; `random` rolls a rig **per cell** (seeded off the maze); unknown values fall back to `heart`)
|                | `game.play3d.<difficulty>.enemy_move_period_ms` | Integer | `1500` | (config-file only — how often each enemy advances one cell, in milliseconds of real-game time; lower = harder)
|                | `game.play3d.<difficulty>.max_hp` | Integer | `3` | (config-file only — player's HP cap and starting HP for this difficulty)
|                | `game.play3d.<difficulty>.levels.count` | Integer | `1` | (config-file only — number of stacked maze levels in a run; `1` = single-level (no transitions); clamped to the renderer's maximum of 20. Each interim level's finish becomes the entry to the next; only the final level completes the run. Score and HP carry across levels)
|                | `game.play3d.<difficulty>.levels.finish_type` | Text (`ladder` / `portal` / `random`) | `ladder` | (config-file only — the rig drawn at each interim finish instead of the gold orb; `random` picks a rig per interim finish cell, seeded; unknown values fall back to `ladder`; inert when `count == 1`)
|                | `game.play3d.<difficulty>.levels.difficulty_change` | Text (`same` / `easier` / `harder`) | `easier` | (config-file only — how difficulty changes as the player climbs; `easier` = hardest at the bottom, easing upward (enemy count is the lever, footprint uniform); `harder` = the reverse; `same` = every level equally hard; unknown values fall back to `easier`; inert when `count == 1`)
|                | `game.play3d.<difficulty>.levels.reset_bag` | Boolean | `true` | (config-file only — whether the player's bag (keys etc.) resets at each level; `true` = every level self-contained; `false` carries the whole bag forward; inert when `count == 1`)
|                | `game.play3d.<difficulty>.levels.taper` | Boolean | `false` | (config-file only — when `true`, upper levels get progressively smaller footprints (positioned per `alignment`) so the stack opens up and a climb eases as it rises; `false` keeps every level the full footprint. Operator choice, independent of `sky_type`; inert when `count == 1`)
|                | `game.play3d.<difficulty>.levels.alignment` | Text (`edge` / `centre` / `random_base` / `random_level`) | `edge` | (config-file only — how a smaller upper (tapered) level sits over the level below; `edge` corner-aligns all layers, `centre` centres each; the random modes pick edge/centre per level from the seed — `random_base` measures each from the ground layer (a corner-stacked level may overhang a centred one below it), `random_level` measures each within the level below (every level nests); unknown values fall back to `edge`; inert when `count == 1` or `taper` is off)
|                | `game.play3d.<difficulty>.levels.perimeter_random` | Boolean | `false` | (config-file only — when `true`, each level randomises its perimeter walls on/off independently; when `false`, every level uses the difficulty's `perimeter_walls`; inert when `count == 1`)
|                | `game.play3d.<difficulty>.levels.hide_completed_enemies` | Boolean | `false` | (config-file only — when `true`, a completed lower level's enemies are despawned once the player climbs past it; when `false`, they idle in place; inert when `count == 1`)
|                | `game.play3d.<difficulty>.levels.top.sky_type` / `.perimeter_walls` | Text / Boolean | inherit base | (config-file only — optional `[…levels.top]` scene override for the final (top) level; each field falls back to the base difficulty's value when unset; only meaningful when `count > 1`)
| OAuth    | `oauth.enabled`    | Boolean | `false`           | `MAZE_WEB_SERVER_OAUTH_ENABLED`
|          | `oauth.connector`  | Text (`internal` / `auth0`) | `internal` | `MAZE_WEB_SERVER_OAUTH_CONNECTOR`
|          | `oauth.mobile_redirect_scheme` | Text | `maze-app` | `MAZE_WEB_SERVER_OAUTH_MOBILE_REDIRECT_SCHEME`
|          | `oauth.internal.providers.<name>.enabled` | Boolean | `false` | (config-file only)
|          | `oauth.internal.providers.<name>.display_name` | Text | (empty) | (config-file only)
|          | `oauth.internal.providers.<name>.client_id` | Text | (empty) | (config-file only)
|          | `oauth.internal.providers.<name>.client_secret_env` | Text | (empty) | (config-file only — names the env var that holds the secret)
|          | `oauth.internal.providers.<name>.redirect_uri` | Text | (empty) | (config-file only)
|          | `oauth.internal.providers.<name>.client_secret` | Text | (env-var only — never read from config files) | named by `client_secret_env`
| Storage  | `storage.type`               | Text (`file` / `sql`) | `file` | `MAZE_WEB_SERVER_STORAGE_TYPE`
|          | `storage.file.data_dir`      | Text    | `data`  | `MAZE_WEB_SERVER_STORAGE_FILE_DATA_DIR`
|          | `storage.sql.driver`         | Text (`sqlite` / `postgres` / `mysql`) | `sqlite` | `MAZE_WEB_SERVER_STORAGE_SQL_DRIVER`
|          | `storage.sql.host`           | Text    | (empty) | `MAZE_WEB_SERVER_STORAGE_SQL_HOST`
|          | `storage.sql.port`           | Integer | `0`     | `MAZE_WEB_SERVER_STORAGE_SQL_PORT`
|          | `storage.sql.database`       | Text    | (empty) | `MAZE_WEB_SERVER_STORAGE_SQL_DATABASE`
|          | `storage.sql.username`       | Text    | (empty) | `MAZE_WEB_SERVER_STORAGE_SQL_USERNAME`
|          | `storage.sql.password`       | Text    | (env-var only — never read from config files) | `MAZE_WEB_SERVER_STORAGE_SQL_PASSWORD`
|          | `storage.sql.path`           | Text    | `maze.db` | `MAZE_WEB_SERVER_STORAGE_SQL_PATH`
|          | `storage.sql.max_connections` | Integer | `5`    | `MAZE_WEB_SERVER_STORAGE_SQL_MAX_CONNECTIONS`
|          | `storage.sql.auto_create_database` | Boolean | `false` | `MAZE_WEB_SERVER_STORAGE_SQL_AUTO_CREATE_DATABASE`
|          | `storage.sql.require_tls`    | Boolean | `false` | `MAZE_WEB_SERVER_STORAGE_SQL_REQUIRE_TLS`
|          | `storage.sql.ca_cert_path`   | Text    | (empty) | `MAZE_WEB_SERVER_STORAGE_SQL_CA_CERT_PATH`
|          | `storage.sql.connect_timeout_secs` | Integer | `10` | `MAZE_WEB_SERVER_STORAGE_SQL_CONNECT_TIMEOUT_SECS`
|          | `storage.sql.idle_timeout_secs` | Integer | `600` | `MAZE_WEB_SERVER_STORAGE_SQL_IDLE_TIMEOUT_SECS`
|          | `storage.sql.acquire_timeout_secs` | Integer | `30` | `MAZE_WEB_SERVER_STORAGE_SQL_ACQUIRE_TIMEOUT_SECS`
| Comms    | `comms.enabled`              | Boolean | `false` | `MAZE_WEB_SERVER_COMMS_ENABLED`
|          | `comms.public_base_url`      | Text    | (empty) | `MAZE_WEB_SERVER_COMMS_PUBLIC_BASE_URL`
|          | `comms.branding.company_name`    | Text | (empty) | `MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_NAME`
|          | `comms.branding.company_address` | Text | (empty) | `MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_ADDRESS`
|          | `comms.branding.company_url`     | Text | (falls back to `comms.public_base_url`) | `MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_URL`
|          | `comms.branding.logo_url`        | Text | (empty) | `MAZE_WEB_SERVER_COMMS_BRANDING_LOGO_URL`
|          | `comms.branding.app_name`        | Text | (empty — falls back to `comms.email.from_name`, then `comms.branding.company_name`) | `MAZE_WEB_SERVER_COMMS_BRANDING_APP_NAME`
|          | `comms.email.provider`           | Text (`stub` / `mailgun` / `smtp_oauth2`) | `stub` | `MAZE_WEB_SERVER_COMMS_EMAIL_PROVIDER`
|          | `comms.email.from`               | Text | (empty) | `MAZE_WEB_SERVER_COMMS_EMAIL_FROM`
|          | `comms.email.from_name`          | Text | `The Maze Team` | `MAZE_WEB_SERVER_COMMS_EMAIL_FROM_NAME`
|          | `comms.email.templates_dir`      | Text | `config/email_templates` | `MAZE_WEB_SERVER_COMMS_EMAIL_TEMPLATES_DIR`
|          | `comms.email.audit.record_unknown_password_reset_requests` | Boolean | `false` | `MAZE_WEB_SERVER_COMMS_EMAIL_AUDIT_RECORD_UNKNOWN_PASSWORD_RESET_REQUESTS`
|          | `comms.email.mailgun.domain`     | Text | (empty) | `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_DOMAIN`
|          | `comms.email.mailgun.region`     | Text (`us` / `eu`) | `us` | `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_REGION`
|          | `comms.email.mailgun.api_key`    | Text | (env-var only — never read from config files) | `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY`
|          | `comms.email.smtp_oauth2.host`   | Text | (empty) | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_HOST`
|          | `comms.email.smtp_oauth2.port`   | Integer | `587` | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_PORT`
|          | `comms.email.smtp_oauth2.tls`    | Text (`starttls` / `implicit` / `plain`) | `starttls` | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_TLS`
|          | `comms.email.smtp_oauth2.username` | Text | (empty) | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_USERNAME`
|          | `comms.email.smtp_oauth2.vendor` | Text (`microsoft` / `google` / `google_personal`) | `microsoft` | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_VENDOR`
|          | `comms.email.smtp_oauth2.microsoft.tenant_id` | Text | (empty) | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_TENANT_ID`
|          | `comms.email.smtp_oauth2.microsoft.client_id` | Text | (empty) | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_ID`
|          | `comms.email.smtp_oauth2.microsoft.scopes`    | Array of Text | `["https://outlook.office.com/.default"]` | (config-file only)
|          | `comms.email.smtp_oauth2.microsoft.client_secret` | Text | (env-var only — never read from config files) | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_SECRET`
|          | `comms.email.smtp_oauth2.google.service_account_json_path` | Text | (empty) | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_SERVICE_ACCOUNT_JSON_PATH`
|          | `comms.email.smtp_oauth2.google.delegated_subject` | Text | (empty) | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_DELEGATED_SUBJECT`
|          | `comms.email.smtp_oauth2.google.scopes`            | Array of Text | `["https://www.googleapis.com/auth/gmail.send"]` | (config-file only)
|          | `comms.email.smtp_oauth2.google_personal.client_id` | Text | (empty) | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_PERSONAL_CLIENT_ID`
|          | `comms.email.smtp_oauth2.google_personal.scopes`    | Array of Text | `["https://mail.google.com/"]` | (config-file only)
|          | `comms.email.smtp_oauth2.google_personal.client_secret` | Text | (env-var only — never read from config files) | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_PERSONAL_CLIENT_SECRET`
|          | `comms.email.smtp_oauth2.google_personal.refresh_token` | Text | (env-var only — never read from config files) | `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_PERSONAL_REFRESH_TOKEN`

These can also be set in a local configuration file called `config.toml` as follows

```toml
port = 8443

[security]
cert_file = "cert.pem"
key_file = "key.pem"

[logging]
log_dir = "logs"
log_level = "info"

[features]
allow_signup = true

# Play 3D presets are fetched by /game/index.html on every game session, so a
# change here propagates to every client without a rebuild. `seed` is fixed per
# difficulty so leaderboard records on the same difficulty share the same maze
# layout from day 1. `min_solution_length` is plumbed through to the maze
# crate's `min_spine_length` generator option. `title` is the in-game splash
# text — set it at the parent level or per difficulty.
[game.play3d]
title = "Maze 3D"

[game.play3d.easy]
mode = "Easy"
rows = 8
cols = 8
timer_seconds = 120
seed = 8080808
min_solution_length = 30
minimap_cell_px = 10
minimap_radius = 5

sky_type = "night"
wall_type = "brick"
# Real path doors (each with one key) auto-placed on the spine. Easy mode
# ships with a couple; spares stay off so easy mode carries no strand risk.
door_count = 2
spare_doors = 0
spare_keys = 0
# Auto-placed enemies (`'E'`) and health pickups (`'H'`). Easy mode ships
# with one goblin and two hearts so the player learns the mechanic.
enemy_count = 1
health_count = 2
treasure_count = 3
enemy_type = "goblin"
health_style = "heart"
enemy_move_period_ms = 1800
max_hp = 3

[game.play3d.easy.landmarks]
wall_tint = true
dead_end_objects = true
wall_decorations = true
floor_accents = true
wall_material_variation = true

[game.play3d.easy.levels]
# Multi-level run settings. count = 1 is a single-level game (the default).
# finish_type: ladder | portal | random. difficulty_change: same | easier |
# harder. alignment: edge | centre | random_base | random_level. perimeter_random randomises each
# level's perimeter walls when true. hide_completed_enemies despawns a lower
# level's enemies once you climb past it. The optional [levels.top] table
# overrides the top level's sky_type / perimeter_walls when count > 1.
count = 1
finish_type = "ladder"
difficulty_change = "easier"
reset_bag = true
alignment = "edge"
perimeter_random = false
hide_completed_enemies = false

[game.play3d.tricky]
mode = "Tricky"
rows = 15
cols = 15
timer_seconds = 240
seed = 15151515
min_solution_length = 90
minimap_cell_px = 10
minimap_radius = 5

sky_type = "night"
wall_type = "brick"
# Tricky raises real doors to 3 and introduces decoys + a spare. With one
# spare key, the player can absorb a single decoy mistake; a second wrong
# door strands them.
door_count = 3
spare_doors = 2
spare_keys = 1
enemy_count = 3
health_count = 3
treasure_count = 5
enemy_type = "goblin"
health_style = "heart"
enemy_move_period_ms = 1500
max_hp = 3

[game.play3d.tricky.landmarks]
wall_tint = true
dead_end_objects = true
wall_decorations = true
floor_accents = true
wall_material_variation = true

[game.play3d.tricky.levels]
count = 2
finish_type = "ladder"
difficulty_change = "easier"
reset_bag = true
alignment = "edge"
perimeter_random = false

[game.play3d.hard]
mode = "Hard"
rows = 25
cols = 25
timer_seconds = 420
seed = 25252525
min_solution_length = 220
minimap_cell_px = 10
minimap_radius = 5

sky_type = "night"
wall_type = "brick"
# Hard adds a third decoy on top of one extra real path door, keeping the
# same one-mistake margin as tricky but with more decoy choices to
# navigate around.
door_count = 4
spare_doors = 3
spare_keys = 1
enemy_count = 5
health_count = 4
treasure_count = 8
enemy_type = "goblin"
health_style = "heart"
enemy_move_period_ms = 1200
max_hp = 3

[game.play3d.hard.landmarks]
wall_tint = true
dead_end_objects = true
wall_decorations = true
floor_accents = true
wall_material_variation = true

[game.play3d.hard.levels]
count = 3
finish_type = "ladder"
difficulty_change = "easier"
reset_bag = true
alignment = "edge"
perimeter_random = false
# Optional top-level scene override (only meaningful when count > 1):
#[game.play3d.hard.levels.top]
#sky_type = "day"
#perimeter_walls = false

[storage]
# Backend selector: "file" (on-disk JSON layout) or "sql" (SQLite/Postgres/MySQL).
type = "file"

[storage.file]
# Directory under which user/maze data is stored, relative to the working
# directory or absolute.
data_dir = "data"

# ---- SQL backend ----
# To switch to a SQL backend, set type = "sql" above and uncomment the
# block below. Driver selection happens at runtime — one binary supports
# all three engines via SQLx's Any backend. The connection URL is
# assembled from these fields at startup. The password is *never*
# stored here — set MAZE_WEB_SERVER_STORAGE_SQL_PASSWORD instead
# (sqlite is exempt — it has no network user).
# [storage.sql]
# driver = "postgres"            # "postgres", "mysql", or "sqlite"
# host = "your-db-host"          # postgres / mysql only
# port = 5432                    # postgres / mysql only
# database = "your_database"     # postgres / mysql only
# username = "your_app_user"     # postgres / mysql only
# path = "your_database.db"      # sqlite only
# max_connections = 5
# auto_create_database = false   # sqlite + dev only — cloud creds rarely have the privilege
# require_tls = false            # set true for any host beyond localhost
# ca_cert_path = ""              # optional CA bundle for full TLS verification
# connect_timeout_secs = 10
# idle_timeout_secs = 600
# acquire_timeout_secs = 30

[oauth]
enabled = false
connector = "internal"
mobile_redirect_scheme = "maze-app"

# Internal connector: speaks OAuth/OIDC directly to each provider.
# Client secrets are NOT stored here — set the env var named in
# `client_secret_env` (e.g. MAZE_OAUTH_GOOGLE_SECRET).
[oauth.internal.providers.google]
enabled = false
display_name = "Google"
client_id = ""
client_secret_env = "MAZE_OAUTH_GOOGLE_SECRET"
redirect_uri = "https://your-host:8443/api/v1/auth/oauth/google/callback"

[oauth.internal.providers.github]
enabled = false
display_name = "GitHub"
client_id = ""
client_secret_env = "MAZE_OAUTH_GITHUB_SECRET"
redirect_uri = "https://your-host:8443/api/v1/auth/oauth/github/callback"

[oauth.internal.providers.facebook]
enabled = false
display_name = "Facebook"
client_id = ""
client_secret_env = "MAZE_OAUTH_FACEBOOK_SECRET"
redirect_uri = "https://your-host:8443/api/v1/auth/oauth/facebook/callback"

# ---- Outbound communications ----
# Disabled by default. Provider secrets are read from environment
# variables only — see the env-var column in the table above.
[comms]
enabled = false
public_base_url = "https://your-host:8443"

[comms.branding]
app_name = "Acme"                                # product name in {{ app_name }} (subjects/bodies); falls back to from_name / company_name
company_name = "Acme, Inc."
company_address = "123 Example St, City, Country"
company_url = "https://acme.example.com"         # defaults to comms.public_base_url
logo_url = "https://your-host:8443/static/logo.png"

[comms.email]
provider = "mailgun"                # "stub" (default), "mailgun", or "smtp_oauth2"
from = "noreply@example.com"
from_name = "The Maze Team"         # display name in From: header
templates_dir = "config/email_templates"

# Email audit-log behaviour. Applies to every provider — medium-level,
# not provider-specific. Default off so small / dev installs don't
# accumulate one audit-log entry per typo or probe; flip on for
# rate-limit / abuse forensics. Anti-enumeration timing and the 200
# response are unaffected either way.
[comms.email.audit]
record_unknown_password_reset_requests = false

# Provider sub-table consulted only when provider matches. The api_key is
# *never* read from this file — set MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY.
[comms.email.mailgun]
domain = "mg.example.com"
region = "us"                      # "us" or "eu"

# SMTP+XOAUTH2 sub-table — consulted only when provider = "smtp_oauth2".
# Pairs an SMTP relay with the OAuth vendor that mints bearer tokens,
# selected by `vendor`:
#   "microsoft"        → Microsoft 365 (Azure AD client-credentials, app-only token)
#   "google"           → Google Workspace (service-account JWT-bearer + DWD)
#   "google_personal"  → personal @gmail.com (per-user OAuth refresh-token)
# The Microsoft client_secret is *never* read from this file — set
# MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_SECRET. The
# Google private key lives in the JSON file at service_account_json_path.
# The Google-personal client_secret and refresh_token are env-only — set
# MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_PERSONAL_CLIENT_SECRET and
# MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_PERSONAL_REFRESH_TOKEN.
[comms.email.smtp_oauth2]
host = "smtp.office365.com"        # M365: smtp.office365.com  | Gmail: smtp.gmail.com
port = 587                         # 587 = STARTTLS, 465 = implicit TLS
tls = "starttls"                   # "starttls", "implicit", or "plain"
username = "noreply@contoso.com"   # SASL identity (typically the From mailbox)
vendor = "microsoft"               # | "google" | "google_personal"

[comms.email.smtp_oauth2.microsoft]
tenant_id = "00000000-0000-0000-0000-000000000000"
client_id = "00000000-0000-0000-0000-000000000000"
scopes    = ["https://outlook.office.com/.default"]

[comms.email.smtp_oauth2.google]
service_account_json_path = "/etc/maze/gcp-service-account.json"
delegated_subject         = "noreply@company.com"
scopes                    = ["https://www.googleapis.com/auth/gmail.send"]

[comms.email.smtp_oauth2.google_personal]
client_id = "1234567890-abc.apps.googleusercontent.com"
scopes    = ["https://mail.google.com/"]   # required for SMTP — gmail.send is API-only
```

Notes:

- Any environment variable values will take precedence over their corresponding configuration file values.
- `log_dir` is relative to the server working directory. Log files are named `{log_file_prefix}{YYYY-MM-DD}.log` and a new file is started each calendar day. Old log files are not deleted automatically.
- `log_file_prefix` is used verbatim — include any desired separator as the final character (e.g. `"maze_web_server_"` produces `maze_web_server_2026-04-09.log`, while `"my-app-"` produces `my-app-2026-04-09.log`).
- Valid `log_level` values are: `error`, `warn`, `info`, `debug`, `trace`.
- `allow_signup` controls whether new users can self-register. Set to `false` to disable public registration.
- `oauth.enabled` is the master switch — when `false`, no OAuth buttons render in any client and the per-provider sections below are not validated.
- `oauth.connector` selects the implementation. `internal` ships in v1; `auth0` is reserved for a future drop-in and will error with a clear "not yet implemented" message at startup.
- OAuth client secrets are **always** read from the environment variable named in `client_secret_env`, never from `config.toml`. On startup the server walks every enabled provider and reports *all* misconfigurations in one error (empty `client_id`, missing env var, etc.) rather than fix-restart-fix-restart looping. See the **OAuth Sign-In** subsection below for full setup steps.
- The `[storage]` section selects between the file-backed (`type = "file"`, the default) and SQL-backed (`type = "sql"`) implementations. The SQL backend supports SQLite, PostgreSQL, and MySQL via SQLx's `Any` driver — all three engines are compiled into the same binary; selection happens at runtime via `storage.sql.driver` and the connection details. See **Storage Backend** below for setup recipes per backend.
- The `[comms]` section configures outbound email — provider settings, templated-message branding, and template-source paths. `comms.enabled = false` (the default) skips the per-provider env-var checks at startup. Provider secrets — currently `comms.email.mailgun.api_key` — are **environment-only**: read from `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY` at startup and never from `config.toml`. Unlike `[oauth]`, missing comms secrets are **soft warnings**: the server still starts and logs a warning naming each unset env var so the operator can see the full set of misconfigurations in one log pass. Setting `comms.email.provider` to a value other than `"stub"` or `"mailgun"` is a hard deserialisation error at startup, not a runtime panic.
- `comms.email.audit.record_unknown_password_reset_requests` (default `false`) controls whether `/password-reset/request` writes an anti-enumeration "recon row" to the email audit log when the supplied email doesn't match a verified user. Off by default so small / dev installs don't accumulate one audit-log entry per typo or probe; flip on for rate-limit / abuse forensics. The 200 response and timing floor are unaffected either way.

## Storage Backend

The server stores users, maze definitions, OAuth identities, and login tokens in a pluggable backend selected by `storage.type`. A maze's stored JSON may also carry an optional `game_settings` sibling to its `definition` — the per-maze 3D environment settings (sky, wall/enemy/health styles, timer, …) authored in the clients; the server stores and returns it opaquely as part of the maze blob.

### When to use which

| Backend | Best for | Setup |
|:--------|:---------|:------|
| `file` | Local dev, single-instance, zero infrastructure | None — server creates `data/` on first run |
| `sql` + `sqlite` | Local dev, single-instance with relational guarantees, low-traffic self-hosted production | None — `auto_create_database = true` creates the `.db` file on first run |
| `sql` + `postgres` | Networked / multi-instance production, cloud deployments | Operator pre-provisions the database and grants the app user (see below) |
| `sql` + `mysql` | Networked / multi-instance production, MySQL-shop deployments | Same operator pattern as PostgreSQL |

### Example configurations

Runnable starter configs are checked in alongside this README:

| File | Description |
|:-----|:------------|
| [`config.example.sqlite.toml`](./config.example.sqlite.toml) | SQLite — no infrastructure, file at `maze.db` |
| [`config.example.postgres.toml`](./config.example.postgres.toml) | PostgreSQL on `localhost` (Docker / LAN), TLS off |
| [`config.example.postgres-cloud.toml`](./config.example.postgres-cloud.toml) | Cloud-managed PostgreSQL (RDS / Cloud SQL / Azure DB), TLS required, longer timeouts, no auto-create |
| [`config.example.mysql.toml`](./config.example.mysql.toml) | MySQL on `localhost` (Docker / LAN), TLS off |

Copy the relevant file over `config.toml` (or merge its `[storage]` block in) and adjust hostnames, usernames, etc. Set `MAZE_WEB_SERVER_STORAGE_SQL_PASSWORD` in the environment before starting the server when using `postgres` or `mysql`.

### Two-phase migration model (PostgreSQL / MySQL)

Production databases are managed in two phases with distinct privileges. The application **never** runs `CREATE DATABASE` or `CREATE USER` against a production server — those are operator-only steps.

**Phase 1 — Operator (one-time, before the app first connects):**

1. Create the database server instance (Docker / managed cloud service / on-prem install).
2. Create the application's database.
3. Create an application user with `CREATE TABLE` rights inside the database, but no server-level admin rights.

PostgreSQL:
```sql
CREATE DATABASE your_database;
CREATE USER your_app_user WITH PASSWORD '<your_app_password>';
GRANT CONNECT ON DATABASE your_database TO your_app_user;
\c your_database
GRANT USAGE, CREATE ON SCHEMA public TO your_app_user;
```

MySQL:
```sql
CREATE DATABASE your_database CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'your_app_user'@'%' IDENTIFIED BY '<your_app_password>';
GRANT CREATE, ALTER, DROP, INDEX, REFERENCES, SELECT, INSERT, UPDATE, DELETE
    ON your_database.* TO 'your_app_user'@'%';
FLUSH PRIVILEGES;
```

The app user gets `CREATE TABLE` rights inside the database but cannot create or drop other databases on the same server. Replace `your_database`, `your_app_user`, and `<your_app_password>` with your own values.

**Phase 2 — Application startup (every deployment):**

4. App connects with the app-user credentials to the pre-existing database.
5. SQLx applies any pending migrations from `storage/migrations/` automatically — this is when `CREATE TABLE` statements run.
6. SQLx tracks applied migrations in its own `_sqlx_migrations` table, so subsequent restarts skip migrations that have already been applied.

**Schema changes** (future migrations) ship as new `0002_*.sql` files alongside `0001_initial.sql` and apply automatically on the next deploy. The same binary runs against dev/staging/prod — only the connection config differs.

**`auto_create_database = true`** is for local dev / SQLite only. PostgreSQL and MySQL cloud credentials typically lack the server-level `CREATEDB`/`CREATE` privilege required for it to work, and managed databases are usually pre-provisioned by IaC (Terraform / CloudFormation / Bicep) anyway.

### TLS

`require_tls = true` enforces TLS for the connection. The URL gets driver-appropriate query parameters appended at startup:

| Driver | Without `ca_cert_path` | With `ca_cert_path` |
|:-------|:-----------------------|:--------------------|
| `postgres` | `?sslmode=require` (TLS used, cert not verified) | `?sslmode=verify-full&sslrootcert=<path>` (full verification) |
| `mysql` | `?ssl-mode=REQUIRED` | `?ssl-mode=VERIFY_CA&ssl-ca=<path>` |
| `sqlite` | (ignored — no network) | (ignored) |

For cloud-managed databases, `ca_cert_path` should point at the provider's CA bundle (e.g. `rds-global-bundle.pem` for AWS RDS).

#### PostgreSQL TLS — local Docker recipe

The default `postgres:16` image ships with `ssl = off`. To enable TLS for a local TLS smoke-test you need to run the container with TLS enabled and a cert mounted. From a host with OpenSSL available:

```bash
# Generate a self-signed cert/key pair (one-off)
openssl req -x509 -nodes -newkey rsa:2048 \
    -keyout postgres-server.key -out postgres-server.crt \
    -days 365 -subj "/CN=localhost"

# Run postgres with TLS enabled and the cert mounted
docker run --name maze-postgres-tls \
    -e POSTGRES_PASSWORD=pw -p 5432:5432 \
    -v "$(pwd)/postgres-server.crt:/var/lib/postgresql/server.crt:ro" \
    -v "$(pwd)/postgres-server.key:/var/lib/postgresql/server.key:ro" \
    -d postgres:16 \
    -c ssl=on \
    -c ssl_cert_file=/var/lib/postgresql/server.crt \
    -c ssl_key_file=/var/lib/postgresql/server.key
```

(On Windows, replace `$(pwd)` with the absolute Windows path to your cert files.)

Then set `require_tls = true` in `config.toml` and start the server. Verify TLS is actually being used by querying the live connection state:

```bash
docker exec -it maze-postgres-tls psql -U postgres -d your_database \
    -c "SELECT datname, usename, ssl, version FROM pg_stat_ssl JOIN pg_stat_activity USING(pid) WHERE usename = 'your_app_user';"
```

`ssl = t` and a `version` of `TLSv1.2` or `TLSv1.3` confirms the pool's connections are encrypted.

#### MySQL TLS — already on by default

The `mysql:8` Docker image enables TLS automatically with an auto-generated self-signed cert. Set `require_tls = true` in `config.toml`, start the server, and verify per-connection TLS state via:

```bash
docker exec -it maze-mysql mysql -uroot -ppw \
    -e "SELECT processlist_user, processlist_host, connection_type FROM performance_schema.threads WHERE processlist_user = 'your_app_user';"
```

`connection_type = SSL/TLS` per pool connection confirms TLS is in use.


## Web Frontend

A React Single Page Application (SPA) is available at [`src/react/maze_web_server/`](../../../src/react/maze_web_server/README.md). Build it and point `static_dir` at the output:

```bash
cd src/react/maze_web_server
npm install
npm run build
```

Then set `static_dir` in `config.toml`:

```toml
static_dir = "../../react/maze_web_server/dist"
```

The server will serve `index.html` for all non-API routes, enabling client-side routing. If `static_dir` does not exist or is not set, the server runs as API-only.

## Authentication

The server supports two authentication mechanisms:

| Mechanism | Header | Usage |
|:----------|:-------|:------|
| Static API key | `X-API-Key: <key>` | API access; key is a UUID stored per user in the data store |
| Bearer token | `Authorization: Bearer <token>` | Per-user login; token obtained via `POST /api/v1/login` |

The following endpoints manage user identity:

| Method | Path | Auth required | Description |
|:-------|:-----|:--------------|:------------|
| `POST` | `/api/v1/signup` | None | Register a new (non-admin) user account; requires email and password only — username is auto-generated from the email local part |
| `POST` | `/api/v1/login` | None | Sign in; returns a bearer token |
| `POST` | `/api/v1/logout` | Bearer | Invalidate the current bearer token |
| `GET` | `/api/v1/auth/oauth/{provider}/start` | None | Begin an OAuth sign-in flow; 302-redirects to the provider's consent page (see **OAuth Sign-In** below) |
| `GET` | `/api/v1/auth/oauth/{provider}/callback` | None | Provider redirects here after consent; mints a bearer token and redirects back to the SPA or mobile app |
| `GET` | `/api/v1/users/me` | Either | Return the signed-in user's profile (includes `email`, `emails`, and `has_password` — see below) |
| `GET` | `/api/v1/users/lookup?username=<prefix>` | Either | Look up users whose username **starts with** `<prefix>` (case-insensitive) for the share people-picker. Returns a page of `{ id, username }` — no email/admin/avatar. Paged via `limit` (default 20, capped at 100) / `offset` with a `has_more` flag; a blank prefix returns an empty page (never lists every user) |
| `PUT` | `/api/v1/users/me/profile` | Either | Update the signed-in user's username and full name. **Email is no longer mutable here** — use `/api/v1/users/me/emails` instead. Sending an `email` field returns `400 Bad Request` |
| `PUT` | `/api/v1/users/me/password` | Either | **Sets or changes** the signed-in user's password — the same endpoint handles both flows (see **Password set-or-change** below) |
| `DELETE` | `/api/v1/users/me` | Either | Delete the signed-in user's account and all their mazes |
| `GET` | `/api/v1/users/me/emails` | Either | List the signed-in user's email addresses with primary/verified status |
| `POST` | `/api/v1/users/me/emails` | Either | Add a new email row (created `verified = true` for now; once email-send-support ships, this becomes `verified = false` until the user clicks the verify link) |
| `DELETE` | `/api/v1/users/me/emails/{email}` | Either | Remove an email; rejects with 409 if the address is the user's only email or their primary |
| `PUT` | `/api/v1/users/me/emails/{email}/primary` | Either | Promote an email to primary; rejects with 409 if the target is unverified |
| `POST` | `/api/v1/users/me/emails/{email}/verify` | Either | **Stub** — returns `501 Not Implemented` until the email-verification flow ships |
| `POST` | `/api/v1/users/me/avatar` | Either | Upload/replace the caller's avatar (`multipart/form-data`, single `file` part: PNG or JPEG, ≤ 2 MiB). The server centre-crops + resizes to a 256×256 PNG; returns `{ "avatar_updated_at": <timestamp> }` |
| `DELETE` | `/api/v1/users/me/avatar` | Either | Remove the caller's avatar (idempotent — `204` even if none was set) |
| `GET` | `/api/v1/users/{id}/avatar` | Either | Serve a user's avatar as `image/png`, or `404` when none. Requires auth, but readable for **any** user id (not just the caller) so a signed-in viewer sees other players' avatars on boards/headers; cache-bust with `?v=<avatar_updated_at>` |

In addition, `GET /api/v1/features` returns an `oauth_providers` array describing the canonical name and human-readable display name of each provider currently enabled — clients render one button per entry.

The full API reference (including maze and admin-user endpoints) is available interactively via the documentation endpoints listed above.

## Leaderboards

Completed 3D runs are recorded per player and surfaced as leaderboards (per maze and per curated difficulty) plus a personal history. Only **won** runs are recorded; the server takes the player from the session and stamps the record time, so neither can be spoofed by the client.

| Method | Path | Auth required | Description |
|:-------|:-----|:--------------|:------------|
| `POST` | `/api/v1/scores` | Either | Record a completed (won) run. The body carries the run's `score` and `elapsed_ms` plus its subject — exactly one of a stored `maze_id` or a curated `challenge` (`"<difficulty>:<seed>"`). The server sets `user_id` from the session and stamps `recorded_at`; neither is trusted from the client. |
| `GET`  | `/api/v1/scores` | Either | A leaderboard page for one subject. Query: `maze_id` **or** `challenge` (exactly one); `metric` (`time` \| `score`, default `time`); `direction` (`asc` \| `desc`, default best-first for the metric); `limit` / `offset` (server-capped paging); `include_usernames` (default `true` — resolves each row's player username; pass `false` for personal boards). |
| `GET`  | `/api/v1/scores/me` | Either | The caller's own run history, most recent first; paged via `limit` / `offset`. |
| `POST` | `/api/v1/scores/me/completed` | Either | Given a set of challenge board keys (body `{ "challenges": ["def:<id>", …] }`, ≤ 200), returns `{ "completed": [...] }` — the subset the caller has scored on. Scoped to the caller's own scores; used to derive campaign progress in one request. |

- A run's **subject** is dual-keyed: a stored user maze (`maze_id`) or a curated game (`challenge = "<difficulty>:<seed>"`). Exactly one is set, so a board aggregates everyone who played that subject — the row's `user_id` is the player, not the maze owner.
- The two canonical orderings are **fastest time** (`elapsed_ms` ascending) and **highest score** (`score` descending); `direction` flips the primary metric, with the secondary metric and record time held as fixed tie-breaks for stable paging.
- `score` is the engine's running total at completion (currently the count of keys collected during the run); `elapsed_ms` is the run duration measured by the game, excluding paused time.

## Game definitions

A **game definition** is a stored, parametric 3D game: it holds no maze grid, only an opaque client-owned `config` blob plus a server-minted `seed` from which the client regenerates the whole game. Any user creates `Private` definitions; admins additionally publish `Curated` ones.

| Method | Path | Auth required | Description |
|:-------|:-----|:--------------|:------------|
| `POST`   | `/api/v1/game-definitions` | Either | Create a definition. The server mints the `seed` and sets id/owner/timestamps; the body carries `name`, `description`, `visibility`, `rotation`, `config`. Setting `visibility = "curated"` requires an admin. |
| `GET`    | `/api/v1/game-definitions` | Either | List game definitions, ordered by name. `scope=visible` (default) is the caller's visible set — their own (all visibilities) + shared-with-them + public + curated, de-duplicated; `scope=mine` is only their own (any visibility), optionally filtered by a case-insensitive name substring `q`; `scope=shared` is only the definitions **shared with** them (a share grant they don't own; public/curated excluded). `excludeDefinitions=true` blanks each game's opaque `config` blob (returning only the light metadata — id/name/visibility/…), for callers that only need to list games (e.g. the collection membership picker). Paged via `limit` (default 20, capped at 100) / `offset`; the response echoes the effective `limit`/`offset` and a `hasMore` flag. |
| `GET`    | `/api/v1/game-definitions/{id}` | Either | **Play-fetch** for one accessible definition (owner ∨ curated ∨ public ∨ granted; otherwise `404`). Returns the definition with the *effective* seed spliced into `config`, plus the computed `challengeKey` and `leaderboardTracked`. |
| `PUT`    | `/api/v1/game-definitions/{id}` | Either | Update a definition the caller owns — or, for an **admin**, a **Featured** (curated) definition they don't own, or one they are featuring (setting curated); ownership preserved, not transferred. (An admin cannot edit an unrelated non-featured definition they don't own.) `seed` and image are server-owned and preserved. Setting `visibility = "curated"` requires an admin; a curated↔non-curated change appends to / removes from the featured catalogue. |
| `POST`   | `/api/v1/game-definitions/{id}/reshuffle` | Either | Re-mint the definition's `seed` to change its generated layout (the seed is otherwise preserved across updates, so this is its own endpoint). If the definition is published its leaderboard is reset (the layout — and thus fair comparison — has changed); a private draft has no board to clear. Owner-only. Returns the definition with its new seed. |
| `DELETE` | `/api/v1/game-definitions/{id}` | Either | Delete a definition the caller owns, removing its shares and resetting its leaderboard(s). |
| `GET`    | `/api/v1/game-definitions/{id}/shares` | Either | List the grantees of a definition the caller owns (manage-shares view) — each resolved to `{ id, username, avatar_updated_at? }` (the marker present only when the grantee has an avatar), ordered by username. |
| `PUT`    | `/api/v1/game-definitions/{id}/shares` | Either | **Set** the definition's share list to the supplied set (body `{ "userIds": [ … ] }`) — anyone not listed is revoked, any new id granted, in one operation. Owner-only; the owner's own id is ignored. Returns the updated grantee list. |
| `POST`   | `/api/v1/game-definitions/{id}/image` | Either | Upload/replace the game's image (`multipart/form-data`, single `file` part: PNG or JPEG, ≤ 2 MiB; centre-cropped + resized to a 256×256 PNG). Owner-only. Returns `{ imageUpdatedAt }`. |
| `DELETE` | `/api/v1/game-definitions/{id}/image` | Either | Remove the game's image (idempotent). Owner-only. |
| `GET`    | `/api/v1/game-definitions/{id}/image` | Either | Serve the game's image as `image/png`, or `404`. **Access-checked** like the play-fetch (owner ∨ curated ∨ public ∨ granted); cache-bust with `?v=<imageUpdatedAt>`. |

- **Leaderboard subject** is per-definition: a `static` game uses its fixed seed and the key `def:<id>`; a `daily` game folds today's UTC date into both the seed and the key (`def:<id>:<yyyy-mm-dd>`), so each day gets a fresh, comparable board. Every game is `leaderboardTracked` — a private game's board is simply **owner-only** (`GET /scores?challenge=def:<id>` and recording a score are both access-checked, owner ∨ curated ∨ public ∨ granted). The board resets only on a gameplay change (structure/scene/content/mechanics or rotation) or a reshuffle/delete; publishing keeps it.

## Game collections

A **game collection** is an ordered, presentation-only grouping of game definitions — it does not affect generation or scoring (leaderboards stay per-definition). Membership is order-only: an item is just a `definitionId` + position, and each game's name/description/image is intrinsic to its definition and shared across every collection it appears in.

On first launch the server seeds a curated **"Difficulty"** collection — the `Easy` / `Tricky` / `Hard` games (values lifted from the shipped `game.play3d.*` presets) — owned by the default admin. The seeding is idempotent, so it runs safely on every launch.

| Method | Path | Auth required | Description |
|:-------|:-----|:--------------|:------------|
| `POST`   | `/api/v1/game-collections` | Either | Create a collection (metadata only — `name`, `description`, `visibility`, `playMode`; starts empty). `playMode` is `arcade` (free-choice, default) or `campaign` (ordered progression). Setting `visibility = "curated"` requires an admin. |
| `GET`    | `/api/v1/game-collections` | Either | List game collections, ordered by name. `scope=visible` (default) is the caller's visible set — own + shared-with-me + public + curated, de-duplicated; `scope=mine` is only their own (any visibility), optionally filtered by a case-insensitive name substring `q`; `scope=shared` is only the collections **shared with** them (a share grant they don't own; public/curated excluded). Paged via `limit` (default 20, capped at 100) / `offset`; echoes the effective `limit`/`offset` + a `hasMore` flag. |
| `GET`    | `/api/v1/game-collections/{id}` | Either | Fetch one accessible collection (owner ∨ curated ∨ public ∨ granted; otherwise `404`) with its member definitions **hydrated, in order, and filtered to what the viewer may access** — a public collection never exposes a private member, and refs to since-deleted definitions are dropped. |
| `PUT`    | `/api/v1/game-collections/{id}` | Either | Update a collection's metadata (`name`, `description`, `visibility`, `playMode`) the caller owns — or, for an **admin**, a **Featured** (curated) collection they don't own, or one they are featuring (setting curated); ownership preserved. (An admin cannot edit an unrelated non-featured collection they don't own.) Membership + image unchanged. Setting `visibility = "curated"` requires an admin; a curated↔non-curated change appends to / removes from the featured catalogue. |
| `DELETE` | `/api/v1/game-collections/{id}` | Either | Delete a collection the caller owns (its member definitions are untouched). |
| `PUT`    | `/api/v1/game-collections/{id}/items` | Either | **Set** the collection's whole membership to the supplied ordered list (body `{ "definitionIds": [definitionId, …] }`) — reconciles in one operation (drop absent, add new, reorder; duplicates collapse). The owner may edit it, or an **admin** may edit a **Featured** (curated) collection they don't own (admin-override, ownership preserved); any other non-owner gets `404`. Returns the updated collection. |
| `GET`    | `/api/v1/game-collections/{id}/shares` | Either | List the grantees of a collection the caller owns — each resolved to `{ id, username, avatar_updated_at? }` (the marker present only when the grantee has an avatar), ordered by username. |
| `PUT`    | `/api/v1/game-collections/{id}/shares` | Either | **Set** the collection's share list to the supplied set (body `{ "userIds": [ … ] }`) — reconciles in one operation. Owner-only; the owner's own id is ignored. Returns the updated grantee list. |
| `POST`   | `/api/v1/game-collections/{id}/image` | Either | Upload/replace the collection's image (`multipart/form-data`, single `file` part: PNG or JPEG, ≤ 2 MiB → 256×256 PNG). Owner-only. Returns `{ imageUpdatedAt }`. |
| `DELETE` | `/api/v1/game-collections/{id}/image` | Either | Remove the collection's image (idempotent). Owner-only. |
| `GET`    | `/api/v1/game-collections/{id}/image` | Either | Serve the collection's image as `image/png`, or `404`. Access-checked (owner ∨ curated ∨ public ∨ granted); cache-bust with `?v=<imageUpdatedAt>`. |

- A collection's `visibility` gates the **grouping**; each member still enforces its own access, so the detail endpoint filters the member list per viewer. Membership stores only references — a ref to an inaccessible or since-deleted definition is simply skipped at detail time (dangling refs are tolerated).

## Featured catalogue

The **featured catalogue** is the admin-ordered list that drives the Play-3D *Featured* section — one ordered sequence mixing curated game definitions and collections. It is a faithful projection of the `curated` visibility tier maintained by the storage layer: an entity becoming `curated` (via the definition / collection `PUT`) is appended, and un-curating or deleting it removes the row and recompacts the order. **Featuring is therefore not a separate action** — an admin features an item by setting its access tier to `curated` on the item itself; these endpoints only *read* and *reorder* the catalogue. The path is deliberately distinct from the app-flags `GET /api/v1/features`.

| Method | Path | Auth required | Description |
|:-------|:-----|:--------------|:------------|
| `GET`  | `/api/v1/featured-game-items` | Either | A page of the ordered catalogue — the curated definitions + collections, hydrated and in sort order. Any signed-in user. Each item is `{ kind: "definition" \| "collection", ownerUsername, definition? , collection? }` (the field matching `kind` is present; `ownerUsername` is the item owner's username, resolved server-side for the admin view, or `"unknown"` if the owner can't be resolved). Paged via `limit` (default 20, capped at 100) / `offset`; echoes the effective `limit`/`offset` + a `hasMore` flag. |
| `PUT`  | `/api/v1/featured-game-items/order` | Either | **Admin-only.** Rewrite the catalogue order in one operation to match the body `{ "entries": [ { "kind", "id" }, … ] }` (order-only — membership stays owned by the `curated` tier). An entry whose entity is not `curated`, or is unknown, is rejected with `400`. Returns the full catalogue in its new order. |

## Game

The 3D maze game (Bevy / WASM, served from `/game/`) fetches its session config at startup from the server, so a single edit to `config.toml` propagates to every client without a rebuild:

| Method | Path | Auth required | Description |
|:-------|:-----|:--------------|:------------|
| `GET`  | `/api/v1/game/play3d-config?difficulty=easy\|tricky\|hard` | None | Returns the configured Play 3D preset for the difficulty: maze dimensions, time limit, fixed RNG seed, minimum solution-path length, and in-game splash title. Difficulty value is case-insensitive; unknown values return `400`. |

Response shape (camelCase):

```json
{
  "difficulty": "easy",
  "rows": 8,
  "cols": 8,
  "timerSeconds": 120,
  "seed": 8080808,
  "minSolutionLength": 30,
  "minimapCellPx": 10,
  "minimapRadius": 5,
  "title": "Maze 3D",
  "mode": "Easy",
  "landmarks": {
    "wallTint": true,
    "deadEndObjects": true,
    "wallDecorations": true,
    "floorAccents": true,
    "wallMaterialVariation": true
  },
  "skyType": "night",
  "wallType": "brick",
  "doorStyle": "swing",
  "keyHolder": "pedestal",
  "doorCount": 2,
  "spareDoors": 0,
  "spareKeys": 0
}
```

- `seed` is **fixed per difficulty** (not minted per request) so leaderboard records on the same difficulty share the same maze layout from day 1. Override per-session via `/game/?difficulty=easy&seed=<n>` if variety is wanted.
- `minSolutionLength` is plumbed through to the maze crate's `min_spine_length` generator option (with the crate's default `max_retries`). Set it too high and generation will error rather than produce a degenerate maze.
- `minimapCellPx` / `minimapRadius` size the in-game minimap: `minimapCellPx` scales its on-screen footprint, `minimapRadius` controls how many cells around the player are visible (a `2r+1` square window). Both default to the shipped values (10 / 5).
- `title` is the in-game splash text shown for ~2 s on game start. Override per difficulty via `[game.play3d.<difficulty>].title`.
- `doorStyle` (`swing` / `slide` / `portcullis` / `dissolve`) and `keyHolder` (`pedestal` / `chest` / `floating_key`) choose the 3D look of doors and key holders. Unknown values fall back to `swing` / `pedestal`.
- `doorCount` is the number of real path doors (each paired with one key) the generator auto-places on the maze's spine, clamped to 8 and to what the maze can hold (`0` = lock-free).
- `spareDoors` / `spareKeys` scatter **decoys** + a **safety budget** onto off-spine branches after the solvability check. Decoys are visually indistinguishable from real path doors — opening one burns a key the player may have needed for a real door, potentially stranding them; spare keys give the player room to absorb that mistake. The shipped tricky and hard presets ramp the strand risk (`tricky: 2/1`, `hard: 3/1`); easy mode ships with both at `0` for no strand risk.

### `GET /api/v1/users/me` shape

The response is a `UserItem` carrying:

- `id`, `is_admin`, `username`, `full_name`
- `email` — the **primary** email address (legacy single-field shape, preserved for backwards-compat)
- `emails` — the full list of email rows: `{ email, is_primary, verified, verified_at }` per row. Always at least one row; exactly one is `is_primary`
- `has_password` — `true` if a password is set, `false` for OAuth-only users who haven't yet added a password. Front-ends use this to choose between the "Change Password" and "Set Password" UI variants
- `avatar_updated_at` — present (an RFC 3339 timestamp) when the user has an avatar, absent otherwise. Doubles as the "has an avatar" signal and the cache-buster: clients render `/api/v1/users/{id}/avatar?v=<avatar_updated_at>` when present, and the generic placeholder when absent

### Password set-or-change

`PUT /api/v1/users/me/password` is a single endpoint that handles both setting an initial password (OAuth-only users adding a password as a second login method) and changing an existing one. The body shape is:

```json
{ "current_password": "...", "new_password": "..." }
```

`current_password` is **optional**, with branching driven by the user's existing state (which the client reads from `has_password` on `GET /me`):

| User state                  | Required body                                | Behaviour                                     |
|:----------------------------|:---------------------------------------------|:----------------------------------------------|
| `has_password = true`       | `current_password` + `new_password`          | Verify `current_password`, then rotate        |
| `has_password = false`      | `new_password` only (omit `current_password`)| Set initial password                          |

Mismatched shapes return `400 Bad Request`:
- Sending `current_password` to a user who doesn't have one yet (the "set" path)
- Omitting `current_password` for a user who does (the "change" path)

A wrong `current_password` on the change path returns `401 Unauthorized`.

### Password Requirements

The following password complexity rules apply when creating an account (`POST /api/v1/signup`) or setting/changing a password (`PUT /api/v1/users/me/password`):

| Rule | Requirement |
|:-----|:------------|
| Minimum length | 8 characters |
| Uppercase letter | At least one (`A`–`Z`) |
| Lowercase letter | At least one (`a`–`z`) |
| Digit | At least one (`0`–`9`) |
| Special character | At least one non-alphanumeric character (e.g. `!`, `@`, `#`) |

A password such as `Password1!` satisfies all rules.

> **Note:** These rules are enforced server-side. The MAUI client also validates them locally before submitting the request.

### OAuth Sign-In

The server supports a server-mediated OAuth / OIDC sign-in flow behind a pluggable `OAuthConnector` trait. v1 ships with the `InternalOAuthConnector` (built on the [`oauth2`](https://crates.io/crates/oauth2) and [`openidconnect`](https://crates.io/crates/openidconnect) crates) which speaks OAuth/OIDC directly to each configured provider. A future `Auth0Connector` (or other broker) can be added as a drop-in implementation of the same trait without touching the handler layer or storage.

#### Provider setup

For each provider you want to enable, register an OAuth client with the provider:

| Provider | Where | Notes |
|:---------|:------|:------|
| **Google** | [Google Cloud Console](https://console.cloud.google.com/apis/credentials) → Create OAuth client ID → Web application | Add the value of `redirect_uri` to **Authorized redirect URIs**. Copy Client ID into `client_id`; the generated client secret goes in the env var named by `client_secret_env`. |
| **GitHub** | [GitHub Developer Settings](https://github.com/settings/developers) → New OAuth App | Set **Authorization callback URL** to match `redirect_uri` exactly. Copy Client ID into `client_id`; the generated client secret goes in the env var. The signed-in GitHub account must have a verified primary email at <https://github.com/settings/emails> for sign-in to succeed. |
| **Facebook** | [Facebook for Developers](https://developers.facebook.com) → My Apps → Create App → "Authenticate and request data from users" → add the **Facebook Login** product | Under Facebook Login → Settings, add the value of `redirect_uri` to **Valid OAuth Redirect URIs** (HTTPS only outside `localhost`). Copy App ID into `client_id`; the App Secret goes in the env var. The Facebook account must have an email on file (declining the `email` scope at consent results in `email_not_verified`). The app must be in **Live** mode (with privacy policy + terms URLs) for non-developer users to sign in — in dev mode only roles you explicitly add (Admin / Developer / Tester) can authenticate. Facebook does not expose an `email_verified` flag, so we treat the email as verified whenever it is present (matches Auth0/Clerk default). |

Then in `config.toml` (for Google):

```toml
[oauth]
enabled = true
connector = "internal"
mobile_redirect_scheme = "maze-app"

[oauth.internal.providers.google]
enabled = true
display_name = "Google"
client_id = "<Google client id>"
client_secret_env = "MAZE_OAUTH_GOOGLE_SECRET"
redirect_uri = "https://your-host:8443/api/v1/auth/oauth/google/callback"
```

…and set the corresponding environment variable before starting the server:

```powershell
$env:MAZE_OAUTH_GOOGLE_SECRET = "<Google client secret>"
```

```bash
export MAZE_OAUTH_GOOGLE_SECRET="<Google client secret>"
```

The server's `redirect_uri` and the provider's registered redirect URI must match **exactly** (scheme, host, port, path).

#### Account resolution

When a callback arrives the server applies these rules in order:

1. **Returning OAuth user** — if `(provider, provider_user_id)` is already linked to a user, sign that user in. The user's `provider_email` and `last_seen_at` are refreshed from the latest provider response.
2. **First-time-OAuth, email matches an existing user** — if the provider asserts a verified email that matches an existing password (or other OAuth) account, append a new OAuth identity to that user and sign them in. **Not** gated by `allow_signup` — attaching a sign-in method to an existing account is not the same as creating one.
3. **First-time-OAuth, no matching account** — create a new user with an empty `password_hash` and the OAuth identity attached. **Only this branch is gated by `allow_signup`.** Username is auto-generated from the email local part with a `_2`, `_3`, … suffix on collision.

OAuth-only users (those created via branch 3) cannot sign in via `POST /api/v1/login` — `verify_password` is hardened to reject empty / non-Argon2 hashes, returning `401 Invalid email or password`. They sign in only via the OAuth flow.

> **Mismatched-email edge case (deliberately unsolved in v1):** if a user's existing password account uses a different email than their OAuth provider account, branch (2) cannot see the connection and falls through to branch (3), creating a duplicate user. A future "Linked accounts" UI in My Account will provide an explicit-link path that side-steps this.

### Default Admin Account

On first run, if no admin user exists in the data store, the server automatically creates one with the following credentials:

| Field | Value |
|:------|:------|
| Username | `admin` |
| Email | `admin@maze.local` |
| Password | `Admin123!` |

Sign in using the **email address** and password. The username is used for display purposes only.

> **Important:** The default password is intentionally simple. **Change it immediately after first sign-in** using the self-service endpoint (`PUT /api/v1/users/me/password`) or the admin user-management API (`PUT /api/v1/users/{id}`).
