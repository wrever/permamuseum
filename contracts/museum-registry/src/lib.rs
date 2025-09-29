#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec, Symbol};

/// Contrato para el registro de museos verificados
/// 
/// Este contrato permite:
/// - Registrar museos con información verificada
/// - Validar autenticidad de museos
/// - Gestionar metadatos de museos
/// - Control de acceso y permisos
#[contract]
pub struct MuseumRegistry;

// Claves de storage
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const MUSEUM_COUNT_KEY: Symbol = symbol_short!("MUS_CNT");
const MUSEUM_VERIFIED_KEY: Symbol = symbol_short!("MUS_VER");

// Estructura de datos para información del museo
#[derive(Clone)]
#[contracttype]
pub struct MuseumInfo {
    pub name: String,
    pub description: String,
    pub metadata: Vec<String>,
    pub registration_date: u64,
    pub verified: bool,
}

#[contractimpl]
impl MuseumRegistry {
    /// Inicializa el contrato
    pub fn initialize(env: Env, admin: Address) {
        // Verificar que no esté ya inicializado
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("Contract already initialized");
        }
        
        // Guardar admin
        env.storage().instance().set(&ADMIN_KEY, &admin);
        
        // Inicializar contador de museos
        env.storage().instance().set(&MUSEUM_COUNT_KEY, &0u32);
    }

    /// Registra un nuevo museo (solo admin)
    pub fn register_museum(
        env: Env,
        museum_address: Address,
        name: String,
        description: String,
        metadata: Vec<String>,
    ) {
        // Verificar que el caller es admin
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Verificar que el museo no esté ya registrado
        let museum_key = symbol_short!("MUSEUM");
        let museum_storage_key = (museum_key, museum_address.clone());
        
        if env.storage().persistent().has(&museum_storage_key) {
            panic!("Museum already registered");
        }
        
        // Crear información del museo
        let museum_info = MuseumInfo {
            name: name.clone(),
            description: description.clone(),
            metadata: metadata.clone(),
            registration_date: env.ledger().timestamp(),
            verified: false, // Requiere verificación manual
        };
        
        // Guardar información del museo
        env.storage().persistent().set(&museum_storage_key, &museum_info);
        
        // Marcar como no verificado inicialmente
        let verified_key = (MUSEUM_VERIFIED_KEY, museum_address);
        env.storage().persistent().set(&verified_key, &false);
        
        // Incrementar contador
        let mut count: u32 = env.storage().instance().get(&MUSEUM_COUNT_KEY).unwrap_or(0);
        count += 1;
        env.storage().instance().set(&MUSEUM_COUNT_KEY, &count);
    }

    /// Verifica un museo (solo admin)
    pub fn verify_museum(env: Env, museum_address: Address) {
        // Verificar que el caller es admin
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Verificar que el museo existe
        let museum_key = symbol_short!("MUSEUM");
        let museum_storage_key = (museum_key, museum_address.clone());
        
        if !env.storage().persistent().has(&museum_storage_key) {
            panic!("Museum not found");
        }
        
        // Marcar como verificado
        let verified_key = (MUSEUM_VERIFIED_KEY, museum_address);
        env.storage().persistent().set(&verified_key, &true);
        
        // Actualizar información del museo
        let mut museum_info: MuseumInfo = env.storage().persistent().get(&museum_storage_key).unwrap();
        museum_info.verified = true;
        env.storage().persistent().set(&museum_storage_key, &museum_info);
    }

    /// Verifica si un museo está registrado y verificado
    pub fn is_verified(env: Env, museum_address: Address) -> bool {
        let verified_key = (MUSEUM_VERIFIED_KEY, museum_address);
        env.storage().persistent().get(&verified_key).unwrap_or(false)
    }

    /// Obtiene información completa del museo
    pub fn get_museum_info(env: Env, museum_address: Address) -> MuseumInfo {
        let museum_key = symbol_short!("MUSEUM");
        let museum_storage_key = (museum_key, museum_address);
        
        env.storage().persistent().get(&museum_storage_key).unwrap_or_else(|| {
            panic!("Museum not found");
        })
    }
    

    /// Obtiene solo el nombre del museo
    pub fn get_museum_name(env: Env, museum_address: Address) -> String {
        let museum_info = Self::get_museum_info(env, museum_address);
        museum_info.name
    }

    /// Obtiene la descripción del museo
    pub fn get_museum_description(env: Env, museum_address: Address) -> String {
        let museum_info = Self::get_museum_info(env, museum_address);
        museum_info.description
    }

    /// Obtiene los metadatos del museo
    pub fn get_museum_metadata(env: Env, museum_address: Address) -> Vec<String> {
        let museum_info = Self::get_museum_info(env, museum_address);
        museum_info.metadata
    }

    /// Obtiene el total de museos registrados
    pub fn get_total_museums(env: Env) -> u32 {
        env.storage().instance().get(&MUSEUM_COUNT_KEY).unwrap_or(0)
    }

    /// Obtiene el admin del contrato
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN_KEY).unwrap()
    }

    /// Actualiza información del museo (solo admin)
    pub fn update_museum_info(
        env: Env,
        museum_address: Address,
        name: Option<String>,
        description: Option<String>,
        metadata: Option<Vec<String>>,
    ) {
        // Verificar que el caller es admin
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        let museum_key = symbol_short!("MUSEUM");
        let museum_storage_key = (museum_key, museum_address);
        
        let mut museum_info: MuseumInfo = env.storage().persistent().get(&museum_storage_key).unwrap_or_else(|| {
            panic!("Museum not found");
        });
        
        // Actualizar campos si se proporcionan
        if let Some(new_name) = name {
            museum_info.name = new_name;
        }
        if let Some(new_description) = description {
            museum_info.description = new_description;
        }
        if let Some(new_metadata) = metadata {
            museum_info.metadata = new_metadata;
        }
        
        // Guardar información actualizada
        env.storage().persistent().set(&museum_storage_key, &museum_info);
    }
}
