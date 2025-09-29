#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec, Symbol};

/// Contrato para el sistema SocialFi de PermaMuseum
/// 
/// Este contrato maneja:
/// - Sistema de puntos por participación
/// - Insignias y logros
/// - Recompensas por contribuciones culturales
/// - Gamificación del ecosistema
#[contract]
pub struct SocialFi;

// Claves de storage
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const POINTS_KEY: Symbol = symbol_short!("POINTS");
const BADGES_KEY: Symbol = symbol_short!("BADGES");
const REWARDS_KEY: Symbol = symbol_short!("REWARDS");
const LEADERBOARD_KEY: Symbol = symbol_short!("LEADER");
const ACTIVITY_KEY: Symbol = symbol_short!("ACTIVITY");

// Estructura para insignias
#[derive(Clone)]
#[contracttype]
pub struct Badge {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub points_required: i128,
    pub rarity: String, // "common", "rare", "epic", "legendary"
    pub category: String, // "collector", "curator", "explorer", "creator"
}

// Estructura para recompensas
#[derive(Clone)]
#[contracttype]
pub struct Reward {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub points_cost: i128,
    pub available: bool,
    pub max_redemptions: u32,
    pub current_redemptions: u32,
}

// Estructura para actividad del usuario
#[derive(Clone)]
#[contracttype]
pub struct UserActivity {
    pub user: Address,
    pub points: i128,
    pub badges: Vec<u32>,
    pub total_activity: u32,
    pub last_activity: u64,
}

// Estructura para registro de actividad
#[derive(Clone)]
#[contracttype]
pub struct ActivityRecord {
    pub user: Address,
    pub activity_type: String,
    pub points_awarded: i128,
    pub timestamp: u64,
    pub description: String,
}

#[contractimpl]
impl SocialFi {
    /// Inicializa el contrato
    pub fn initialize(env: Env, admin: Address) {
        // Verificar que no esté ya inicializado
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("Contract already initialized");
        }
        
        // Guardar admin
        env.storage().instance().set(&ADMIN_KEY, &admin);
        
