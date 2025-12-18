# MusicBrainz Tools

Outils de recherche dans la base de données MusicBrainz pour trouver des artistes, albums et morceaux.

## Architecture

Le module est organisé en fichiers par domaine :

```
mb/
├── mod.rs           # Exports et re-exports
├── common.rs        # Utilitaires partagés (validation MBID, formatage)
├── artist.rs        # Recherche d'artistes et leurs releases
├── release.rs       # Recherche de releases, tracks, versions
├── recording.rs     # Recherche de recordings et leurs apparitions
├── advanced.rs      # Recherche avancée multi-entités
├── search.rs        # (Legacy) Tool monolithique, à supprimer
└── README.md        # Cette documentation
```

## Outils disponibles

### 1. Artist Tool (`MbArtistTool`)

Recherche d'artistes et leurs releases.

**Types de recherche:**
- `artist` - Rechercher des artistes par nom
- `artist_releases` - Trouver les releases d'un artiste (par nom ou MBID)

**Exemple:**
```json
{
  "search_type": "artist",
  "query": "Nirvana",
  "limit": 5
}
```

### 2. Release Tool (`MbReleaseTool`)

Recherche de releases (albums), tracks et versions.

**Types de recherche:**
- `release` - Rechercher des releases par titre
- `release_recordings` - Obtenir la tracklist d'une release
- `release_group_releases` - Trouver toutes les versions d'un release group

**Exemple:**
```json
{
  "search_type": "release_recordings",
  "query": "OK Computer",
  "limit": 20
}
```

### 3. Recording Tool (`MbRecordingTool`)

Recherche de recordings (morceaux/tracks).

**Types de recherche:**
- `recording` - Rechercher des recordings par titre
- `recording_releases` - Trouver sur quelles releases apparaît un recording

**Exemple:**
```json
{
  "search_type": "recording_releases",
  "query": "Paranoid Android",
  "limit": 10
}
```

### 4. Advanced Search Tool (`MbAdvancedSearchTool`)

Recherche avancée sur différents types d'entités.

**Types d'entités:**
- `artist` - Artistes
- `release` - Releases/albums
- `release_group` - Groupes de releases
- `recording` - Recordings/tracks
- `work` - Œuvres musicales
- `label` - Labels discographiques

**Exemple:**
```json
{
  "entity": "label",
  "query": "Sony",
  "limit": 5
}
```

## Module commun (`common.rs`)

Utilitaires partagés :

- `is_mbid(query)` - Vérifie si une chaîne est un UUID MusicBrainz
- `format_duration(ms)` - Formate une durée en MM:SS
- `extract_year(date)` - Extrait l'année d'une DateString
- `format_date(date)` - Formate une date pour affichage
- `get_artist_name(credit)` - Extrait le nom d'artiste d'un artist_credit
- `error_result(msg)` / `success_result(content)` - Helpers pour CallToolResult
- `default_limit()` / `validate_limit(n)` - Gestion des limites de résultats

## Tests

### Tests unitaires

```bash
# Tous les tests du module mb
cargo test --lib mb::

# Tests d'un module spécifique
cargo test --lib mb::artist
cargo test --lib mb::common
```

### Tests d'intégration (requêtes API réelles)

⚠️ **Important**: Ces tests font de vraies requêtes à l'API MusicBrainz. 
- Ils sont marqués `#[ignore]` par défaut
- Respectez le rate limit de MusicBrainz (1 req/sec)
- **Utilisez toujours `--test-threads=1`** pour éviter les erreurs de rate limiting

```bash
# Lancer tous les tests d'intégration
cargo test --lib mb:: -- --ignored --test-threads=1

# Avec logs détaillés
cargo test --lib mb:: -- --ignored --test-threads=1 --nocapture
```

## Handlers HTTP vs STDIO

Chaque tool expose deux handlers :

- `handle_http()` - Utilise `std::thread::spawn` pour éviter "Cannot start a runtime from within a runtime"
- `handle_stdio()` - Utilise `tokio::task::spawn_blocking` pour la compatibilité STDIO/TCP

## API MusicBrainz

- Documentation: https://musicbrainz.org/doc/MusicBrainz_API
- Rate limit: 1 requête/seconde (moyenne)
- Bibliothèque: `musicbrainz_rs` v0.12 (mode blocking)

## Limitations

- Requiert une connexion internet active
- Soumis aux limites de l'API MusicBrainz
- Mode blocking uniquement (synchrone via thread/spawn_blocking)
- Pas de cache (chaque recherche fait une vraie requête)
