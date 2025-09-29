#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec, Symbol};

/// Contrato para tokens de patrimonio cultural
/// 
/// Este contrato implementa un standard de NFT para patrimonio cultural con:
/// - Metadatos enriquecidos para patrimonio
/// - Trazabilidad de procedencia
/// - Atributos culturales específicos
/// - Integración con museos verificados
#[contract]
pub struct CulturalNFT;

// Claves de storage
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const NAME_KEY: Symbol = symbol_short!("NAME");
const SYMBOL_KEY: Symbol = symbol_short!("SYMBOL");
const TOKEN_COUNT_KEY: Symbol = symbol_short!("TOKEN_CNT");
const OWNER_KEY: Symbol = symbol_short!("OWNER");
const METADATA_KEY: Symbol = symbol_short!("METADATA");
const PROVENANCE_KEY: Symbol = symbol_short!("PROV");
const MUSEUM_REGISTRY_KEY: Symbol = symbol_short!("MUS_REG");

// Estructura para metadatos culturales
#[derive(Clone)]
#[contracttype]
pub struct CulturalMetadata {
    pub title: String,
    pub artist: String,
    pub period: String,
    pub culture: String,
    pub material: String,
    pub dimensions: String,
    pub condition: String,
    pub significance: String,
    pub museum_address: Address,
}

// Estructura para información de procedencia
#[derive(Clone)]
#[contracttype]
pub struct PROVENANCERecord {
    pub date: u64,
    pub from: Address,
    pub to: Address,
    pub transaction_type: String,
    pub notes: String,
}

#[contractimpl]
impl CulturalNFT {
    /// Inicializa el contrato
    pub fn initialize(
        env: Env, 
        admin: Address, 
        name: String, 
        symbol: String,
        museum_registry: Address
    ) {
        // Verificar que no esté ya inicializado
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("Contract already initialized");
        }
        
