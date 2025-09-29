# PermaMuseum Backend - Contratos Soroban

Este proyecto contiene los contratos inteligentes de Soroban para el ecosistema PermaMuseum, una plataforma de museos digitales y patrimonio cultural.

## Estado del Proyecto
- ✅ Contratos implementados y compilando correctamente
- ✅ Estructura del proyecto completa
- 🔄 Preparando para deploy

## Contratos

### 🏛️ MuseumRegistry
Registro de museos verificados con validación de autenticidad y metadatos.

### 🎨 CulturalNFT  
Token standard para patrimonio cultural con metadatos enriquecidos y trazabilidad.

### 🛒 Marketplace
Gestión de ventas, subastas y distribución automática de royalties.

### 🏆 SocialFi
Sistema de puntos, insignias y recompensas para la comunidad.

## Estructura del Proyecto

```
├── contracts/
│   ├── museum-registry/     # Registro de museos
│   ├── cultural-nft/        # Token de patrimonio cultural  
│   ├── marketplace/         # Mercado de NFTs
│   └── socialfi/           # Sistema de recompensas
├── scripts/                # Scripts de deployment
└── tests/                  # Tests de integración
```

## Desarrollo

Para desarrollar y desplegar los contratos:

```bash
# Instalar dependencias
cargo install soroban-cli

# Compilar todos los contratos
cargo build --workspace

# Ejecutar tests
cargo test --workspace
```

## Tecnologías

- **Soroban**: Plataforma de contratos inteligentes de Stellar
- **Rust**: Lenguaje de programación
- **Stellar Network**: Red blockchain
