use specs::prelude::*;
use crate::systems::{
    VisibilitySystem, MovementSystem, RenderSystem, PlayerController,
    ExperienceSystem, LevelUpSystem, AbilitySystem, ExperienceGainSystem,
    EquipmentSystem, EquipmentBonusSystem, ResourceRegenerationSystem,
    StatusEffectSystem, AbilityUsageSystem, PlayerDeathSystem,
    DeathPenaltySystem, RevivalSystem, GameOverSystem, EnhancedCombatSystem,
    EnhancedDamageSystem, InitiativeSystem, TurnOrderSystem, CombatResolutionSystem,
    CriticalHitSystem, CriticalChanceSystem, DamageTypeSystem, ResistanceManagementSystem,
    CombatFeedbackSystem, SoundEffectSystem, ScreenShakeSystem, VisualEffectsSystem,
    ParticleEffectSystem, ScreenShakeState, SpecialAbilitiesSystem, AbilityTargetingSystem,
    AbilityCooldownSystem, CombatRewardsSystem, TreasureSystem
};
use crate::inventory::{InventorySystem, EquipmentSystem, ItemUseSystem};
use crate::combat::{CombatSystem, DamageSystem, DeathSystem};

pub struct SystemRunner {
    pub visibility_system: VisibilitySystem,
    pub movement_system: MovementSystem,
    pub render_system: RenderSystem,
    pub player_controller: PlayerController,
    pub experience_system: ExperienceSystem,
    pub level_up_system: LevelUpSystem,
    pub ability_system: AbilitySystem,
    pub experience_gain_system: ExperienceGainSystem,
    pub equipment_system: EquipmentSystem,
    pub equipment_bonus_system: EquipmentBonusSystem,
    pub resource_regeneration_system: ResourceRegenerationSystem,
    pub status_effect_system: StatusEffectSystem,
    pub ability_usage_system: AbilityUsageSystem,
    pub player_death_system: PlayerDeathSystem,
    pub death_penalty_system: DeathPenaltySystem,
    pub revival_system: RevivalSystem,
    pub game_over_system: GameOverSystem,
    pub enhanced_combat_system: EnhancedCombatSystem,
    pub enhanced_damage_system: EnhancedDamageSystem,
    pub initiative_system: InitiativeSystem,
    pub turn_order_system: TurnOrderSystem,
    pub combat_resolution_system: CombatResolutionSystem,
    pub critical_hit_system: CriticalHitSystem,
    pub critical_chance_system: CriticalChanceSystem,
    pub damage_type_system: DamageTypeSystem,
    pub resistance_management_system: ResistanceManagementSystem,
    pub combat_feedback_system: CombatFeedbackSystem,
    pub sound_effect_system: SoundEffectSystem,
    pub screen_shake_system: ScreenShakeSystem,
    pub visual_effects_system: VisualEffectsSystem,
    pub particle_effect_system: ParticleEffectSystem,
    pub special_abilities_system: SpecialAbilitiesSystem,
    pub ability_targeting_system: AbilityTargetingSystem,
    pub ability_cooldown_system: AbilityCooldownSystem,
    pub combat_rewards_system: CombatRewardsSystem,
    pub treasure_system: TreasureSystem,
    pub inventory_system: InventorySystem,
    pub equipment_system: EquipmentSystem,
    pub item_use_system: ItemUseSystem,
    pub combat_system: CombatSystem,
    pub damage_system: DamageSystem,
    pub death_system: DeathSystem,
}