        // Guardar configuración inicial
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&NAME_KEY, &name);
        env.storage().instance().set(&SYMBOL_KEY, &symbol);
        env.storage().instance().set(&MUSEUM_REGISTRY_KEY, &museum_registry);
        
        // Inicializar contador de tokens
        env.storage().instance().set(&TOKEN_COUNT_KEY, &0u32);
    }

    /// Crea un nuevo NFT de patrimonio cultural
    pub fn mint_cultural_nft(
        env: Env,
        to: Address,
        token_id: u32,
        cultural_metadata: CulturalMetadata,
        provenance: Vec<PROVENANCERecord>,
    ) {
        // Verificar que el caller es admin
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Verificar que el token no existe
        let owner_key = (OWNER_KEY, token_id);
        if env.storage().persistent().has(&owner_key) {
            panic!("Token already exists");
        }
        
        // Verificar que el museo está verificado
        let _museum_registry: Address = env.storage().instance().get(&MUSEUM_REGISTRY_KEY).unwrap();
        // TODO: Llamar al contrato de registry para verificar museo
        
        // Asignar propietario
        env.storage().persistent().set(&owner_key, &to);
        
        // Guardar metadatos culturales
        let metadata_key = (METADATA_KEY, token_id);
        env.storage().persistent().set(&metadata_key, &cultural_metadata);
        
        // Guardar procedencia
        let provenance_key = (PROVENANCE_KEY, token_id);
        env.storage().persistent().set(&provenance_key, &provenance);
        
        // Incrementar contador
        let mut count: u32 = env.storage().instance().get(&TOKEN_COUNT_KEY).unwrap_or(0);
        count += 1;
        env.storage().instance().set(&TOKEN_COUNT_KEY, &count);
    }

    /// Obtiene el propietario de un token
    pub fn owner_of(env: Env, token_id: u32) -> Address {
        let owner_key = (OWNER_KEY, token_id);
        env.storage().persistent().get(&owner_key).unwrap_or_else(|| {
            panic!("Token does not exist");
        })
    }

    /// Obtiene metadatos culturales del token
    pub fn get_cultural_metadata(env: Env, token_id: u32) -> CulturalMetadata {
        let metadata_key = (METADATA_KEY, token_id);
        env.storage().persistent().get(&metadata_key).unwrap_or_else(|| {
            panic!("Token metadata not found");
        })
    }

    /// Obtiene la procedencia del token
    pub fn get_provenance(env: Env, token_id: u32) -> Vec<PROVENANCERecord> {
        let provenance_key = (PROVENANCE_KEY, token_id);
        env.storage().persistent().get(&provenance_key).unwrap_or_else(|| {
            panic!("Token provenance not found");
        })
    }

    /// Transfiere el token
    pub fn transfer(env: Env, from: Address, to: Address, token_id: u32) {
        // Verificar que el caller es el propietario
        from.require_auth();
        
        // Verificar que el token existe
        let owner_key = (OWNER_KEY, token_id);
        let current_owner: Address = env.storage().persistent().get(&owner_key).unwrap_or_else(|| {
            panic!("Token does not exist");
        });
        
        // Verificar que el caller es el propietario actual
        if current_owner != from {
            panic!("Not the owner");
        }
        
        // Transferir token
        env.storage().persistent().set(&owner_key, &to);
        
        // Agregar registro de procedencia
        let provenance_key = (PROVENANCE_KEY, token_id);
        let mut provenance: Vec<PROVENANCERecord> = env.storage().persistent().get(&provenance_key).unwrap_or_else(|| {
            Vec::new(&env)
        });
        
        let new_record = PROVENANCERecord {
            date: env.ledger().timestamp(),
            from: from.clone(),
            to: to.clone(),
            transaction_type: String::from_str(&env, "transfer"),
            notes: String::from_str(&env, "Direct transfer"),
        };
        
        provenance.push_back(new_record);
        env.storage().persistent().set(&provenance_key, &provenance);
    }

    /// Aprueba una transferencia (para marketplace)
    pub fn approve(env: Env, from: Address, to: Address, token_id: u32) {
        // Verificar que el caller es el propietario
        from.require_auth();
        
        // Verificar que el token existe y es del propietario
        let owner_key = (OWNER_KEY, token_id);
        let current_owner: Address = env.storage().persistent().get(&owner_key).unwrap_or_else(|| {
            panic!("Token does not exist");
        });
        
        if current_owner != from {
            panic!("Not the owner");
        }
        
        // Guardar aprobación
        let approval_key = (symbol_short!("APPROVAL"), token_id);
        env.storage().persistent().set(&approval_key, &to);
    }

    /// Transfiere desde una dirección aprobada
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, token_id: u32) {
        // Verificar que el spender está autorizado
        spender.require_auth();
        
        // Verificar que el token existe
        let owner_key = (OWNER_KEY, token_id);
        let current_owner: Address = env.storage().persistent().get(&owner_key).unwrap_or_else(|| {
            panic!("Token does not exist");
        });
        
        if current_owner != from {
            panic!("Not the owner");
        }
        
        // Verificar aprobación
        let approval_key = (symbol_short!("APPROVAL"), token_id);
        let approved: Address = env.storage().persistent().get(&approval_key).unwrap_or_else(|| {
            panic!("Not approved");
        });
        
        if approved != spender {
            panic!("Not approved");
        }
        
        // Transferir token
        env.storage().persistent().set(&owner_key, &to);
        
        // Limpiar aprobación
        env.storage().persistent().remove(&approval_key);
        
        // Agregar registro de procedencia
        let provenance_key = (PROVENANCE_KEY, token_id);
        let mut provenance: Vec<PROVENANCERecord> = env.storage().persistent().get(&provenance_key).unwrap_or_else(|| {
            Vec::new(&env)
        });
        
        let new_record = PROVENANCERecord {
            date: env.ledger().timestamp(),
            from: from.clone(),
            to: to.clone(),
            transaction_type: String::from_str(&env, "transfer_from"),
            notes: String::from_str(&env, "Approved transfer"),
        };
        
        provenance.push_back(new_record);
        env.storage().persistent().set(&provenance_key, &provenance);
    }

    /// Obtiene el nombre del token
    pub fn name(env: Env) -> String {
        env.storage().instance().get(&NAME_KEY).unwrap()
    }

    /// Obtiene el símbolo del token
    pub fn symbol(env: Env) -> String {
        env.storage().instance().get(&SYMBOL_KEY).unwrap()
    }

    /// Obtiene el total de tokens mintados
    pub fn total_supply(env: Env) -> u32 {
        env.storage().instance().get(&TOKEN_COUNT_KEY).unwrap_or(0)
    }

    /// Verifica si un token existe
    pub fn exists(env: Env, token_id: u32) -> bool {
        let owner_key = (OWNER_KEY, token_id);
        env.storage().persistent().has(&owner_key)
    }

    /// Obtiene información básica del token
    pub fn get_token_info(env: Env, token_id: u32) -> (Address, CulturalMetadata, Vec<PROVENANCERecord>) {
        let owner = Self::owner_of(env.clone(), token_id);
        let metadata = Self::get_cultural_metadata(env.clone(), token_id);
        let provenance = Self::get_provenance(env, token_id);
        
        (owner, metadata, provenance)
    }


}
