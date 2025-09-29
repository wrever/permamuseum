#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

/// Contrato para el marketplace de NFTs culturales
/// 
/// Este contrato maneja:
/// - Ventas directas de NFTs
/// - Subastas de patrimonio cultural
/// - Distribución automática de royalties
/// - Comisiones del marketplace
#[contract]
pub struct Marketplace;


// Claves de storage
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const FEE_PERCENTAGE_KEY: Symbol = symbol_short!("FEE_PCT");
const LISTING_COUNT_KEY: Symbol = symbol_short!("LIST_CNT");
const AUCTION_COUNT_KEY: Symbol = symbol_short!("AUCT_CNT");
const LISTING_KEY: Symbol = symbol_short!("LISTING");
const AUCTION_KEY: Symbol = symbol_short!("AUCTION");
const BID_KEY: Symbol = symbol_short!("BID");

// Estructura para listado de NFT
#[derive(Clone)]
#[contracttype]
pub struct Listing {
    pub seller: Address,
    pub nft_contract: Address,
    pub token_id: u32,
    pub price: i128,
    pub active: bool,
    pub created_at: u64,
}

// Estructura para subasta
#[derive(Clone)]
#[contracttype]
pub struct Auction {
    pub seller: Address,
    pub nft_contract: Address,
    pub token_id: u32,
    pub starting_price: i128,
    pub current_bid: i128,
    pub highest_bidder: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub active: bool,
}

// Estructura para puja
#[derive(Clone)]
#[contracttype]
pub struct Bid {
    pub bidder: Address,
    pub amount: i128,
    pub timestamp: u64,
}

// Estructura para royalties
#[derive(Clone)]
#[contracttype]
pub struct RoyaltyInfo {
    pub recipient: Address,
    pub percentage: u32, // En basis points (100 = 1%)
}

#[contractimpl]
impl Marketplace {
    /// Inicializa el contrato
    pub fn initialize(env: Env, admin: Address, fee_percentage: u32) {
        // Verificar que no esté ya inicializado
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("Contract already initialized");
        }
        
