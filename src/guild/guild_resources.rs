use bevy::prelude::*;
use std::collections::HashMap;
use crate::guild::*;

/// Resource income source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceSource {
    Missions,
    Facilities,
    Members,
    Trading,
    Events,
}

impl ResourceSource {
    /// Get the name of this source
    pub fn name(&self) -> &'static str {
        match self {
            ResourceSource::Missions => "Missions",
            ResourceSource::Facilities => "Facilities",
            ResourceSource::Members => "Members",
            ResourceSource::Trading => "Trading",
            ResourceSource::Events => "Events",
        }
    }
}

/// Resource transaction record
#[derive(Debug, Clone)]
pub struct ResourceTransaction {
    pub resource: GuildResource,
    pub amount: i32,
    pub source: ResourceSource,
    pub timestamp: f64,
    pub description: String,
}

impl ResourceTransaction {
    /// Create a new resource transaction
    pub fn new(resource: GuildResource, amount: i32, source: ResourceSource, timestamp: f64, description: String) -> Self {
        ResourceTransaction {
            resource,
            amount,
            source,
            timestamp,
            description,
        }
    }

    /// Check if this is an income transaction
    pub fn is_income(&self) -> bool {
        self.amount > 0
    }

    /// Check if this is an expense transaction
    pub fn is_expense(&self) -> bool {
        self.amount < 0
    }
}

/// Resource for tracking guild resource transactions
#[derive(Resource)]
pub struct GuildResourceTracker {
    pub transactions: Vec<ResourceTransaction>,
    pub daily_income: HashMap<GuildResource, u32>,
    pub daily_expenses: HashMap<GuildResource, u32>,
    pub last_update_day: u32,
}

impl Default for GuildResourceTracker {
    fn default() -> Self {
        let mut daily_income = HashMap::new();
        let mut daily_expenses = HashMap::new();
        
        for resource in GuildResource::all() {
            daily_income.insert(resource, 0);
            daily_expenses.insert(resource, 0);
        }
        
        GuildResourceTracker {
            transactions: Vec::new(),
            daily_income,
            daily_expenses,
            last_update_day: 0,
        }
    }
}

impl GuildResourceTracker {
    /// Add a transaction to the tracker
    pub fn add_transaction(&mut self, transaction: ResourceTransaction) {
        // Update daily income/expenses
        let resource = transaction.resource;
        let amount = transaction.amount.abs() as u32;
        
        if transaction.is_income() {
            *self.daily_income.entry(resource).or_insert(0) += amount;
        } else {
            *self.daily_expenses.entry(resource).or_insert(0) += amount;
        }
        
        self.transactions.push(transaction);
    }

    /// Get transactions for a specific resource
    pub fn get_transactions_for_resource(&self, resource: GuildResource) -> Vec<&ResourceTransaction> {
        self.transactions.iter()
            .filter(|t| t.resource == resource)
            .collect()
    }

    /// Get transactions from a specific source
    pub fn get_transactions_from_source(&self, source: ResourceSource) -> Vec<&ResourceTransaction> {
        self.transactions.iter()
            .filter(|t| t.source == source)
            .collect()
    }