        // Inicializar contadores
        env.storage().instance().set(&symbol_short!("BADGE_CNT"), &0u32);
        env.storage().instance().set(&symbol_short!("REWARD_CN"), &0u32);
    }

    /// Otorga puntos a un usuario
    pub fn award_points(
        env: Env,
        user: Address,
        points: i128,
        reason: String,
    ) {
        // Verificar que el caller es admin o un contrato autorizado
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Verificar que los puntos son positivos
        if points <= 0 {
            panic!("Points must be positive");
        }
        
        // Obtener balance actual
        let points_key = (POINTS_KEY, user.clone());
        let current_balance: i128 = env.storage().persistent().get(&points_key).unwrap_or(0);
        
        // Actualizar balance
        let new_balance = current_balance + points;
        env.storage().persistent().set(&points_key, &new_balance);
        
        // Registrar actividad
        let activity_record = ActivityRecord {
            user: user.clone(),
            activity_type: String::from_str(&env, "points_awarded"),
            points_awarded: points,
            timestamp: env.ledger().timestamp(),
            description: reason,
        };
        
        let activity_key = (ACTIVITY_KEY, user.clone(), env.ledger().timestamp());
        env.storage().persistent().set(&activity_key, &activity_record);
        
        // Actualizar leaderboard
        Self::update_leaderboard(env, user);
    }

    /// Obtiene el balance de puntos de un usuario
    pub fn get_points_balance(env: Env, user: Address) -> i128 {
        let points_key = (POINTS_KEY, user);
        env.storage().persistent().get(&points_key).unwrap_or(0)
    }

    /// Otorga una insignia a un usuario
    pub fn award_badge(
        env: Env,
        user: Address,
        badge_id: u32,
        _badge_name: String,
    ) {
        // Verificar que el caller es admin
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Obtener insignias actuales del usuario
        let badges_key = (BADGES_KEY, user.clone());
        let mut user_badges: Vec<u32> = env.storage().persistent().get(&badges_key).unwrap_or_else(|| {
            Vec::new(&env)
        });
        
        // Verificar que no tiene ya la insignia
        for badge in user_badges.iter() {
            if badge == badge_id {
                panic!("User already has this badge");
            }
        }
        
        // Agregar insignia
        user_badges.push_back(badge_id);
        env.storage().persistent().set(&badges_key, &user_badges);
        
        // Registrar actividad
        let activity_record = ActivityRecord {
            user: user.clone(),
            activity_type: String::from_str(&env, "badge_awarded"),
            points_awarded: 0,
            timestamp: env.ledger().timestamp(),
            description: String::from_str(&env, "Badge awarded"),
        };
        
        let activity_key = (ACTIVITY_KEY, user.clone(), env.ledger().timestamp());
        env.storage().persistent().set(&activity_key, &activity_record);
    }

    /// Obtiene las insignias de un usuario
    pub fn get_user_badges(env: Env, user: Address) -> Vec<u32> {
        let badges_key = (BADGES_KEY, user);
        env.storage().persistent().get(&badges_key).unwrap_or_else(|| {
            Vec::new(&env)
        })
    }

    /// Crea una nueva insignia (solo admin)
    pub fn create_badge(
        env: Env,
        name: String,
        description: String,
        points_required: i128,
        rarity: String,
        category: String,
    ) -> u32 {
        // Verificar que el caller es admin
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Obtener nuevo ID
        let badge_count_key = symbol_short!("BADGE_CNT");
        let mut count: u32 = env.storage().instance().get(&badge_count_key).unwrap_or(0);
        count += 1;
        env.storage().instance().set(&badge_count_key, &count);
        
        // Crear insignia
        let badge = Badge {
            id: count,
            name: name.clone(),
            description: description.clone(),
            points_required,
            rarity: rarity.clone(),
            category: category.clone(),
        };
        
        // Guardar insignia
        let badge_key = (symbol_short!("BADGE_INF"), count);
        env.storage().persistent().set(&badge_key, &badge);
        
        count
    }

    /// Obtiene información de una insignia
    pub fn get_badge_info(env: Env, badge_id: u32) -> Badge {
        let badge_key = (symbol_short!("BADGE_INF"), badge_id);
        env.storage().persistent().get(&badge_key).unwrap_or_else(|| {
            panic!("Badge not found");
        })
    }

    
    /// Crea una nueva recompensa (solo admin)
    pub fn create_reward(
        env: Env,
        name: String,
        description: String,
        points_cost: i128,
        max_redemptions: u32,
    ) -> u32 {
        // Verificar que el caller es admin
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Obtener nuevo ID
        let reward_count_key = symbol_short!("REWARD_CN");
        let mut count: u32 = env.storage().instance().get(&reward_count_key).unwrap_or(0);
        count += 1;
        env.storage().instance().set(&reward_count_key, &count);
        
        // Crear recompensa
        let reward = Reward {
            id: count,
            name: name.clone(),
            description: description.clone(),
            points_cost,
            available: true,
            max_redemptions,
            current_redemptions: 0,
        };
        
        // Guardar recompensa
        let reward_key = (REWARDS_KEY, count);
        env.storage().persistent().set(&reward_key, &reward);
        
        count
    }

    /// Obtiene información de una recompensa
    pub fn get_reward_info(env: Env, reward_id: u32) -> Reward {
        let reward_key = (REWARDS_KEY, reward_id);
        env.storage().persistent().get(&reward_key).unwrap_or_else(|| {
            panic!("Reward not found");
        })
    }

    /// Canjea puntos por recompensas
    pub fn redeem_points(
        env: Env,
        user: Address,
        reward_id: u32,
    ) {
        // Verificar que el caller es el usuario
        user.require_auth();
        
        // Obtener información de la recompensa
        let reward = Self::get_reward_info(env.clone(), reward_id);
        
        // Verificar que la recompensa está disponible
        if !reward.available {
            panic!("Reward not available");
        }
        
        // Verificar que no se ha agotado
        if reward.current_redemptions >= reward.max_redemptions {
            panic!("Reward sold out");
        }
        
        // Verificar que el usuario tiene suficientes puntos
        let user_balance = Self::get_points_balance(env.clone(), user.clone());
        if user_balance < reward.points_cost {
            panic!("Insufficient points");
        }
        
        // Descontar puntos
        let new_balance = user_balance - reward.points_cost;
        let points_key = (POINTS_KEY, user.clone());
        env.storage().persistent().set(&points_key, &new_balance);
        
        // Actualizar recompensa
        let mut updated_reward = reward.clone();
        updated_reward.current_redemptions += 1;
        let reward_key = (REWARDS_KEY, reward_id);
        env.storage().persistent().set(&reward_key, &updated_reward);
        
        // Registrar actividad
        let activity_record = ActivityRecord {
            user: user.clone(),
            activity_type: String::from_str(&env, "reward_redeemed"),
            points_awarded: -reward.points_cost,
            timestamp: env.ledger().timestamp(),
            description: String::from_str(&env, "Redeemed reward"),
        };
        
        let activity_key = (ACTIVITY_KEY, user.clone(), env.ledger().timestamp());
        env.storage().persistent().set(&activity_key, &activity_record);
    }

    /// Obtiene el ranking de usuarios
    pub fn get_leaderboard(env: Env, _limit: u32) -> Vec<Address> {
        let leaderboard_key = LEADERBOARD_KEY;
        env.storage().persistent().get(&leaderboard_key).unwrap_or_else(|| {
            Vec::new(&env)
        })
    }

    /// Obtiene la actividad de un usuario
    pub fn get_user_activity(env: Env, _user: Address, _limit: u32) -> Vec<ActivityRecord> {
        // TODO: Implementar obtención de actividad del usuario
        // Esto requeriría un sistema de indexación más complejo
        Vec::new(&env)
    }

    /// Obtiene estadísticas del usuario
    pub fn get_user_stats(env: Env, user: Address) -> (i128, Vec<u32>, u32) {
        let points = Self::get_points_balance(env.clone(), user.clone());
        let badges = Self::get_user_badges(env.clone(), user.clone());
        
        // Contar actividades (simplificado)
        let activity_count = 0u32; // TODO: Implementar conteo real
        
        (points, badges, activity_count)
    }

    /// Actualiza el leaderboard
    fn update_leaderboard(_env: Env, _user: Address) {
        // TODO: Implementar actualización del leaderboard
        // Esto requeriría un sistema de ordenamiento más complejo
    }

    /// Obtiene el total de usuarios con puntos
    pub fn get_total_users(_env: Env) -> u32 {
        // TODO: Implementar conteo de usuarios
        0
    }

    /// Obtiene el total de insignias creadas
    pub fn get_total_badges(env: Env) -> u32 {
        env.storage().instance().get(&symbol_short!("BADGE_CNT")).unwrap_or(0)
    }

    /// Obtiene el total de recompensas creadas
    pub fn get_total_rewards(env: Env) -> u32 {
        env.storage().instance().get(&symbol_short!("REWARD_CN")).unwrap_or(0)
    }

    /// Obtiene el admin del contrato
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN_KEY).unwrap()
    }
}