        // Guardar configuración inicial
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&FEE_PERCENTAGE_KEY, &fee_percentage);
        
        // Inicializar contadores
        env.storage().instance().set(&LISTING_COUNT_KEY, &0u32);
        env.storage().instance().set(&AUCTION_COUNT_KEY, &0u32);
    }

    /// Lista un NFT para venta
    pub fn list_nft(
        env: Env,
        seller: Address,
        nft_contract: Address,
        token_id: u32,
        price: i128,
    ) {
        // Verificar que el caller es el vendedor
        seller.require_auth();
        
        // Verificar que el precio es positivo
        if price <= 0 {
            panic!("Price must be positive");
        }
        
        // Verificar que el NFT no está ya listado
        let listing_key = (LISTING_KEY, nft_contract.clone(), token_id);
        if env.storage().persistent().has(&listing_key) {
            panic!("NFT already listed");
        }
        
        // Crear listado
        let listing = Listing {
            seller: seller.clone(),
            nft_contract: nft_contract.clone(),
            token_id,
            price,
            active: true,
            created_at: env.ledger().timestamp(),
        };
        
        // Guardar listado
        env.storage().persistent().set(&listing_key, &listing);
        
        // Incrementar contador
        let mut count: u32 = env.storage().instance().get(&LISTING_COUNT_KEY).unwrap_or(0);
        count += 1;
        env.storage().instance().set(&LISTING_COUNT_KEY, &count);
    }

    /// Compra un NFT listado
    pub fn buy_nft(
        env: Env,
        buyer: Address,
        nft_contract: Address,
        token_id: u32,
    ) {
        // Verificar que el caller es el comprador
        buyer.require_auth();
        
        // Obtener listado
        let listing_key = (LISTING_KEY, nft_contract.clone(), token_id);
        let mut listing: Listing = env.storage().persistent().get(&listing_key).unwrap_or_else(|| {
            panic!("NFT not listed");
        });
        
        // Verificar que el listado está activo
        if !listing.active {
            panic!("Listing not active");
        }
        
        // Verificar que el comprador no es el vendedor
        if listing.seller == buyer {
            panic!("Cannot buy your own NFT");
        }
        
        // TODO: Implementar transferencia de tokens (XLM)
        // Por ahora solo marcamos como inactivo
        
        // Marcar listado como inactivo
        listing.active = false;
        env.storage().persistent().set(&listing_key, &listing);
        
        // TODO: Transferir NFT al comprador
        // TODO: Transferir pago al vendedor
        // TODO: Distribuir royalties
    }

    /// Crea una subasta
    pub fn create_auction(
        env: Env,
        seller: Address,
        nft_contract: Address,
        token_id: u32,
        starting_price: i128,
        duration: u64,
    ) {
        // Verificar que el caller es el vendedor
        seller.require_auth();
        
        // Verificar que el precio inicial es positivo
        if starting_price <= 0 {
            panic!("Starting price must be positive");
        }
        
        // Verificar que la duración es válida
        if duration == 0 {
            panic!("Duration must be positive");
        }
        
        // Verificar que el NFT no está ya en subasta
        let auction_key = (AUCTION_KEY, nft_contract.clone(), token_id);
        if env.storage().persistent().has(&auction_key) {
            panic!("NFT already in auction");
        }
        
        // Crear subasta
        let start_time = env.ledger().timestamp();
        let auction = Auction {
            seller: seller.clone(),
            nft_contract: nft_contract.clone(),
            token_id,
            starting_price,
            current_bid: 0,
            highest_bidder: seller.clone(), // Inicialmente el vendedor
            start_time,
            end_time: start_time + duration,
            active: true,
        };
        
        // Guardar subasta
        env.storage().persistent().set(&auction_key, &auction);
        
        // Incrementar contador
        let mut count: u32 = env.storage().instance().get(&AUCTION_COUNT_KEY).unwrap_or(0);
        count += 1;
        env.storage().instance().set(&AUCTION_COUNT_KEY, &count);
    }

    /// Hace una puja en una subasta
    pub fn bid(env: Env, bidder: Address, nft_contract: Address, token_id: u32, amount: i128) {
        // Verificar que el caller es el pujador
        bidder.require_auth();
        
        // Obtener subasta
        let auction_key = (AUCTION_KEY, nft_contract.clone(), token_id);
        let mut auction: Auction = env.storage().persistent().get(&auction_key).unwrap_or_else(|| {
            panic!("Auction not found");
        });
        
        // Verificar que la subasta está activa
        if !auction.active {
            panic!("Auction not active");
        }
        
        // Verificar que la subasta no ha terminado
        if env.ledger().timestamp() >= auction.end_time {
            panic!("Auction ended");
        }
        
        // Verificar que la puja es mayor que la actual
        if amount <= auction.current_bid {
            panic!("Bid must be higher than current bid");
        }
        
        // Verificar que la puja es mayor que el precio inicial
        if amount < auction.starting_price {
            panic!("Bid must be at least starting price");
        }
        
        // Devolver puja anterior si existe
        if auction.current_bid > 0 {
            // TODO: Devolver tokens al pujador anterior
        }
        
        // Actualizar subasta
        auction.current_bid = amount;
        auction.highest_bidder = bidder.clone();
        
        // Guardar subasta actualizada
        env.storage().persistent().set(&auction_key, &auction);
        
        // Guardar puja
        let bid_key = (BID_KEY, nft_contract, token_id, bidder.clone());
        let bid = Bid {
            bidder: bidder.clone(),
            amount,
            timestamp: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&bid_key, &bid);
    }

    /// Finaliza una subasta
    pub fn end_auction(env: Env, nft_contract: Address, token_id: u32) {
        // Obtener subasta
        let auction_key = (AUCTION_KEY, nft_contract.clone(), token_id);
        let mut auction: Auction = env.storage().persistent().get(&auction_key).unwrap_or_else(|| {
            panic!("Auction not found");
        });
        
        // Verificar que la subasta está activa
        if !auction.active {
            panic!("Auction not active");
        }
        
        // Verificar que la subasta ha terminado
        if env.ledger().timestamp() < auction.end_time {
            panic!("Auction not ended yet");
        }
        
        // Marcar subasta como inactiva
        auction.active = false;
        env.storage().persistent().set(&auction_key, &auction);
        
        // Si hay pujas, transferir NFT al ganador
        if auction.current_bid > 0 {
            // TODO: Transferir NFT al ganador
            // TODO: Transferir pago al vendedor
            // TODO: Distribuir royalties
        }
    }

    /// Obtiene información de un listado
    pub fn get_listing(env: Env, nft_contract: Address, token_id: u32) -> Listing {
        let listing_key = (LISTING_KEY, nft_contract, token_id);
        env.storage().persistent().get(&listing_key).unwrap_or_else(|| {
            panic!("Listing not found");
        })
    }

    /// Obtiene información de una subasta
    pub fn get_auction(env: Env, nft_contract: Address, token_id: u32) -> Auction {
        let auction_key = (AUCTION_KEY, nft_contract, token_id);
        env.storage().persistent().get(&auction_key).unwrap_or_else(|| {
            panic!("Auction not found");
        })
    }

    /// Obtiene la puja más alta de una subasta
    pub fn get_highest_bid(env: Env, nft_contract: Address, token_id: u32) -> i128 {
        let auction = Self::get_auction(env, nft_contract, token_id);
        auction.current_bid
    }

    /// Obtiene el pujador más alto de una subasta
    pub fn get_highest_bidder(env: Env, nft_contract: Address, token_id: u32) -> Address {
        let auction = Self::get_auction(env, nft_contract, token_id);
        auction.highest_bidder
    }

    /// Cancela un listado
    pub fn cancel_listing(env: Env, seller: Address, nft_contract: Address, token_id: u32) {
        // Verificar que el caller es el vendedor
        seller.require_auth();
        
        // Obtener listado
        let listing_key = (LISTING_KEY, nft_contract.clone(), token_id);
        let mut listing: Listing = env.storage().persistent().get(&listing_key).unwrap_or_else(|| {
            panic!("Listing not found");
        });
        
        // Verificar que el caller es el vendedor
        if listing.seller != seller {
            panic!("Not the seller");
        }
        
        // Verificar que el listado está activo
        if !listing.active {
            panic!("Listing not active");
        }
        
        // Marcar como inactivo
        listing.active = false;
        env.storage().persistent().set(&listing_key, &listing);
    }

    /// Cancela una subasta
    pub fn cancel_auction(env: Env, seller: Address, nft_contract: Address, token_id: u32) {
        // Verificar que el caller es el vendedor
        seller.require_auth();
        
        // Obtener subasta
        let auction_key = (AUCTION_KEY, nft_contract.clone(), token_id);
        let mut auction: Auction = env.storage().persistent().get(&auction_key).unwrap_or_else(|| {
            panic!("Auction not found");
        });
        
        // Verificar que el caller es el vendedor
        if auction.seller != seller {
            panic!("Not the seller");
        }
        
        // Verificar que la subasta está activa
        if !auction.active {
            panic!("Auction not active");
        }
        
        // Verificar que no hay pujas
        if auction.current_bid > 0 {
            panic!("Cannot cancel auction with bids");
        }
        
        // Marcar como inactiva
        auction.active = false;
        env.storage().persistent().set(&auction_key, &auction);
    }

    /// Obtiene el porcentaje de comisión del marketplace
    pub fn get_fee_percentage(env: Env) -> u32 {
        env.storage().instance().get(&FEE_PERCENTAGE_KEY).unwrap()
    }

    /// Obtiene el total de listados
    pub fn get_total_listings(env: Env) -> u32 {
        env.storage().instance().get(&LISTING_COUNT_KEY).unwrap_or(0)
    }

    /// Obtiene el total de subastas
    pub fn get_total_auctions(env: Env) -> u32 {
        env.storage().instance().get(&AUCTION_COUNT_KEY).unwrap_or(0)
    }

    /// Distribuye royalties automáticamente
    pub fn distribute_royalties(
        _env: Env,
        _nft_contract: Address,
        _token_id: u32,
        _sale_price: i128,
    ) {
        // TODO: Implementar distribución de royalties
        // Esto requeriría integración con el contrato de NFT
        // para obtener información de royalties
    }
}
