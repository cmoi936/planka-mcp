# Logging Guide / Guide de Journalisation

## English

### Overview

planka-mcp uses structured logging with configurable log levels to provide detailed information about API operations and errors. All logs are written to stderr to keep stdout clean for JSON-RPC communication.

### Log Levels

Configure the log level using the `RUST_LOG` environment variable:

| Level | Description | Use Case |
|-------|-------------|----------|
| `error` | Only errors | Production - minimal logging |
| `warn` | Warnings and errors | Production - standard logging |
| `info` | Info, warnings, and errors | Default - normal operations |
| `debug` | Debug messages + all above | Development - troubleshooting |
| `trace` | All messages including requests/responses | Development - detailed debugging |

### Configuration Examples

#### Basic Configuration

```bash
# Default (info level)
./planka-mcp

# Error level only
RUST_LOG=error ./planka-mcp

# Debug level
RUST_LOG=debug ./planka-mcp

# Trace level (most verbose)
RUST_LOG=trace ./planka-mcp
```

#### Module-Specific Configuration

You can set different log levels for different modules:

```bash
# Debug for planka_mcp, info for everything else
RUST_LOG=planka_mcp=debug ./planka-mcp

# Trace for API client only
RUST_LOG=planka_mcp::planka=trace ./planka-mcp

# Debug for tools, trace for API, info for MCP server
RUST_LOG="planka_mcp::tools=debug,planka_mcp::planka=trace,planka_mcp::mcp=info" ./planka-mcp
```

#### Docker Configuration

```bash
docker run -it --rm \
  -e RUST_LOG=debug \
  -e PLANKA_URL="https://kanban.local" \
  -e PLANKA_TOKEN="your-token" \
  ghcr.io/cmoi936/planka-mcp:latest
```

### What Gets Logged

#### Error Level
- API request failures with status codes and response bodies
- Authentication errors
- JSON parsing errors
- Configuration errors

#### Warn Level
- Unknown notifications or methods
- Tool execution failures
- Destructive operations (delete card, delete list)

#### Info Level
- Server startup and shutdown
- Client initialization
- Tool calls and their results
- API operation success (with counts/IDs)

#### Debug Level
- JSON-RPC message handling
- Request/response flow
- API request preparation
- Tool dispatching

#### Trace Level
- Raw JSON-RPC messages
- Complete request/response bodies
- Detailed API request/response data
- Full tool arguments and results

### Log Format

Logs include structured fields for easy parsing:

```
2024-01-20T20:00:00.123Z INFO planka_mcp::planka: Creating new card card_type=project list_id=123 name="New Task"
2024-01-20T20:00:00.456Z ERROR planka_mcp::planka: Card creation failed status=404 path=/api/lists/123/cards response_body="List not found"
```

### Troubleshooting with Logs

#### API Connection Issues
```bash
RUST_LOG=planka_mcp::planka=trace ./planka-mcp
```
Shows: URL construction, authentication, request/response details

#### Tool Execution Problems
```bash
RUST_LOG=planka_mcp::tools=debug ./planka-mcp
```
Shows: Tool arguments, dispatching, results

#### MCP Protocol Issues
```bash
RUST_LOG=planka_mcp::mcp=trace ./planka-mcp
```
Shows: Raw JSON-RPC messages, parsing, handling

#### General Debugging
```bash
RUST_LOG=trace ./planka-mcp 2>debug.log
```
Captures everything to debug.log file

---

## Français

### Vue d'ensemble

planka-mcp utilise une journalisation structurée avec des niveaux de log configurables pour fournir des informations détaillées sur les opérations API et les erreurs. Tous les logs sont écrits sur stderr pour garder stdout propre pour la communication JSON-RPC.

### Niveaux de Log

Configurez le niveau de log avec la variable d'environnement `RUST_LOG` :

| Niveau | Description | Cas d'usage |
|--------|-------------|-------------|
| `error` | Erreurs uniquement | Production - journalisation minimale |
| `warn` | Avertissements et erreurs | Production - journalisation standard |
| `info` | Info, avertissements et erreurs | Par défaut - opérations normales |
| `debug` | Messages de débogage + tout ce qui précède | Développement - dépannage |
| `trace` | Tous les messages y compris requêtes/réponses | Développement - débogage détaillé |

### Exemples de Configuration

#### Configuration de Base

```bash
# Par défaut (niveau info)
./planka-mcp

# Niveau erreur uniquement
RUST_LOG=error ./planka-mcp

# Niveau debug
RUST_LOG=debug ./planka-mcp

# Niveau trace (le plus verbeux)
RUST_LOG=trace ./planka-mcp
```

#### Configuration par Module

Vous pouvez définir différents niveaux de log pour différents modules :

```bash
# Debug pour planka_mcp, info pour le reste
RUST_LOG=planka_mcp=debug ./planka-mcp

# Trace pour le client API uniquement
RUST_LOG=planka_mcp::planka=trace ./planka-mcp

# Debug pour les outils, trace pour l'API, info pour le serveur MCP
RUST_LOG="planka_mcp::tools=debug,planka_mcp::planka=trace,planka_mcp::mcp=info" ./planka-mcp
```

#### Configuration Docker

```bash
docker run -it --rm \
  -e RUST_LOG=debug \
  -e PLANKA_URL="https://kanban.local" \
  -e PLANKA_TOKEN="your-token" \
  ghcr.io/cmoi936/planka-mcp:latest
```

### Ce qui est Journalisé

#### Niveau Error
- Échecs de requêtes API avec codes de statut et corps de réponse
- Erreurs d'authentification
- Erreurs d'analyse JSON
- Erreurs de configuration

#### Niveau Warn
- Notifications ou méthodes inconnues
- Échecs d'exécution des outils
- Opérations destructives (suppression de carte, suppression de liste)

#### Niveau Info
- Démarrage et arrêt du serveur
- Initialisation du client
- Appels d'outils et leurs résultats
- Succès des opérations API (avec compteurs/IDs)

#### Niveau Debug
- Gestion des messages JSON-RPC
- Flux requête/réponse
- Préparation des requêtes API
- Dispatching des outils

#### Niveau Trace
- Messages JSON-RPC bruts
- Corps complets des requêtes/réponses
- Données détaillées des requêtes/réponses API
- Arguments et résultats complets des outils

### Format des Logs

Les logs incluent des champs structurés pour un parsing facile :

```
2024-01-20T20:00:00.123Z INFO planka_mcp::planka: Creating new card card_type=project list_id=123 name="New Task"
2024-01-20T20:00:00.456Z ERROR planka_mcp::planka: Card creation failed status=404 path=/api/lists/123/cards response_body="List not found"
```

### Dépannage avec les Logs

#### Problèmes de Connexion API
```bash
RUST_LOG=planka_mcp::planka=trace ./planka-mcp
```
Affiche : Construction d'URL, authentification, détails requête/réponse

#### Problèmes d'Exécution d'Outils
```bash
RUST_LOG=planka_mcp::tools=debug ./planka-mcp
```
Affiche : Arguments des outils, dispatching, résultats

#### Problèmes de Protocole MCP
```bash
RUST_LOG=planka_mcp::mcp=trace ./planka-mcp
```
Affiche : Messages JSON-RPC bruts, parsing, traitement

#### Débogage Général
```bash
RUST_LOG=trace ./planka-mcp 2>debug.log
```
Capture tout dans le fichier debug.log