impl SystemRunner {
    pub fn new() -> Self {
        SystemRunner {
            visibility_system: VisibilitySystem {},
            movement_system: MovementSystem {},
            render_system: RenderSystem::new(),
            player_controller: PlayerController {},
            experience_system: ExperienceSystem {},
            level_up_system: LevelUpSystem {},
            ability_system: AbilitySystem {},
            experience_gain_system: ExperienceGainSystem {},
            equipment_system: EquipmentSystem {},
            equipment_bonus_system: EquipmentBonusSystem {},
            resource_regeneration_system: ResourceRegenerationSystem {},
            status_effect_system: StatusEffectSystem {},
            ability_usage_system: AbilityUsageSystem {},
            player_death_system: PlayerDeathSystem {},
            death_penalty_system: DeathPenaltySystem {},
            revival_system: RevivalSystem {},
            game_over_system: GameOverSystem {},
            enhanced_combat_system: EnhancedCombatSystem {},
            enhanced_damage_system: EnhancedDamageSystem {},
            initiative_system: InitiativeSystem {},
            turn_order_system: TurnOrderSystem {},
            combat_resolution_system: CombatResolutionSystem {},
            critical_hit_system: CriticalHitSystem {},
            critical_chance_system: CriticalChanceSystem {},
            damage_type_system: DamageTypeSystem {},
            resistance_management_system: ResistanceManagementSystem {},
            combat_feedback_system: CombatFeedbackSystem {},
            sound_effect_system: SoundEffectSystem {},
            screen_shake_system: ScreenShakeSystem {},
            visual_effects_system: VisualEffectsSystem {},
            particle_effect_system: ParticleEffectSystem {},
            special_abilities_system: SpecialAbilitiesSystem {},
            ability_targeting_system: AbilityTargetingSystem {},
            ability_cooldown_system: AbilityCooldownSystem {},
            combat_rewards_system: CombatRewardsSystem {},
            treasure_system: TreasureSystem {},
            inventory_system: InventorySystem {},
            equipment_system: EquipmentSystem {},
            item_use_system: ItemUseSystem {},
            combat_system: CombatSystem {},
            damage_system: DamageSystem {},
            death_system: DeathSystem {},
        }
    }
    
    pub fn run_systems(&mut self, world: &mut World) {
        // Run the player controller system
        self.player_controller.run_now(world);
        
        // Run the visibility system
        self.visibility_system.run_now(world);
        
        // Run the movement system
        self.movement_system.run_now(world);
        
        // Run the combat systems
        self.initiative_system.run_now(world);
        self.turn_order_system.run_now(world);
        self.critical_chance_system.run_now(world);
        self.resistance_management_system.run_now(world);
        self.combat_resolution_system.run_now(world);
        self.critical_hit_system.run_now(world);
        self.damage_type_system.run_now(world);
        self.enhanced_combat_system.run_now(world);
        self.enhanced_damage_system.run_now(world);
        self.combat_system.run_now(world);
        self.damage_system.run_now(world);
        self.death_system.run_now(world);
        
        // Run the inventory systems
        self.inventory_system.run_now(world);
        self.equipment_system.run_now(world);
        self.item_use_system.run_now(world);
        
        // Run the equipment bonus system
        self.equipment_bonus_system.run_now(world);
        
        // Run the resource systems
        self.resource_regeneration_system.run_now(world);
        self.status_effect_system.run_now(world);
        self.ability_usage_system.run_now(world);
        
        // Run the combat rewards system
        self.combat_rewards_system.run_now(world);
        
        // Run the treasure system
        self.treasure_system.run_now(world);
        
        // Run the experience gain system to award XP for kills
        self.experience_gain_system.run_now(world);
        
        // Run the experience system to check for level ups
        self.experience_system.run_now(world);
        
        // Run the level up system to apply level up bonuses
        self.level_up_system.run_now(world);
        
        // Run the ability systems
        self.ability_cooldown_system.run_now(world);
        self.ability_targeting_system.run_now(world);
        self.special_abilities_system.run_now(world);
        self.ability_system.run_now(world);
        
        // Run the death and revival systems
        self.player_death_system.run_now(world);
        self.death_penalty_system.run_now(world);
        self.revival_system.run_now(world);
        self.game_over_system.run_now(world);
        
        // Run the combat feedback systems
        self.combat_feedback_system.run_now(world);
        self.sound_effect_system.run_now(world);
        self.screen_shake_system.run_now(world);
        self.visual_effects_system.run_now(world);
        self.particle_effect_system.run_now(world);
        
        // Apply changes to the world
        world.maintain();
    }
    
    pub fn render(&mut self, world: &World) {
        // Run the render system
        self.render_system.run_now(world);
    }
}