    /// Get recent transactions
    pub fn get_recent_transactions(&self, count: usize) -> Vec<&ResourceTransaction> {
        self.transactions.iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Reset daily tracking
    pub fn reset_daily_tracking(&mut self) {
        for value in self.daily_income.values_mut() {
            *value = 0;
        }
        
        for value in self.daily_expenses.values_mut() {
            *value = 0;
        }
    }

    /// Calculate net income for a resource
    pub fn net_income(&self, resource: GuildResource) -> i32 {
        let income = *self.daily_income.get(&resource).unwrap_or(&0) as i32;
        let expenses = *self.daily_expenses.get(&resource).unwrap_or(&0) as i32;
        income - expenses
    }
}

/// System for generating passive income from facilities
pub fn facility_income_system(
    time: Res<Time>,
    mut guild_manager: ResMut<GuildManager>,
    mut resource_tracker: ResMut<GuildResourceTracker>,
) {
    // Only run once per game day
    let current_day = (time.elapsed_seconds() / 86400.0) as u32;
    if current_day <= resource_tracker.last_update_day {
        return;
    }
    resource_tracker.last_update_day = current_day;

    // Process each guild
    for guild in guild_manager.guilds.values_mut() {
        // Generate income from facilities
        for (facility_type, instance) in &guild.facilities {
            let income = calculate_facility_income(*facility_type, instance);
            
            for (resource, amount) in income {
                guild.add_resource(resource, amount);
                
                // Record transaction
                let transaction = ResourceTransaction::new(
                    resource,
                    amount as i32,
                    ResourceSource::Facilities,
                    time.elapsed_seconds_f64(),
                    format!("Income from {} (Level {})", facility_type.name(), instance.level),
                );
                resource_tracker.add_transaction(transaction);
            }
        }
    }
}

/// Calculate income generated by a facility
fn calculate_facility_income(facility_type: GuildFacility, instance: &GuildFacilityInstance) -> HashMap<GuildResource, u32> {
    let mut income = HashMap::new();
    let effectiveness = instance.effectiveness();
    
    match facility_type {
        GuildFacility::Headquarters => {
            income.insert(GuildResource::Reputation, (5.0 * effectiveness) as u32);
        }
        GuildFacility::TrainingHall => {
            // Training halls don't generate direct income
        }
        GuildFacility::Library => {
            income.insert(GuildResource::Reputation, (10.0 * effectiveness) as u32);
        }
        GuildFacility::Workshop => {
            income.insert(GuildResource::Gold, (15.0 * effectiveness) as u32);
            income.insert(GuildResource::Supplies, (5.0 * effectiveness) as u32);
        }
        GuildFacility::Infirmary => {
            // Infirmaries don't generate direct income
        }
        GuildFacility::MagicLab => {
            income.insert(GuildResource::MagicEssence, (3.0 * effectiveness) as u32);
        }
        GuildFacility::Vault => {
            // Vaults increase gold income by percentage
            income.insert(GuildResource::Gold, (5.0 * effectiveness) as u32);
        }
        GuildFacility::Garden => {
            income.insert(GuildResource::Supplies, (10.0 * effectiveness) as u32);
        }
        GuildFacility::Forge => {
            income.insert(GuildResource::Gold, (20.0 * effectiveness) as u32);
        }
        GuildFacility::TavernHall => {
            income.insert(GuildResource::Gold, (10.0 * effectiveness) as u32);
            income.insert(GuildResource::Reputation, (5.0 * effectiveness) as u32);
        }
    }
    
    income
}

/// System for calculating upkeep costs
pub fn guild_upkeep_system(
    time: Res<Time>,
    mut guild_manager: ResMut<GuildManager>,
    mut resource_tracker: ResMut<GuildResourceTracker>,
) {
    // Only run once per game day
    let current_day = (time.elapsed_seconds() / 86400.0) as u32;
    if current_day <= resource_tracker.last_update_day {
        return;
    }
    resource_tracker.last_update_day = current_day;

    // Process each guild
    for guild in guild_manager.guilds.values_mut() {
        // Calculate upkeep costs
        let mut upkeep = HashMap::new();
        
        // Base upkeep
        upkeep.insert(GuildResource::Gold, 10 * guild.level);
        upkeep.insert(GuildResource::Supplies, 5 * guild.level);
        
        // Facility upkeep
        for (facility_type, instance) in &guild.facilities {
            let facility_upkeep = calculate_facility_upkeep(*facility_type, instance);
            
            for (resource, amount) in facility_upkeep {
                *upkeep.entry(resource).or_insert(0) += amount;
            }
        }
        
        // Apply upkeep costs
        for (resource, amount) in upkeep {
            if guild.remove_resource(resource, amount) {
                // Record transaction
                let transaction = ResourceTransaction::new(
                    resource,
                    -(amount as i32),
                    ResourceSource::Facilities,
                    time.elapsed_seconds_f64(),
                    format!("Daily upkeep costs"),
                );
                resource_tracker.add_transaction(transaction);
            } else {
                // Not enough resources - consequences
                handle_insufficient_resources(guild, resource);
            }
        }
    }
}

/// Calculate upkeep cost for a facility
fn calculate_facility_upkeep(facility_type: GuildFacility, instance: &GuildFacilityInstance) -> HashMap<GuildResource, u32> {
    let mut upkeep = HashMap::new();
    let level_factor = instance.level as f32;
    
    match facility_type {
        GuildFacility::Headquarters => {
            upkeep.insert(GuildResource::Gold, (10.0 * level_factor) as u32);
            upkeep.insert(GuildResource::Supplies, (5.0 * level_factor) as u32);
        }
        GuildFacility::TrainingHall => {
            upkeep.insert(GuildResource::Gold, (5.0 * level_factor) as u32);
            upkeep.insert(GuildResource::Supplies, (8.0 * level_factor) as u32);
        }
        GuildFacility::Library => {
            upkeep.insert(GuildResource::Gold, (8.0 * level_factor) as u32);
        }
        GuildFacility::Workshop => {
            upkeep.insert(GuildResource::Gold, (7.0 * level_factor) as u32);
            upkeep.insert(GuildResource::Supplies, (10.0 * level_factor) as u32);
        }
        GuildFacility::Infirmary => {
            upkeep.insert(GuildResource::Gold, (6.0 * level_factor) as u32);
            upkeep.insert(GuildResource::Supplies, (5.0 * level_factor) as u32);
        }
        GuildFacility::MagicLab => {
            upkeep.insert(GuildResource::Gold, (15.0 * level_factor) as u32);
            upkeep.insert(GuildResource::MagicEssence, (2.0 * level_factor) as u32);
        }
        GuildFacility::Vault => {
            upkeep.insert(GuildResource::Gold, (5.0 * level_factor) as u32);
        }
        GuildFacility::Garden => {
            upkeep.insert(GuildResource::Gold, (3.0 * level_factor) as u32);
            upkeep.insert(GuildResource::Supplies, (2.0 * level_factor) as u32);
        }
        GuildFacility::Forge => {
            upkeep.insert(GuildResource::Gold, (12.0 * level_factor) as u32);
            upkeep.insert(GuildResource::Supplies, (15.0 * level_factor) as u32);
        }
        GuildFacility::TavernHall => {
            upkeep.insert(GuildResource::Gold, (8.0 * level_factor) as u32);
            upkeep.insert(GuildResource::Supplies, (10.0 * level_factor) as u32);
        }
    }
    
    upkeep
}

/// Handle insufficient resources for upkeep
fn handle_insufficient_resources(guild: &mut Guild, resource: GuildResource) {
    // Decrease reputation
    if guild.reputation >= 10 {
        guild.reputation -= 10;
    } else {
        guild.reputation = 0;
    }
    
    // TODO: Add more consequences like facility degradation
}

/// Plugin for guild resource management
pub struct GuildResourcePlugin;

impl Plugin for GuildResourcePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuildResourceTracker>()
            .add_systems(Update, (
                facility_income_system,
                guild_upkeep_system,
            ).chain());
    }
}